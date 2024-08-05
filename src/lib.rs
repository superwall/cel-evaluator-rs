uniffi::include_scaffolding!("cel");
mod models;
use cel_interpreter::{Context, ExecutionError, Program, Value};
use std::fmt;
use ::serde::{Deserialize, Serialize};
use crate::models::{ExecutionContext};


pub fn evaluate_with_context(definition: String) -> String {
    let data: ExecutionContext = serde_json::from_str(definition.as_str()).unwrap();
    let compiled = Program::compile(data.expression.as_str()).unwrap();
    let mut ctx = Context::default();
    data.variables.map.iter().for_each(|it| {
        ctx.add_variable(it.0.as_str(), it.1.to_cel()).unwrap()
    });

    ctx.add_function("test_custom_func", |arg: i64| {
        return arg + 1;
    });
    let val = compiled.execute(&ctx);
    match val {
        Ok(val) => {
            let val = DisplayableValue(val);
            println!("{}", val.to_string());
            val.to_string()
        }
        Err(err) => {
            let val = DisplayableError(err);
            println!("{}", val.to_string());
            val.to_string()
        }
    }
}


// Wrappers around CEL values so we can create extensions on them
pub struct DisplayableValue(cel_interpreter::Value);

pub struct DisplayableError(cel_interpreter::ExecutionError);

impl fmt::Display for DisplayableValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Assuming Value is an enum with variants Integer, Float, and Str
        match &self.0 {
            Value::Int(i) => write!(f, "{}", i),
            Value::Float(x) => write!(f, "{}", x),
            Value::String(s) => write!(f, "{}", s),
            // Add more variants as needed
            Value::UInt(i) => write!(f, "{}", i),
            Value::Bytes(b) => { write!(f, "{}", "bytes go here") }
            Value::Bool(b) => write!(f, "{}", b),
            Value::Duration(d) => write!(f, "{}", d),
            Value::Timestamp(t) => write!(f, "{}", t),
            Value::Null => write!(f, "{}", "null"),
            _ => write!(f, "{}", "Collection or something else")
        }
    }
}

impl fmt::Display for DisplayableError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Assuming Value is an enum with variants Integer, Float, and Str
        match &self.0 {
            ExecutionError::InvalidArgumentCount { .. } => write!(f, "InvalidArgumentCount"),
            ExecutionError::UnsupportedTargetType { .. } => write!(f, "UnsupportedTargetType"),
            ExecutionError::NotSupportedAsMethod { .. } => write!(f, "NotSupportedAsMethod"),
            ExecutionError::UnsupportedKeyType(_) => write!(f, "UnsupportedKeyType"),
            ExecutionError::UnexpectedType { .. } => write!(f, "UnexpectedType"),
            ExecutionError::NoSuchKey(_) => write!(f, "NoSuchKey"),
            ExecutionError::UndeclaredReference(_) => write!(f, "UndeclaredReference"),
            ExecutionError::MissingArgumentOrTarget => write!(f, "MissingArgumentOrTarget"),
            ExecutionError::ValuesNotComparable(_, _) => write!(f, "ValuesNotComparable"),
            ExecutionError::UnsupportedUnaryOperator(_, _) => write!(f, "UnsupportedUnaryOperator"),
            ExecutionError::UnsupportedBinaryOperator(_, _, _) => write!(f, "UnsupportedBinaryOperator"),
            ExecutionError::UnsupportedMapIndex(_) => write!(f, "UnsupportedMapIndex"),
            ExecutionError::UnsupportedListIndex(_) => write!(f, "UnsupportedListIndex"),
            ExecutionError::UnsupportedIndex(_, _) => write!(f, "UnsupportedIndex"),
            ExecutionError::UnsupportedFunctionCallIdentifierType(_) => write!(f, "UnsupportedFunctionCallIdentifierType"),
            ExecutionError::UnsupportedFieldsConstruction(_) => write!(f, "UnsupportedFieldsConstruction"),
            ExecutionError::FunctionError { .. } => write!(f, "FunctionError"),
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::models::PassableValue;
    use crate::models::PassableValue::{Function, Int};
    use super::*;

    #[test]
    fn test_variables() {
        let definition = "foo + bar".to_string();
        let res = evaluate_with_context(r#"
        {
            "variables": {
             "map" : {
                    "foo": {"type": "int", "value": 100}
            }},
            "expression": "foo == 100"
        }

        "#.to_string());
        assert_eq!(res, "true");
    }

    #[test]
    fn test_execution_with_ctx() {
        let definition = "foo + bar".to_string();
        let res = evaluate_with_context(r#"
        {
            "variables": {
             "map" : {
                    "foo": {"type": "int", "value": 100},
                    "bar": {"type": "int", "value": 42}
            }},
            "expression": "foo + bar == 142"
        }

        "#.to_string());
        assert_eq!(res, "true");
    }

    #[test]
    fn test_custom_function_with_arg() {
        let definition = "foo + bar".to_string();
        let res = evaluate_with_context(r#"
        {
            "variables": {
             "map" : {
                    "foo": {"type": "int", "value": 100}
            }},
            "expression": "test_custom_func(foo) == 101"
        }

        "#.to_string());
        assert_eq!(res, "true");
    }

    #[test]
    fn test_list_contains() {
        let definition = "foo + bar".to_string();
        let res = evaluate_with_context(r#"
        {
            "variables": {
                 "map" : {
                    "numbers": {
                        "type" : "list",
                        "value" : [
                            {"type": "int", "value": 1},
                            {"type": "int", "value": 2},
                            {"type": "int", "value": 3}
                             ]
                       }
                 }
            },
            "expression": "numbers.contains(2)"
        }

        "#.to_string());
        assert_eq!(res, "true");
    }
}
