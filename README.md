# Rustra

Rustra is a multi-threaded web framework with WebAssembly written in Rust. (Development)

## Table of Contents

- [Installation](#installation)
- [Usage](#usage)
- [Development](#development)
- [License](#license)

## Installation

To get started with Rustra, clone the repository and build the project:

```sh
git clone https://github.com/5uf/rustra.git
cd rustra
cargo build --release
wasm-pack build --target web
```

## Usage

```sh
cargo run
```

add .html files into pages folder and declare them in the main.rs (line 45)
the wasm can be accessed in HTML once declared in main.rs

## Development

- JSON settings
- WASM
- Load Balancer
- DDOS Protection




