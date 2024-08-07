uniffi::include_scaffolding!("cel");
mod models;

use std::collections::HashMap;
use cel_interpreter::{Context, ExecutionError, Expression, FunctionContext, Program, ResolveResult, Value};
use std::fmt;
use std::hash::Hash;
use std::sync::Arc;
use cel_interpreter::extractors::This;
use cel_interpreter::objects::{Key, Map, TryIntoValue};
use crate::models::{ExecutionContext, PassableValue};
use crate::models::PassableValue::{Function};


trait Caller<S, V> {
    fn for_name(&self, name: S) -> Option<V>
        where
            S: Into<String>,
            V: TryIntoValue;
}

struct ComputedProperty<S, V> where
    S: Into<String>,
    V: TryIntoValue {
    caller: dyn Caller<S, V>,
}


pub fn evaluate_with_context(definition: String) -> String {
    let data: ExecutionContext = serde_json::from_str(definition.as_str()).unwrap();
    let compiled = Program::compile(data.expression.as_str()).unwrap();
    let mut ctx = Context::default();


    data.variables.map.iter().for_each(|it| {
        ctx.add_variable(it.0.as_str(), it.1.to_cel()).unwrap()
    });

    fn prop_for(name: String) -> Option<PassableValue> {
        // Add callback interface function here
        match name.as_str() {
            "daysSinceEvent" => Some(PassableValue::Int(7)),
            "pest" => Some(PassableValue::String("name".to_string())),
            _ => None
        }
    }

    let platform = data.platform;
    let platform = platform.unwrap().clone();
    let clone = platform.clone();
    let platform_properties: HashMap<Key, Value> = clone.iter().map(|it| {
        (Key::String(Arc::new(it.0.clone())), Function(it.1.clone(), None).to_cel())
    }).collect();
    println!("Platform properties: {:?}", platform_properties);
    ctx.add_variable("platform", Value::Map(Map { map: Arc::new(platform_properties) })).unwrap();

    ctx.add_function("maybe", maybe);
    for it in clone.iter() {
        let key = it.0.clone();
        let value = it.1.clone();
        ctx.add_function(key.clone().as_str(), move |ftx: &FunctionContext| {
            let x = prop_for(key.to_string()).unwrap();
            Ok(x.to_cel())
        });
    }

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


pub fn maybe(
    ftx: &FunctionContext,
    This(this): This<Value>,
    left: Expression,
    right: Expression
) -> Result<Value, ExecutionError> {
    return ftx.ptx.resolve(&left).or_else(|_| ftx.ptx.resolve(&right))
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
            Value::Function(name, arg) => write!(f, "{}", name),
            Value::Map(map) => {
                let res: HashMap<String, String> = map.map.iter().map(|(k, v)| {
                    let key = DisplayableValue(k.try_into_value().unwrap().clone()).to_string();
                    let value = DisplayableValue(v.clone()).to_string().replace("\\", "");
                    (key, value)
                }).collect();
                let map = serde_json::to_string(&res).unwrap();
                write!(f, "{}", map)
            }
            Value::List(list) => write!(f, "{}", list.iter().map(|v| {
                let key = DisplayableValue(v.clone());
                return format!("({})", key);
            }).collect::<Vec<_>>().join(",\n ")),

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

    #[test]
    fn test_execution_with_map() {
        let definition = "foo + bar".to_string();
        let res = evaluate_with_context(r#"
        {
                    "variables": {
                        "map": {
                            "user": {
                                "type": "map",
                                "value": {
                                    "should_display": {
                                        "type": "bool",
                                        "value": true
                                    },
                                    "some_value": {
                                        "type": "uint",
                                        "value": 13
                                    }
                                }
                            }
                        }
                    },
                    "expression": "user.should_display == true && user.some_value > 12"
    }

        "#.to_string());
        println!("{}", res);
        assert_eq!(res, "true");
    }

    #[test]
    fn test_execution_with_platform_reference() {
        let definition = "foo + bar".to_string();
        let res = evaluate_with_context(r#"
        {
                    "variables": {
                        "map": {
                            "user": {
                                "type": "map",
                                "value": {
                                    "should_display": {
                                        "type": "bool",
                                        "value": true
                                    },
                                    "some_value": {
                                        "type": "uint",
                                        "value": 7
                                    }
                                }
                            }
                        }
                    },
                    "platform" : {
                      "daysSinceEvent": "0"
                    },
                    "expression": "platform.daysSinceEvent() == user.some_value"
    }

        "#.to_string());
        println!("{}", res);
        assert_eq!(res, "true");
    }
}
