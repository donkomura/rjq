use jaq_json::Val;
use serde::de::{self, Deserialize, DeserializeSeed, Deserializer, MapAccess, SeqAccess, Visitor};
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

/// One top-level JSON value deserialized directly into `Val`.
///
/// Object keys are interned per top-level value: inputs such as large arrays of
/// records repeat the same few keys millions of times, and sharing one `Rc` per
/// distinct key removes one allocation per occurrence.
pub struct ParsedVal(pub Val);

impl<'de> Deserialize<'de> for ParsedVal {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let mut keys = KeyInterner::default();
        ValSeed { keys: &mut keys }
            .deserialize(deserializer)
            .map(ParsedVal)
    }
}

#[derive(Default)]
struct KeyInterner {
    keys: HashMap<String, Rc<String>>,
}

impl KeyInterner {
    fn intern(&mut self, key: &str) -> Rc<String> {
        if let Some(shared) = self.keys.get(key) {
            return shared.clone();
        }
        let shared = Rc::new(key.to_string());
        self.keys.insert(key.to_string(), shared.clone());
        shared
    }
}

struct ValSeed<'k> {
    keys: &'k mut KeyInterner,
}

impl<'de> DeserializeSeed<'de> for ValSeed<'_> {
    type Value = Val;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Val, D::Error> {
        deserializer.deserialize_any(self)
    }
}

impl<'de> Visitor<'de> for ValSeed<'_> {
    type Value = Val;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("a JSON value")
    }

    fn visit_unit<E: de::Error>(self) -> Result<Val, E> {
        Ok(Val::Null)
    }

    fn visit_bool<E: de::Error>(self, b: bool) -> Result<Val, E> {
        Ok(Val::Bool(b))
    }

    fn visit_i64<E: de::Error>(self, i: i64) -> Result<Val, E> {
        Ok(isize::try_from(i).map_or_else(|_| Val::Num(Rc::new(i.to_string())), Val::Int))
    }

    fn visit_u64<E: de::Error>(self, u: u64) -> Result<Val, E> {
        Ok(isize::try_from(u).map_or_else(|_| Val::Num(Rc::new(u.to_string())), Val::Int))
    }

    fn visit_f64<E: de::Error>(self, f: f64) -> Result<Val, E> {
        Ok(Val::Float(f))
    }

    fn visit_str<E: de::Error>(self, s: &str) -> Result<Val, E> {
        Ok(Val::Str(Rc::new(s.to_string())))
    }

    fn visit_string<E: de::Error>(self, s: String) -> Result<Val, E> {
        Ok(Val::Str(Rc::new(s)))
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Val, A::Error> {
        let mut items = Vec::with_capacity(seq.size_hint().unwrap_or(0));
        while let Some(item) = seq.next_element_seed(ValSeed { keys: self.keys })? {
            items.push(item);
        }
        Ok(Val::Arr(Rc::new(items)))
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Val, A::Error> {
        let mut entries = Vec::with_capacity(map.size_hint().unwrap_or(0));
        while let Some(key) = map.next_key_seed(KeySeed { keys: self.keys })? {
            let value = map.next_value_seed(ValSeed { keys: self.keys })?;
            entries.push((key, value));
        }
        Ok(Val::obj(entries.into_iter().collect()))
    }
}

struct KeySeed<'k> {
    keys: &'k mut KeyInterner,
}

impl<'de> DeserializeSeed<'de> for KeySeed<'_> {
    type Value = Rc<String>;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Rc<String>, D::Error> {
        deserializer.deserialize_str(self)
    }
}

impl<'de> Visitor<'de> for KeySeed<'_> {
    type Value = Rc<String>;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("an object key")
    }

    fn visit_str<E: de::Error>(self, key: &str) -> Result<Rc<String>, E> {
        Ok(self.keys.intern(key))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::into_val;
    use serde_json::json;
    use std::rc::Rc;

    fn parse(input: &str) -> Val {
        serde_json::from_str::<ParsedVal>(input).unwrap().0
    }

    #[test]
    fn test_parse_matches_serde_conversion() {
        let tests = vec![
            ("null", "null", json!(null)),
            ("bool", "true", json!(true)),
            ("integer", "-42", json!(-42)),
            ("float", "1.5", json!(1.5)),
            ("string", r#""a\"b\n""#, json!("a\"b\n")),
            ("array", r#"[1, [2, []], {}]"#, json!([1, [2, []], {}])),
            (
                "object",
                r#"{"a": {"b": [true]}}"#,
                json!({"a": {"b": [true]}}),
            ),
        ];

        for (name, input, expected) in tests {
            assert_eq!(into_val(expected), parse(input), "{name}");
        }
    }

    #[test]
    fn test_parse_keeps_number_kinds() {
        let tests = vec![
            ("integer", "7", Val::Int(7)),
            ("float with zero fraction", "2.0", Val::Float(2.0)),
            (
                "u64 beyond isize",
                "18446744073709551615",
                Val::Num(Rc::new(u64::MAX.to_string())),
            ),
        ];

        for (name, input, expected) in tests {
            let actual = parse(input);
            assert_eq!(expected, actual, "{name}");
            assert_eq!(
                std::mem::discriminant(&expected),
                std::mem::discriminant(&actual),
                "{name}"
            );
        }
    }

    #[test]
    fn test_parse_preserves_key_order() {
        let Val::Obj(entries) = parse(r#"{"b": 1, "a": 2, "c": 3}"#) else {
            panic!("expected object");
        };

        let keys: Vec<&str> = entries.keys().map(|k| k.as_str()).collect();
        assert_eq!(vec!["b", "a", "c"], keys);
    }

    #[test]
    fn test_parse_keeps_last_duplicate_key() {
        assert_eq!(into_val(json!({"a": 2})), parse(r#"{"a": 1, "a": 2}"#));
    }

    #[test]
    fn test_parse_shares_repeated_keys() {
        let Val::Arr(items) = parse(r#"[{"name": 1}, {"name": 2}]"#) else {
            panic!("expected array");
        };
        let key_of = |v: &Val| match v {
            Val::Obj(entries) => entries.keys().next().unwrap().clone(),
            _ => panic!("expected object"),
        };

        assert!(Rc::ptr_eq(&key_of(&items[0]), &key_of(&items[1])));
    }
}
