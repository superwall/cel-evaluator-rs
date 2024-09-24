#[cfg(not(target_arch = "wasm32"))]
uniffi::include_scaffolding!("cel");
mod ast;
mod models;

use crate::ast::{ASTExecutionContext, JSONExpression};
use crate::models::PassableValue::Function;
use crate::models::{ExecutionContext, PassableMap, PassableValue};
use crate::ExecutableType::{CompiledProgram, AST};
use async_trait::async_trait;
use cel_interpreter::extractors::This;
use cel_interpreter::objects::{Key, Map, TryIntoValue};
use cel_interpreter::{Context, ExecutionError, Expression, FunctionContext, Program, Value};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::fmt::Debug;
use std::ops::Deref;
use std::sync::{Arc, mpsc, Mutex};
use std::thread::spawn;
use cel_parser::parse;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::spawn_local;
#[cfg(not(target_arch = "wasm32"))]
use futures_lite::future::block_on;


/**
 * Host context trait that defines the methods that the host context should implement,
 * i.e. iOS or Android calling code. This trait is used to resolve dynamic properties in the
 * CEL expression during evaluation, such as `computed.daysSinceEvent("event_name")` or similar.
 * Note: Since WASM async support in the browser is still not fully mature, we're using the
 * target_arch cfg to define the trait methods differently for WASM and non-WASM targets.
 */
#[cfg(target_arch = "wasm32")]
pub trait HostContext: Send + Sync {
    fn computed_property(&self, name: String, args: String) -> String;

    fn device_property(&self, name: String, args: String) -> String;
}

#[cfg(not(target_arch = "wasm32"))]
#[async_trait]
pub trait HostContext: Send + Sync {
    async fn computed_property(&self, name: String, args: String) -> String;

    async fn device_property(&self, name: String, args: String) -> String;
}

/**
 * Evaluate a CEL expression with the given AST
 * @param ast The AST Execution Context, serialized as JSON. This defines the AST, the variables, and the platform properties.
 * @param host The host context to use for resolving properties
 * @return The result of the evaluation, either "true" or "false"
 */
pub fn evaluate_ast_with_context(definition: String, host: Arc<dyn HostContext>) -> String {
    let data: ASTExecutionContext = serde_json::from_str(definition.as_str()).expect("Invalid context definition for AST Execution");
    let host = host.clone();
    execute_with(
        AST(data.expression.into()),
        data.variables,
        data.computed,
        data.device,
        host,
    )
}

/**
 * Evaluate a CEL expression with the given AST without any context
 * @param ast The AST of the expression, serialized as JSON. This AST should contain already resolved dynamic variables.
 * @return The result of the evaluation, either "true" or "false"
 */
pub fn evaluate_ast(ast: String) -> String {
    let data: JSONExpression = serde_json::from_str(ast.as_str()).expect("Invalid definition for AST Execution");
    let ctx = Context::default();
    let res = ctx.resolve(&data.into()).unwrap();
    let res = DisplayableValue(res.clone());
    res.to_string()
}

/**
 * Evaluate a CEL expression with the given definition by compiling it first.
 * @param definition The definition of the expression, serialized as JSON. This defines the expression, the variables, and the platform properties.
 * @param host The host context to use for resolving properties
 * @return The result of the evaluation, either "true" or "false"
 */

pub fn evaluate_with_context(definition: String, host: Arc<dyn HostContext>) -> String {
    let data: Result<ExecutionContext, _> = serde_json::from_str(definition.as_str());
    let data = match data {
        Ok(data) => data,
        Err(e) => {
            panic!("Error: {}", e.to_string());
        }
    };
    let compiled = Program::compile(data.expression.as_str()).expect("Failed to compile expression");
    execute_with(
        CompiledProgram(compiled),
        data.variables,
        data.computed,
        data.device,
        host,
    )
}

/**
 * Transforms a given CEL expression into a CEL AST, serialized as JSON.
 * @param expression The CEL expression to parse
 * @return The AST of the expression, serialized as JSON
 */
pub fn parse_to_ast(expression: String) -> String {
    let ast : JSONExpression = parse(expression.as_str()).expect(
        format!("Failed to parse expression: {}", expression).as_str()
    ).into();
    serde_json::to_string(&ast).expect("Failed to serialize AST into JSON")
}

/**
Type of expression to be executed, either a compiled program or an AST.
 */
enum ExecutableType {
    AST(Expression),
    CompiledProgram(Program),
}

/**
 * Execute a CEL expression, either compiled or pure AST; with the given context.
 * @param executable The executable type, either an AST or a compiled program
 * @param variables The variables to use in the expression
 * @param platform The platform properties or functions to use in the expression
 * @param host The host context to use for resolving properties
 */
fn execute_with(
    executable: ExecutableType,
    variables: PassableMap,
    computed: Option<HashMap<String, Vec<PassableValue>>>,
    device: Option<HashMap<String, Vec<PassableValue>>>,
    host: Arc<dyn HostContext + 'static>,
) -> String {
    let host = host.clone();
    let host = Arc::new(Mutex::new(host));
    let mut ctx = Context::default();

    // Add predefined variables locally to the context
    variables
        .map
        .iter()
        .for_each(|it| ctx.add_variable(it.0.as_str(), it.1.to_cel())
            .expect(format!("Failed to add variable locally - {}", it.0).as_str()));
    // Add maybe function
    ctx.add_function("maybe", maybe);

    // This function is used to extract the value of a property from the host context
    // As UniFFi doesn't support recursive enums yet, we have to pass it in as a
    // JSON serialized string of a PassableValue from Host and deserialize it here

    enum PropType {
        Computed,
        Device,
    }
    #[cfg(not(target_arch = "wasm32"))]
    fn prop_for(
        prop_type: PropType,
        name: Arc<String>,
        args: Option<Vec<PassableValue>>,
        ctx: &Arc<dyn HostContext>,
    ) -> Option<PassableValue> {
        // Get computed property
        let val = futures_lite::future::block_on(async move {
            let ctx = ctx.clone();

            match prop_type {
                PropType::Computed => ctx.computed_property(
                    name.clone().to_string(),
                    serde_json::to_string(&args).expect("Failed to serialize args for computed property"),
                ).await,
                PropType::Device => ctx.device_property(
                    name.clone().to_string(),
                    serde_json::to_string(&args).expect("Failed to serialize args for computed property"),
                ).await,
            }
        });
        // Deserialize the value
        let passable: Option<PassableValue> = serde_json::from_str(val.as_str()).unwrap_or(Some(PassableValue::Null));

        passable
    }

    #[cfg(target_arch = "wasm32")]
    fn prop_for(
        prop_type: PropType,
        name: Arc<String>,
        args: Option<Vec<PassableValue>>,
        ctx: &Arc<dyn HostContext>,
    ) -> Option<PassableValue> {
        let ctx = ctx.clone();

        let val = match prop_type {
            PropType::Computed => ctx.computed_property(
                name.clone().to_string(),
                serde_json::to_string(&args).expect("Failed to serialize args for computed property"),
            ),
            PropType::Device => ctx.device_property(
                name.clone().to_string(),
                serde_json::to_string(&args).expect("Failed to serialize args for computed property"),
            ),
        };
        // Deserialize the value
        let passable: Option<PassableValue> = serde_json::from_str(val.as_str()).unwrap_or(Some(PassableValue::Null));

        passable
    }

    let computed = computed.unwrap_or(HashMap::new()).clone();

    // Create computed properties as a map of keys and function names
    let computed_host_properties: HashMap<Key, Value> = computed
        .iter()
        .map(|it| {
            let args = it.1.clone();
            let args = if args.is_empty() {
                None
            } else {
                Some(Box::new(PassableValue::List(args)))
            };
            let name = it.0.clone();
            (
                Key::String(Arc::new(name.clone())),
                Function(name, args).to_cel(),
            )
        })
        .collect();

    let device = device.unwrap_or(HashMap::new()).clone();

    // Create device properties as a map of keys and function names
    let device_host_properties: HashMap<Key, Value> = device
        .iter()
        .map(|it| {
            let args = it.1.clone();
            let args = if args.is_empty() {
                None
            } else {
                Some(Box::new(PassableValue::List(args)))
            };
            let name = it.0.clone();
            (
                Key::String(Arc::new(name.clone())),
                Function(name, args).to_cel(),
            )
        })
        .collect();


    // Add the map to the `computed` object
    ctx.add_variable(
        "computed",
        Value::Map(Map {
            map: Arc::new(computed_host_properties),
        }),
    )
        .unwrap();

    let binding = device.clone();
    // Combine the device and computed properties
    let host_properties = binding
        .iter()
        .chain(computed.iter())
        .map(|(k, v)| (k.clone(), v.clone()))
        .into_iter();

    let mut device_properties_clone = device.clone().clone();
    // Add those functions to the context
    for it in host_properties {
        let mut value = device_properties_clone.clone();
        let key = it.0.clone();
        let host_clone = Arc::clone(&host); // Clone the Arc to pass into the closure
        let key_str = key.clone(); // Clone key for usage in the closure
        ctx.add_function(
            key_str.as_str(),
            move |ftx: &FunctionContext| -> Result<Value, ExecutionError> {
                let device = value.clone();
                let fx = ftx.clone();
                let name = fx.name.clone(); // Move the name into the closure
                let args = fx.args.clone(); // Clone the arguments
                let host = host_clone.lock().unwrap(); // Lock the host for safe access
                prop_for(
                    if device.contains_key(&it.0)
                    { PropType::Device } else { PropType::Computed },
                    name.clone(),
                    Some(
                        args.iter()
                            .map(|expression| {
                                DisplayableValue(ftx.ptx.resolve(expression).unwrap()).to_passable()
                            })
                            .collect(),
                    ),
                    &*host,
                )
                    .map_or(Err(ExecutionError::UndeclaredReference(name)), |v| {
                        Ok(v.to_cel())
                    })
            },
        );
    }

    let val = match executable {
        AST(ast) => &ctx.resolve(&ast),
        CompiledProgram(program) => &program.execute(&ctx),
    };

    match val {
        Ok(val) => {
            let val = DisplayableValue(val.clone());
            val.to_string()
        }
        Err(err) => {
            let val = DisplayableError(err.clone());
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

// Wrappers around CEL values used so that we can create extensions on them
pub struct DisplayableValue(Value);

pub struct DisplayableError(ExecutionError);

impl fmt::Display for DisplayableValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.0 {
            Value::Int(i) => write!(f, "{}", i),
            Value::Float(x) => write!(f, "{}", x),
            Value::String(s) => write!(f, "{}", s),
            // Add more variants as needed
            Value::UInt(i) => write!(f, "{}", i),
            Value::Bytes(_) => {
                write!(f, "{}", "bytes go here")
            }
            Value::Bool(b) => write!(f, "{}", b),
            Value::Duration(d) => write!(f, "{}", d),
            Value::Timestamp(t) => write!(f, "{}", t),
            Value::Null => write!(f, "{}", "null"),
            Value::Function(name, _) => write!(f, "{}", name),
            Value::Map(map) => {
                let res: HashMap<String, String> = map
                    .map
                    .iter()
                    .map(|(k, v)| {
                        let key = DisplayableValue(k.try_into_value().unwrap().clone()).to_string();
                        let value = DisplayableValue(v.clone()).to_string().replace("\\", "");
                        (key, value)
                    })
                    .collect();
                let map = serde_json::to_string(&res).unwrap();
                write!(f, "{}", map)
            }
            Value::List(list) => write!(
                f,
                "{}",
                list.iter()
                    .map(|v| {
                        let key = DisplayableValue(v.clone());
                        return key.to_string();
                    })
                    .collect::<Vec<_>>()
                    .join(",\n ")
            ),
        }
    }
}

impl fmt::Display for DisplayableError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0.to_string().as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestContext {
        map: HashMap<String, String>,
    }

    #[async_trait]
    impl HostContext for TestContext {
        async fn computed_property(&self, name: String, args: String) -> String {
            self.map.get(&name).unwrap().to_string()
        }

        async fn device_property(&self, name: String, args: String) -> String {
            self.map.get(&name).unwrap().to_string()
        }
    }

    #[tokio::test]
    async fn test_variables() {
        let ctx = Arc::new(TestContext {
            map: HashMap::new(),
        });
        let res = evaluate_with_context(
            r#"
        {
            "variables": {
             "map" : {
                    "foo": {"type": "int", "value": 100}
            }},
            "expression": "foo == 100"
        }

        "#
                .to_string(),
            ctx,
        );
        assert_eq!(res, "true");
    }

    #[tokio::test]
    async fn test_execution_with_ctx() {
        let ctx = Arc::new(TestContext {
            map: HashMap::new(),
        });
        let res = evaluate_with_context(
            r#"
        {
            "variables": {
             "map" : {
                    "foo": {"type": "int", "value": 100},
                    "bar": {"type": "int", "value": 42}
            }},
            "expression": "foo + bar == 142"
        }

        "#
                .to_string(),
            ctx,
        );
        assert_eq!(res, "true");
    }

    #[test]
    fn test_unknown_function_with_arg_fails_with_undeclared_ref() {
        let ctx = Arc::new(TestContext {
            map: HashMap::new(),
        });

        let res = evaluate_with_context(
            r#"
        {
            "variables": {
             "map" : {
                    "foo": {"type": "int", "value": 100}
            }},
            "expression": "test_custom_func(foo) == 101"
        }

        "#
                .to_string(),
            ctx,
        );
        assert_eq!(res, "Undeclared reference to 'test_custom_func'");
    }

    #[test]
    fn test_list_contains() {
        let ctx = Arc::new(TestContext {
            map: HashMap::new(),
        });
        let res = evaluate_with_context(
            r#"
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

        "#
                .to_string(),
            ctx,
        );
        assert_eq!(res, "true");
    }

    #[tokio::test]
    async fn test_execution_with_map() {
        let ctx = Arc::new(TestContext {
            map: HashMap::new(),
        });
        let res = evaluate_with_context(
            r#"
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

        "#
                .to_string(),
            ctx,
        );
        println!("{}", res);
        assert_eq!(res, "true");
    }

    #[tokio::test]
    async fn test_execution_with_platform_reference() {
        let days_since = PassableValue::UInt(7);
        let days_since = serde_json::to_string(&days_since).unwrap();
        let ctx = Arc::new(TestContext {
            map: [("daysSinceEvent".to_string(), days_since)]
                .iter()
                .cloned()
                .collect(),
        });
        let res = evaluate_with_context(
            r#"
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
                    "computed" : {
                      "daysSinceEvent": [{
                                        "type": "string",
                                        "value": "event_name"
                                    }]
                    },
                    "device" : {
                      "timeSinceEvent": [{
                                        "type": "string",
                                        "value": "event_name"
                                    }]
                    },
                    "expression": "computed.daysSinceEvent(\"test\") == user.some_value"
        }
        "#
                .to_string(),
            ctx,
        );
        println!("{}", res);
        assert_eq!(res, "true");
    }


    #[test]
    fn test_parse_to_ast() {
        let expression = "device.daysSince(app_install) == 3";
        let ast_json = parse_to_ast(expression.to_string());
        println!("\nSerialized AST:");
        println!("{}", ast_json);
        // Deserialize back to JSONExpression
        let deserialized_json_expr: JSONExpression = serde_json::from_str(&ast_json).unwrap();

        // Convert back to original Expression
        let deserialized_expr: Expression = deserialized_json_expr.into();

        println!("\nDeserialized Expression:");
        println!("{:?}", deserialized_expr);

        let parsed_expression = parse(expression).unwrap();
        assert_eq!(parsed_expression, deserialized_expr);
        println!("\nOriginal and deserialized expressions are equal!");
    }
}
