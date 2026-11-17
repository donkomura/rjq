use jaq_json::Val;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

// Containers whose subtree spans at least this many lines keep a prefix sum of
// their children's line counts, so both counting and seeking into them are cheap.
const MEMO_MIN_LINES: usize = 1024;

/// Query outputs rendered in the same layout as `serde_json::to_string_pretty`,
/// producing only the requested window of lines instead of the whole text.
/// Zero outputs render as `null` and multiple outputs render as one array.
pub struct PrettyDocument {
    outputs: Vec<Val>,
    output_prefix: Vec<usize>,
    counter: LineCounter,
}

impl PrettyDocument {
    pub fn new() -> Self {
        Self {
            outputs: Vec::new(),
            output_prefix: vec![0],
            counter: LineCounter::default(),
        }
    }

    pub fn push(&mut self, value: Val) {
        let lines = self.counter.count(&value);
        let last = self.output_prefix[self.output_prefix.len() - 1];
        self.output_prefix.push(last + lines);
        self.outputs.push(value);
    }

    pub fn clear(&mut self) {
        self.outputs.clear();
        self.output_prefix = vec![0];
        self.counter = LineCounter::default();
    }

    pub fn len(&self) -> usize {
        self.outputs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.outputs.is_empty()
    }

    pub fn total_lines(&self) -> usize {
        match self.outputs.len() {
            0 => 1,
            1 => self.output_prefix[1],
            n => 2 + self.output_prefix[n],
        }
    }

    pub fn lines(&self, offset: usize, height: usize) -> Vec<String> {
        let mut window = Window {
            counter: &self.counter,
            skip: offset,
            remaining: height,
            lines: Vec::new(),
        };
        match self.outputs.as_slice() {
            [] => window.emit(|| "null".to_string()),
            [single] => window.value(single, 0, None, false),
            outputs => {
                window.emit(|| "[".to_string());
                window.children(outputs.len(), Some(&self.output_prefix), 1, |i| {
                    (None, &outputs[i])
                });
                window.emit(|| "]".to_string());
            }
        }
        window.lines
    }
}

impl Default for PrettyDocument {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Default)]
struct LineCounter {
    // Keyed by the address of the container's Rc allocation. Every memoized
    // container is reachable from the owning document's outputs, so an address
    // cannot be reused while its entry exists.
    children_prefix: RefCell<HashMap<usize, Rc<[usize]>>>,
}

impl LineCounter {
    fn count(&self, value: &Val) -> usize {
        match value {
            Val::Arr(items) if !items.is_empty() => {
                self.count_container(container_key(value), items.len(), |i| &items[i])
            }
            Val::Obj(entries) if !entries.is_empty() => {
                self.count_container(container_key(value), entries.len(), |i| {
                    entries.get_index(i).map(|(_, v)| v).unwrap()
                })
            }
            _ => 1,
        }
    }

    fn count_container<'v>(
        &self,
        key: usize,
        len: usize,
        child: impl Fn(usize) -> &'v Val,
    ) -> usize {
        if let Some(prefix) = self.children_prefix.borrow().get(&key) {
            return 2 + prefix[len];
        }
        let children: usize = (0..len).map(|i| self.count(child(i))).sum();
        if children >= MEMO_MIN_LINES {
            let mut prefix = Vec::with_capacity(len + 1);
            prefix.push(0);
            let mut sum = 0;
            for i in 0..len {
                sum += self.count(child(i));
                prefix.push(sum);
            }
            self.children_prefix.borrow_mut().insert(key, prefix.into());
        }
        2 + children
    }

    fn children_prefix(&self, value: &Val) -> Option<Rc<[usize]>> {
        self.children_prefix
            .borrow()
            .get(&container_key(value))
            .cloned()
    }
}

fn container_key(value: &Val) -> usize {
    match value {
        Val::Arr(items) => Rc::as_ptr(items) as *const () as usize,
        Val::Obj(entries) => Rc::as_ptr(entries) as *const () as usize,
        _ => unreachable!("only containers are memoized"),
    }
}

struct Window<'c> {
    counter: &'c LineCounter,
    skip: usize,
    remaining: usize,
    lines: Vec<String>,
}

impl Window<'_> {
    fn emit(&mut self, line: impl FnOnce() -> String) {
        if self.skip > 0 {
            self.skip -= 1;
        } else if self.remaining > 0 {
            self.lines.push(line());
            self.remaining -= 1;
        }
    }

    fn value(&mut self, value: &Val, depth: usize, key: Option<&Rc<String>>, comma: bool) {
        if self.remaining == 0 {
            return;
        }
        let lines = self.counter.count(value);
        if lines <= self.skip {
            self.skip -= lines;
            return;
        }
        let comma = if comma { "," } else { "" };
        let head = || {
            format!(
                "{}{}",
                indent(depth),
                key.map(format_key).unwrap_or_default()
            )
        };
        match value {
            Val::Arr(items) if !items.is_empty() => {
                self.emit(|| format!("{}[", head()));
                let prefix = self.counter.children_prefix(value);
                self.children(items.len(), prefix.as_deref(), depth + 1, |i| {
                    (None, &items[i])
                });
                self.emit(|| format!("{}]{comma}", indent(depth)));
            }
            Val::Obj(entries) if !entries.is_empty() => {
                self.emit(|| format!("{}{{", head()));
                let prefix = self.counter.children_prefix(value);
                self.children(entries.len(), prefix.as_deref(), depth + 1, |i| {
                    entries.get_index(i).map(|(k, v)| (Some(k), v)).unwrap()
                });
                self.emit(|| format!("{}}}{comma}", indent(depth)));
            }
            scalar => self.emit(|| format!("{}{}{comma}", head(), format_scalar(scalar))),
        }
    }

    fn children<'v>(
        &mut self,
        len: usize,
        prefix: Option<&[usize]>,
        depth: usize,
        child: impl Fn(usize) -> (Option<&'v Rc<String>>, &'v Val),
    ) {
        let start = match prefix {
            Some(prefix) => {
                let first = prefix.partition_point(|&lines| lines <= self.skip) - 1;
                self.skip -= prefix[first];
                first
            }
            None => 0,
        };
        for i in start..len {
            if self.remaining == 0 {
                return;
            }
            let (key, value) = child(i);
            self.value(value, depth, key, i + 1 < len);
        }
    }
}

fn indent(depth: usize) -> String {
    "  ".repeat(depth)
}

fn format_key(key: &Rc<String>) -> String {
    format!("{}: ", quote(key))
}

// Scalars are formatted through serde_json rather than jaq's Display so the
// escaping (e.g. `\b`, `\f`) and float notation match `to_string_pretty`.
fn format_scalar(value: &Val) -> String {
    match value {
        Val::Null => "null".to_string(),
        Val::Bool(b) => b.to_string(),
        Val::Int(i) => i.to_string(),
        Val::Float(f) => serde_json::Number::from_f64(*f)
            .map(|n| n.to_string())
            .unwrap_or_else(|| "null".to_string()),
        Val::Num(literal) => literal.to_string(),
        Val::Str(s) => quote(s),
        Val::Arr(_) => "[]".to_string(),
        Val::Obj(_) => "{}".to_string(),
    }
}

fn quote(s: &str) -> String {
    serde_json::to_string(s).expect("serializing a str cannot fail")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::into_val;
    use serde_json::{Value, json};

    fn document_of(values: &[Value]) -> PrettyDocument {
        let mut doc = PrettyDocument::new();
        for v in values {
            doc.push(into_val(v.clone()));
        }
        doc
    }

    fn expected_lines(values: &[Value]) -> Vec<String> {
        let text = match values {
            [] => "null".to_string(),
            [single] => serde_json::to_string_pretty(single).unwrap(),
            many => serde_json::to_string_pretty(many).unwrap(),
        };
        text.lines().map(str::to_string).collect()
    }

    fn samples() -> Vec<(&'static str, Vec<Value>)> {
        vec![
            ("no outputs", vec![]),
            ("scalar null", vec![json!(null)]),
            ("scalar number", vec![json!(42)]),
            (
                "negative and float",
                vec![json!([-1, 1.5, 0.0, 1e300, -2.5e-7])],
            ),
            ("bool and string", vec![json!([true, false, "hello"])]),
            (
                "escaped string",
                vec![json!(
                    "quote\" back\\ nl\n tab\t ctrl\u{1} bs\u{8} ff\u{c} 日本語"
                )],
            ),
            (
                "empty containers",
                vec![json!({"a": [], "b": {}, "c": [[], {}]})],
            ),
            (
                "nested object",
                vec![
                    json!({"users": [{"name": "Alice", "tags": ["a", "b"]}, {"name": "Bob", "tags": []}], "count": 2}),
                ],
            ),
            ("escaped key", vec![json!({"k\"ey\n": 1})]),
            (
                "multiple outputs",
                vec![json!(1), json!({"a": [1, 2]}), json!("x")],
            ),
        ]
    }

    #[test]
    fn test_full_render_matches_serde_pretty() {
        for (name, values) in samples() {
            let doc = document_of(&values);
            let expected = expected_lines(&values);

            assert_eq!(expected.len(), doc.total_lines(), "{name}");
            assert_eq!(expected, doc.lines(0, usize::MAX), "{name}");
        }
    }

    #[test]
    fn test_window_render_matches_slice_of_full_render() {
        for (name, values) in samples() {
            let doc = document_of(&values);
            let expected = expected_lines(&values);

            for offset in 0..=expected.len() {
                for height in [0, 1, 3, 7] {
                    let end = (offset + height).min(expected.len());
                    assert_eq!(
                        expected[offset..end].to_vec(),
                        doc.lines(offset, height),
                        "{name} offset={offset} height={height}"
                    );
                }
            }
        }
    }

    #[test]
    fn test_large_containers_render_window_at_any_offset() {
        let items: Vec<Value> = (0..5000)
            .map(|i| json!({"id": i, "nested": {"list": [i, i + 1]}, "empty": []}))
            .collect();
        let values = vec![json!({"data": items, "tail": true})];
        let doc = document_of(&values);
        let expected = expected_lines(&values);

        assert_eq!(expected.len(), doc.total_lines());
        for offset in [0, 1, 2, 999, 12345, expected.len() - 5, expected.len() - 1] {
            assert_eq!(
                expected[offset..(offset + 5).min(expected.len())].to_vec(),
                doc.lines(offset, 5),
                "offset={offset}"
            );
        }
    }

    #[test]
    fn test_push_appends_outputs_incrementally() {
        let mut doc = PrettyDocument::new();
        let mut pushed = Vec::new();

        for i in 0..3000 {
            let value = json!({"i": i});
            doc.push(into_val(value.clone()));
            pushed.push(value);

            if i % 997 == 0 || i == 2999 {
                let expected = expected_lines(&pushed);
                assert_eq!(expected.len(), doc.total_lines(), "after {i}");
                let offset = expected.len() / 2;
                assert_eq!(
                    expected[offset..(offset + 4).min(expected.len())].to_vec(),
                    doc.lines(offset, 4),
                    "after {i}"
                );
            }
        }
        assert_eq!(3000, doc.len());
    }

    #[test]
    fn test_clear_resets_to_null() {
        let mut doc = document_of(&[json!([1, 2, 3])]);
        doc.clear();

        assert!(doc.is_empty());
        assert_eq!(vec!["null".to_string()], doc.lines(0, 10));
    }
}
