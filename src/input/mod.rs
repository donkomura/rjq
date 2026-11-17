mod parse;

use jaq_json::Val;
use parse::ParsedVal;
use std::io::{BufReader, Read};
use std::sync::mpsc::{Receiver, SyncSender, sync_channel};
use std::thread;

// Bounds how many parsed values can wait for the UI thread, so a fast parser
// cannot buffer an unbounded part of the input ahead of the consumer.
const CHANNEL_CAPACITY: usize = 16;

/// A value parsed on the reader thread whose ownership moves to the UI thread.
///
/// `Val` is built from `Rc`s and therefore is not `Send`. Parsing directly into
/// `Val` on the reader thread avoids holding a second, `Send` copy of the input
/// (such as `serde_json::Value`) alongside it.
#[derive(Debug, PartialEq)]
pub struct OwnedVal(Val);

// SAFETY: An `OwnedVal` is the only owner of every `Rc` reachable from it. The
// reader thread creates it from a freshly parsed value whose key interner has
// already been dropped, sends it, and keeps no clone, so no reference count is
// ever touched from two threads.
unsafe impl Send for OwnedVal {}

impl OwnedVal {
    /// # Safety
    ///
    /// No `Rc` reachable from `value` may be shared with anything outside it.
    pub(crate) unsafe fn new(value: Val) -> Self {
        Self(value)
    }

    pub fn into_inner(self) -> Val {
        self.0
    }
}

#[derive(Debug, PartialEq)]
pub enum InputEvent {
    Value(OwnedVal),
    Failed(String),
    Finished,
}

pub fn spawn_reader<R: Read + Send + 'static>(reader: R) -> Receiver<InputEvent> {
    let (sender, receiver) = sync_channel(CHANNEL_CAPACITY);
    thread::spawn(move || stream_values(reader, sender));
    receiver
}

fn stream_values<R: Read>(reader: R, sender: SyncSender<InputEvent>) {
    let values =
        serde_json::Deserializer::from_reader(BufReader::new(reader)).into_iter::<ParsedVal>();
    for value in values {
        let event = match value {
            // SAFETY: `ParsedVal` owns a value built by a single deserialize call,
            // and the interner used during that call has already been dropped.
            Ok(ParsedVal(value)) => InputEvent::Value(unsafe { OwnedVal::new(value) }),
            Err(e) => {
                let _ = sender.send(InputEvent::Failed(e.to_string()));
                return;
            }
        };
        if sender.send(event).is_err() {
            return;
        }
    }
    let _ = sender.send(InputEvent::Finished);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::into_val;
    use serde_json::{Value, json};
    use std::io::Cursor;

    fn collect_events(input: &'static str) -> Vec<InputEvent> {
        spawn_reader(Cursor::new(input)).iter().collect()
    }

    fn value_event(value: Value) -> InputEvent {
        // SAFETY: the converted value is freshly built and not shared.
        InputEvent::Value(unsafe { OwnedVal::new(into_val(value)) })
    }

    #[test]
    fn test_spawn_reader_streams_values() {
        let tests = vec![
            ("empty input", "", vec![InputEvent::Finished]),
            ("whitespace only", " \n\t", vec![InputEvent::Finished]),
            (
                "single value",
                r#"{"a": 1}"#,
                vec![value_event(json!({"a": 1})), InputEvent::Finished],
            ),
            (
                "newline delimited values",
                "{\"a\": 1}\n{\"a\": 2}\n",
                vec![
                    value_event(json!({"a": 1})),
                    value_event(json!({"a": 2})),
                    InputEvent::Finished,
                ],
            ),
            (
                "concatenated values",
                "1 2[3]",
                vec![
                    value_event(json!(1)),
                    value_event(json!(2)),
                    value_event(json!([3])),
                    InputEvent::Finished,
                ],
            ),
        ];

        for (name, input, expected) in tests {
            assert_eq!(expected, collect_events(input), "{name}");
        }
    }

    #[test]
    fn test_spawn_reader_reports_parse_error_after_valid_values() {
        let events = collect_events("{\"a\": 1}\n{\"a\": }");

        assert_eq!(2, events.len());
        assert_eq!(value_event(json!({"a": 1})), events[0]);
        assert!(
            matches!(events[1], InputEvent::Failed(_)),
            "{:?}",
            events[1]
        );
    }

    #[test]
    fn test_spawn_reader_stops_when_receiver_is_dropped() {
        let input = "1\n".repeat(10_000);
        let receiver = spawn_reader(Cursor::new(input));

        assert_eq!(value_event(json!(1)), receiver.recv().unwrap());
        drop(receiver);
    }
}
