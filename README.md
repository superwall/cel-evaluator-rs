## Mobile CEL runtime

This is the Mobile Android CEL runtime library.
It is a standalone library that can be used to evaluate CEL expressions in Mobile applications by using
`cel-rust` crate.

## Installation

To build the library, you'll need to install:

1. Rust (https://www.rust-lang.org/tools/install)
2. Docker (for cross) (https://docs.docker.com/get-docker/)
3. cross (https://github.com/cross-rs/cross)

## Building

To build the library, run:

```shell
./build.sh
```

(note: for the first run you will need to `chmod +x build.sh` and wait a bit until the docker images are downloaded)

This will:

- Clear the previously built jniLibs
- Build the library using cross for Defined Android images (add a new image in the script if needed).
- Copy the generated library to the `jniLibs` folder.
- Use UniFFI to generate the JNI bindings and a `cel.kt` file at `./src/uniffi/cel/cel.kt`.
- Copy the necessary files to the `./target/android/` folder.

## Usage


### Android

To use the library in your Android application, you need to:
- Copy the `jniLibs` folder from `./target/android` to Android project's `superwall/src/main` folder.
- Copy the `cel.kt` file from `./src/uniffi/cel/cel.kt` to your Android project's `superwall/src/main/java/com/superwall/uniffi/cel/` folder.


The library exposes a single function currently:
`fn evaluate_with_context(definition: String) -> String`

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
      }

    }},
  // The expression to evaluate
  "expression": "foo == 100"
}
```

### iOS

To use the library in your iOS application, you need to:

## Updating

When updating the library, you need to pay attention to uniffi bindings and ensure they match the signature of the library functions.
While it is tempting to migrate the library to use uniffi for the entire library, we still need to use JSON
for the input and output since UniFFI does not support recursive enums yet (such as PassableValue).
For that, track this [issue](https://github.com/mozilla/uniffi-rs/issues/396) for updates.
