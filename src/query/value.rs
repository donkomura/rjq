use jaq_json::Val;
use serde_json::{Number, Value};
use std::rc::Rc;

// jaq_json's own `From<serde_json::Value>` stores every number that does not fit
// in isize (including all floats) as a heap-allocated literal string (`Val::Num`).
// Converting floats to `Val::Float` here avoids one allocation per float.
pub fn into_val(value: Value) -> Val {
    match value {
        Value::Null => Val::Null,
        Value::Bool(b) => Val::Bool(b),
        Value::Number(n) => number_into_val(n),
        Value::String(s) => Val::Str(Rc::new(s)),
        Value::Array(items) => Val::Arr(Rc::new(items.into_iter().map(into_val).collect())),
        Value::Object(entries) => Val::obj(
            entries
                .into_iter()
                .map(|(k, v)| (Rc::new(k), into_val(v)))
                .collect(),
        ),
    }
}

fn number_into_val(n: Number) -> Val {
    if let Some(i) = n.as_i64().and_then(|i| isize::try_from(i).ok()) {
        return Val::Int(i);
    }
    if n.is_f64()
        && let Some(f) = n.as_f64()
    {
        return Val::Float(f);
    }
    Val::Num(Rc::new(n.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_into_val_keeps_number_kinds() {
        let tests = vec![
            ("integer", json!(42), Val::Int(42)),
            ("negative integer", json!(-7), Val::Int(-7)),
            ("float", json!(1.5), Val::Float(1.5)),
            ("float with zero fraction", json!(2.0), Val::Float(2.0)),
        ];

        for (name, input, expected) in tests {
            assert_eq!(expected, into_val(input), "{name}");
        }
    }

    #[test]
    fn test_into_val_keeps_u64_beyond_isize_as_literal() {
        let actual = into_val(json!(u64::MAX));

        assert_eq!(Val::Num(std::rc::Rc::new(u64::MAX.to_string())), actual);
    }

    #[test]
    fn test_into_val_round_trips_structure() {
        let input = json!({"b": [1, "two", null, true, {"c": 3.25}], "a": {}});

        let actual: serde_json::Value = into_val(input.clone()).into();

        assert_eq!(input, actual);
    }
}
