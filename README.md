# bevy_wasm_bench

Automated testing of Bevy for runtime performance with various wasm optimizations

## Requirements

- [`node`](https://nodejs.org/en/download)
- [`wasm-opt`](https://github.com/WebAssembly/binaryen/releases)
- [`wasm-bindgen-cli`](https://rustwasm.github.io/wasm-bindgen/reference/cli.html)

## Run

`cargo run -p runner --release`

## License

MIT or Apache 2.0

## Results

[`d98b3a8_win11_i713700KF_4080super.csv`](./results/d98b3a8_win11_i713700KF_4080super.csv)

## Conclusions

Data analysis is being left as an exercise for the reader. There are tradeoffs between frame time, compile time, and file size to consider. But for my dist builds, I will be using:

|setting|value|note|
|-|-|-|
|opt_level|`S`|`Z` is slow|
|wasm_opt|`S`|Use this, but the particular setting isn't super important|
|lto|`Fat`|`Thin` if compile time is a concern|
|codegen_units|`Default`|`One` if file size is a concern|
|strip|`None`|Insignificant|
