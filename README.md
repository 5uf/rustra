# Rustra

Rustra is a lightweight multi-threaded HTTP server with a WebAssembly demo, written in Rust.

It uses a custom `ThreadPool` backed by `crossbeam-channel` to handle concurrent connections, serves static HTML pages from the `pages/` directory, and exposes a `greet` function via `wasm-bindgen` for use in a browser WASM context.

## Table of Contents

- [Requirements](#requirements)
- [Building and running](#building-and-running)
- [Adding a new route](#adding-a-new-route)
- [Building the WASM bundle](#building-the-wasm-bundle)
- [Development](#development)
- [License](#license)

## Requirements

- [Rust](https://www.rust-lang.org/tools/install) (edition 2021)
- [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/) (for WASM builds only)

## Building and running

```sh
# Build (debug)
cargo build

# Build (release)
cargo build --release

# Run the server (listens on http://127.0.0.1:8080)
cargo run
```

Open `http://127.0.0.1:8080` in a browser. The server reads HTML files relative to the working directory, so run it from the repository root.

## Adding a new route

1. Add a `.html` file to the `pages/` directory, e.g. `pages/about.html`.
2. Add a new entry to the `routes` slice in `src/main.rs`:

```rust
let routes: &[(&str, &str, &str)] = &[
    ("/",      "200 OK", "pages/index.html"),
    ("/about", "200 OK", "pages/about.html"),
];
```

Unmatched requests automatically fall back to `pages/404.html` with a `404 Not Found` status.

## Building the WASM bundle

The `greet(name)` function in `src/lib.rs` is exported via `wasm-bindgen` and available when building for the `wasm32` target. `pages/index.html` loads it from `./pkg/rustra.js`.

```sh
wasm-pack build --target web
```

This produces a `pkg/` directory. Serve `pages/index.html` (and the `pkg/` directory) with any static file server to see the WASM greeting in action.

## Development

- [X] Multi-threaded TCP server with custom `ThreadPool`
- [X] Static HTML page serving from `pages/`
- [X] WASM `greet` export via `wasm-bindgen`
- [ ] WASM integration wired into the running server
- [ ] JSON configuration / settings
- [ ] Application-level load balancer
- [ ] Application-level DDoS protection

## License

MIT / Apache-2.0
