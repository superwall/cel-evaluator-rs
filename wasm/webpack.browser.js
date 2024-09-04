// webpack.browser.js used to build the WASM module for the browser environment.
const path = require("path");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");

const dist = path.resolve(__dirname, "./target/browser/");

module.exports = {
    name: "supercel-browser",
    mode: "production",
    entry: {
        index: "./index.js"
    },
    output: {
        path: path.resolve(__dirname, './target/browser'),
        filename: "supercel.js"
    },
    devServer: {
        contentBase: dist,
    },
    plugins: [
        new WasmPackPlugin({
            crateDirectory: path.resolve(__dirname, "."),
            outDir: "./target/browser",
            target: "web"
        }),
    ],
    experiments: {
        asyncWebAssembly: true,
        topLevelAwait: true
    },
};