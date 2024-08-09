uniffi::include_scaffolding!("cel");
mod models;

use std::collections::HashMap;
use cel_interpreter::{Context, ExecutionError, Expression, FunctionContext, Program, Value};
use std::fmt;
use std::sync::{Arc, Mutex};
use cel_interpreter::extractors::This;
use cel_interpreter::objects::{Key, Map, TryIntoValue};
use crate::models::{ExecutionContext, PassableValue};
use crate::models::PassableValue::{Function};


pub trait HostContext: Send + Sync {
    fn computed_property(&self, name: String) -> String;
}


pub fn evaluate_with_context(definition: String, host: Box<dyn HostContext + 'static>) -> String {
    let data: ExecutionContext = serde_json::from_str(definition.as_str()).unwrap();
    let compiled = Program::compile(data.expression.as_str()).unwrap();
    let host = Arc::new(Mutex::new(host));
    let mut ctx = Context::default();

    // Add predefined variables locally to the context
    data.variables.map.iter().for_each(|it| {
        ctx.add_variable(it.0.as_str(), it.1.to_cel()).unwrap()
    });
    // Add maybe function
    ctx.add_function("maybe", maybe);

    // This function is used to extract the value of a property from the host context
    // As UniFFi doesn't support recursive enums yet, we have to pass it in as a
    // JSON serialized string of a PassableValue from Host and deserialize it here
    fn prop_for(name: Arc<String>,
                args: Option<Vec<PassableValue>>, ctx: &Box<dyn HostContext>) -> Option<PassableValue> {
        // Get computed property
        let val = ctx.computed_property(name.clone().to_string());
        println!("{}", val);
        // Deserialize the value
        let passable: Option<PassableValue> = serde_json::from_str(val.as_str())
            .unwrap_or(None);

        passable
    }



    let platform = data.platform;
    let platform = platform.unwrap().clone();

    // Create platform properties as a map of keys and function names
    let platform_properties: HashMap<Key, Value> = platform.iter().map(|it| {
        (Key::String(Arc::new(it.0.clone())), Function(it.1.clone(), None).to_cel())
    }).collect();

    // Add the map to the platform object
    ctx.add_variable("platform", Value::Map(Map { map: Arc::new(platform_properties) })).unwrap();

    // Add those functions to the context
    for it in platform.iter() {

        let key = it.0.clone();
        let host_clone = Arc::clone(&host); // Clone the Arc to pass into the closure
        let key_str = key.clone(); // Clone key for usage in the closure
        ctx.add_function(key_str.as_str(), move |ftx: &FunctionContext| -> Result<Value, ExecutionError> {
            let fx = ftx.clone();
            let name = fx.name.clone(); // Move the name into the closure
            let args = fx.args.clone(); // Clone the arguments
            let host = host_clone.lock().unwrap(); // Lock the host for safe access
            prop_for(name.clone(), Some(args.iter().map(|expression|
                DisplayableValue(ftx.ptx.resolve(expression).unwrap()).to_passable()).collect()), &*host)
                .map_or(Err(ExecutionError::UndeclaredReference(name)), |v| Ok(v.to_cel()))
        });
    }


    let val = compiled.execute(&ctx);
    match val {
        Ok(val) => {
            let val = DisplayableValue(val);
            val.to_string()
        }
        Err(err) => {
            let val = DisplayableError(err);
            val.to_string()
        }
    }
}


pub fn maybe(
    ftx: &FunctionContext,
    This(_this): This<Value>,
    left: Expression,
    right: Expression,
) -> Result<Value, ExecutionError> {
    return ftx.ptx.resolve(&left).or_else(|_| ftx.ptx.resolve(&right));
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
            Value::Bytes(_) => { write!(f, "{}", "bytes go here") }
            Value::Bool(b) => write!(f, "{}", b),
            Value::Duration(d) => write!(f, "{}", d),
            Value::Timestamp(t) => write!(f, "{}", t),
            Value::Null => write!(f, "{}", "null"),
            Value::Function(name, _) => write!(f, "{}", name),
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
                return key.to_string();
            }).collect::<Vec<_>>().join(",\n ")),
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

    struct TestContext {
        map: HashMap<String, String>
    }

    impl HostContext for TestContext {
        fn computed_property(&self, name: String) -> String {
            self.map.get(&name).unwrap().to_string()
        }

    }

    #[test]
    fn test_variables() {
        let ctx = Box::new(TestContext {
            map: HashMap::new()
        });
        let res = evaluate_with_context(r#"
        {
            "variables": {
             "map" : {
                    "foo": {"type": "int", "value": 100}
            }},
            "expression": "foo == 100"
        }

        "#.to_string(), ctx);
        assert_eq!(res, "true");
    }

    #[test]
    fn test_execution_with_ctx() {
        let ctx = Box::new(TestContext {
            map: HashMap::new()
        });
        let res = evaluate_with_context(r#"
        {
            "variables": {
             "map" : {
                    "foo": {"type": "int", "value": 100},
                    "bar": {"type": "int", "value": 42}
            }},
            "expression": "foo + bar == 142"
        }

        "#.to_string(), ctx);
        assert_eq!(res, "true");
    }

    #[test]
    fn test_custom_function_with_arg() {
        let ctx = Box::new(TestContext {
            map: HashMap::new()
        });

        let res = evaluate_with_context(r#"
        {
            "variables": {
             "map" : {
                    "foo": {"type": "int", "value": 100}
            }},
            "expression": "test_custom_func(foo) == 101"
        }

        "#.to_string(),ctx);
        assert_eq!(res, "true");
    }

    #[test]
    fn test_list_contains() {
        let ctx = Box::new(TestContext {
            map: HashMap::new()
        });
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

        "#.to_string(),ctx);
        assert_eq!(res, "true");
    }

    #[test]
    fn test_execution_with_map() {
        let ctx = Box::new(TestContext {
            map: HashMap::new()
        });
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

        "#.to_string(), ctx);
        println!("{}", res);
        assert_eq!(res, "true");
    }

    #[test]
    fn test_execution_with_platform_reference() {
        let days_since = PassableValue::UInt(7);
        let days_since = serde_json::to_string(&days_since).unwrap();
        let ctx = Box::new(TestContext {
            map: [("daysSinceEvent".to_string(), days_since)].iter().cloned().collect()
        });
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
        "#.to_string(), ctx);
        println!("{}", res);
        assert_eq!(res, "true");
    }
}
