# Rustra

Rustra is a lightweight multi-threaded HTTP server with a WebAssembly demo, written in Rust.

It uses a custom `ThreadPool` backed by `crossbeam-channel` to handle concurrent connections, serves static HTML pages from the `pages/` directory, and exposes a `greet` function via `wasm-bindgen` for use in a browser WASM context.

## Table of Contents

- [Requirements](#requirements)
- [Building and running](#building-and-running)
- [Running tests](#running-tests)
- [Adding a new route](#adding-a-new-route)
- [Building the WASM bundle](#building-the-wasm-bundle)
- [HTTP response details](#http-response-details)
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

## Running tests

```sh
cargo test
```

The test suite covers:

- `ThreadPool`: zero-size panic, size accessor, job execution, clean shutdown.
- Route matching (`find_route`): root path, unknown paths (404), non-GET methods, CRLF line endings.

## Adding a new route

1. Add a `.html` file to the `pages/` directory, e.g. `pages/about.html`.
2. Add a new entry to the `ROUTES` constant in `src/main.rs`:

```rust
const ROUTES: &[(&str, &str, &str)] = &[
    ("/",      "200 OK", "pages/index.html"),
    ("/about", "200 OK", "pages/about.html"),
];
```

Unmatched requests automatically fall back to `pages/404.html` with a `404 Not Found` status.  
If a page file cannot be read at runtime, the server returns a `500 Internal Server Error` response instead of silently dropping the connection.

## Building the WASM bundle

The `greet(name)` function in `src/lib.rs` is exported via `wasm-bindgen` and available when building for the `wasm32` target. `pages/index.html` loads it from `./pkg/rustra.js`.

```sh
wasm-pack build --target web
```

This produces a `pkg/` directory. Serve `pages/index.html` (and the `pkg/` directory) with any static file server to see the WASM greeting in action.

## HTTP response details

Every response includes:

| Header | Value |
|--------|-------|
| `Content-Type` | `text/html; charset=utf-8` |
| `Content-Length` | byte length of the response body |
| `Connection` | `close` |

## Development

- [X] Multi-threaded TCP server with custom `ThreadPool`
- [X] Static HTML page serving from `pages/`
- [X] WASM `greet` export via `wasm-bindgen`
- [X] `Content-Type`, `Content-Length`, and `Connection` response headers
- [X] 500 fallback when a page file cannot be read
- [X] Unit tests for `ThreadPool` and route matching
- [ ] WASM integration wired into the running server
- [ ] JSON configuration / settings
- [ ] Application-level load balancer
- [ ] Application-level DDoS protection

## License

MIT / Apache-2.0
