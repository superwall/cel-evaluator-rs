# Supercel WASM Module

This project demonstrates how to use WebAssembly (WASM) with Rust and JavaScript to create a dynamic expression evaluator. The evaluator can call host environment functions and compute dynamic properties.

## Getting Started

### Prerequisites

- [Node.js](https://nodejs.org/)
- [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/)
- [Cargo & Rust](https://www.rust-lang.org/tools/install)

### Building the Project

To build the project, you can use either of the following commands:

```bash
npm run build
```

This will generate targets in the `/target/` directory
* `./target/browser` for browser environments
* `./target/node` for Node.js environments

(TODO: Add a script to build one module for both)

### Running the Project

