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

|file|bevy|wasm_opt|rust|notes|
|-|-|-|-|-|
|[`42674c1_win11_i713700KF_4080super.csv`](./results/42674c1_win11_i713700KF_4080super.csv)|0.16|123|1.86.0||
|[`6efce10_win11_i713700KF_4080super.csv`](./results/6efce10_win11_i713700KF_4080super.csv)|0.16|123|1.86.0|Bugged for `opt_level = 3`|
|[`d98b3a8_win11_i713700KF_4080super.csv`](./results/d98b3a8_win11_i713700KF_4080super.csv)|0.14|118|1.78.0|Bugged for `opt_level = 3`|

## Conclusions

There's some very rudimentary analysis in the [analysis](./analysis) folder, but there are tradeoffs between frame time, compile time, and file size to consider, so you may want to do your own.

For own my dist builds, I will be using:

|setting|value|note|
|-|-|-|
|opt_level|`S`|Avoid `Z`. `S` for file size, `3` for runtime performance.|
|wasm_opt|`Z`|Use `wasm_opt`. `Z` is slightly better for slightly longer compiles.|
|lto|`Fat`|`Thin` if compile time is at all a concern.|
|codegen_units|`One`|Seems good all around when combined with `wasm_opt`.|
|strip|`None`|Insignificant, and I want debug info.|
|panic|`unwind`|Insignificant, and I want debug info.|
