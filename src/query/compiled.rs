use crate::app::error::AppError;
use jaq_core::{
    Ctx, Filter, Native, RcIter,
    load::{Arena, File, Loader},
};
use jaq_json::Val;

pub struct CompiledQuery {
    filter: Filter<Native<Val>>,
}

impl CompiledQuery {
    pub fn compile(query: &str) -> Result<Self, AppError> {
        let program = File {
            code: query,
            path: (),
        };
        let loader = Loader::new(jaq_std::defs().chain(jaq_json::defs()));
        let arena = Arena::default();
        let modules = loader
            .load(&arena, program)
            .map_err(|e| AppError::QueryCompile(format!("Loader: {:?}", e)))?;
        let filter = jaq_core::Compiler::default()
            .with_funs(jaq_std::funs().chain(jaq_json::funs()))
            .compile(modules)
            .map_err(|e| AppError::QueryCompile(format!("Compiler: {:?}", e)))?;
        Ok(Self { filter })
    }

    pub fn run(&self, input: Val) -> Vec<Val> {
        let inputs = RcIter::new(core::iter::empty());
        self.filter
            .run((Ctx::new([], &inputs), input))
            .filter_map(Result::ok)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::into_val;
    use serde_json::{Value, json};

    fn run(query: &str, input: Value) -> Vec<Value> {
        CompiledQuery::compile(query)
            .unwrap()
            .run(into_val(input))
            .into_iter()
            .map(Value::from)
            .collect()
    }

    #[test]
    fn test_run_compiled_query() {
        let tests = vec![
            ("identity", ".", json!({"a": 1}), vec![json!({"a": 1})]),
            ("field", ".a", json!({"a": [1, 2]}), vec![json!([1, 2])]),
            (
                "multiple outputs",
                ".[]",
                json!([1, 2]),
                vec![json!(1), json!(2)],
            ),
            ("no outputs", "empty", json!(1), vec![]),
            ("runtime error is dropped", ".a", json!([1]), vec![]),
        ];

        for (name, query, input, expected) in tests {
            assert_eq!(expected, run(query, input), "{name}");
        }
    }

    #[test]
    fn test_compiled_query_is_reusable_across_inputs() {
        let query = CompiledQuery::compile(".x").unwrap();

        let first: Vec<Value> = query
            .run(into_val(json!({"x": 1})))
            .into_iter()
            .map(Value::from)
            .collect();
        let second: Vec<Value> = query
            .run(into_val(json!({"x": 2})))
            .into_iter()
            .map(Value::from)
            .collect();

        assert_eq!(vec![json!(1)], first);
        assert_eq!(vec![json!(2)], second);
    }

    #[test]
    fn test_compile_error() {
        let tests = vec![("syntax error", ".["), ("undefined function", "nosuchfn")];

        for (name, query) in tests {
            assert!(
                matches!(
                    CompiledQuery::compile(query),
                    Err(AppError::QueryCompile(_))
                ),
                "{name}"
            );
        }
    }
}
