const path = require("path");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");

const dist = path.resolve(__dirname, "./target/browser/");

module.exports = {
    mode: "production",
    entry: {
        index: "./index.js"
    },
    output: {
        path: path.resolve(__dirname, './target/browser'),
        filename: "[name].js"
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
    },
};