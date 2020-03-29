## About

Routines for music analysis written in Rust packages into WebAssembly.

## 🚴 Usage

### 🛠️ Build with `wasm-pack build`

```
wasm-pack build --target web
```

Note: `--target web` provides necessary glue code in the generated `.js` for easy consumtion by an AudioWorklet. Omit if this is to be consumed from a regular Javascript module.

### 🔬 Test in Headless Browsers with `wasm-pack test`

```
wasm-pack test --headless --firefox
```

### 🎁 Publish to NPM with `wasm-pack publish`

```
wasm-pack publish --target web
```

## 🔋 Batteries Included

- [`wasm-bindgen`](https://github.com/rustwasm/wasm-bindgen) for communicating
  between WebAssembly and JavaScript.
- [`console_error_panic_hook`](https://github.com/rustwasm/console_error_panic_hook)
  for logging panic messages to the developer console.
- [`wee_alloc`](https://github.com/rustwasm/wee_alloc), an allocator optimized
  for small code size.
- [`cqt`]("https://github.com/alexjago/cqt") for applying a cqt transform to calculate
  pitches present in an audio sample.
