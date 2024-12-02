## Superscript runtime

This is the Superscript runtime library.
It is a standalone library that can be used to evaluate Superscript expressions in Mobile applications by using
`cel-rust` crate with addons for dynamic code execution and WASM.

## Installation

To build the library, you'll need to install:

1. Rust (https://www.rust-lang.org/tools/install)
2. Docker (for cross) (https://docs.docker.com/get-docker/)
3. cross (https://github.com/cross-rs/cross)

## Building

To build the library, run:

```shell
./build_android.sh
```

(note: for the first run you will need to `chmod +x build.sh` and wait a bit until the docker images are downloaded)

This will:

- Clear the previously built jniLibs
- Build the library using cross for Defined Android images (add a new image in the script if needed).
- Copy the generated library to the `jniLibs` folder.
- Use UniFFI to generate the JNI bindings and a `cel.kt` file at `./src/uniffi/cel/cel.kt`.
- Copy the necessary files to the `./target/android/` folder.

## Usage

The library defines three methods exposed to the host platform, which you can use depending on the type of
expression you want to evaluate:

```idl
 // Evaluates a CEL expression with provided variables and platform callbacks
 string evaluate_with_context(string definition, HostContext context);
 
 // Evaluates a CEL AST expression with provided variables, platform callbacks
 string evaluate_ast_with_context(string definition, HostContext context);
 
 // Evaluates a pure CEL AST expression
 string evaluate_ast(string ast);
```

The `HostContext` object is a callback interface allowing us to invoke host (iOS/Android) functions from our Rust code.
It provides a single function `computedProperty(name: String, args: String) -> String` that can be used to get the value of a property from the host.
The function passes in the name and the args (if required, serialized as JSON) of the dynamic function/property we want to invoke



### Android

To use the library in your Android application, you need to:
- Copy the `jniLibs` folder from `./target/android` to Android project's `superwall/src/main` folder.
- Copy the `cel.kt` file from `./src/uniffi/cel/cel.kt` to your Android project's `superwall/src/main/java/com/superwall/uniffi/cel/` folder.


The library exposes a single function currently:
`fn evaluate_with_context(definition: String, ctx: HostContext) -> String`

This function takes in a JSON containing the variables to be used and the expression to evaluate and returns the result.
The JSON is required to be in shape of `ExecutionContext`, which is defined as:

```json
{
  "variables": {
    // Map of variables that can be used in the expression
    // The key is the variable name, and the value is the variable value wrapped together with a type discriminator
    "map" : {
      "foo": {"type": "int", "value": 100},
      "numbers": {
        "type" : "list",
        "value" : [
          {"type": "int", "value": 1},
          {"type": "int", "value": 2},
          {"type": "int", "value": 3}
        ]
      },
      // Functions for our platform object - signature will be changed soon to allow for args
      "computed" : {
          "functionName": [{ // List of args
                            "type": "string",
                            "value": "event_name"
                        }]
      },
      // Functions for our device object - signature will be changed soon to allow for args
      "device" : {
        "functionName": [{ // List of args
          "type": "string",
          "value": "event_name"
        }]
      }

    }},
  // The expression to evaluate
  "expression": "foo == 100"
}
```

The `HostContext` object is a callback interface allowing us to invoke host (iOS/Android) functions from our Rust code.
It provides a single function `computedProperty(name: String) -> String` that can be used to get the value of a property from the host.
The function should return a JSON string containing the value of the property as `PassableValue`.

### iOS

To use the library in your iOS application, you need to:

1. Make the build script executable:
- `chmod +x ./build_ios.sh`
2. Run the build script:
- `./build_ios.sh`
3. Get the resulting XCframework from the `./target/xcframeworks/` folder and add it to your iOS project together 
with generated swift files from `./target/ios`


This should give you a `HostContext` protocol:
```swift
public protocol HostContextProtocol : AnyObject {
    func computedProperty(name: String)  -> String   
}
```

And a  `evaluateWithContext` method you can invoke:
```swift
public func evaluateWithContext(definition: String, context: HostContext) -> String
```


## Updating

When updating the library, you need to pay attention to uniffi bindings and ensure they match the signature of the library functions.
While it is tempting to migrate the library to use uniffi for the entire library, we still need to use JSON
for the input and output since UniFFI does not support recursive enums yet (such as PassableValue).
For that, track this [issue](https://github.com/mozilla/uniffi-rs/issues/396) for updates.
