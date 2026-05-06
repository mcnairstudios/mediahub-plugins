# MediaHub Plugins

Each subdirectory contains a standalone WASM source plugin for MediaHub. Plugins are written in either **Rust** or **TinyGo** and compile to platform-independent `.wasm` binaries.

## Plugin Index

| Plugin | Language | Binary | Description |
|--------|----------|--------|-------------|
| [demo](demo/) | TinyGo | 325KB | Free public test streams (NASA Live, Big Buck Bunny, etc.) |
| [space](space/) | Rust | 90KB | Space launches from all agencies via Launch Library 2 API |
| [radiogarden](radiogarden/) | Rust | 183KB | Live radio stations from 12,000+ cities worldwide |
| [trailers](trailers/) | TinyGo | 423KB | Trending movie trailers from TMDB |

## Languages

### Rust

Rust plugins produce the smallest binaries (90-183KB) with zero-cost abstractions, no garbage collector, and Rust's memory safety guarantees. The WASM binary inherits Rust's ownership model — no use-after-free, no buffer overflows.

**Structure:**
```
plugin-name/
  Cargo.toml       # Dependencies: serde, serde_json
  src/lib.rs        # Plugin implementation
  .gitignore        # Excludes /target build dir
```

**Build:** `cargo build --release --target wasm32-wasip1`

**Requires:** Rust + `wasm32-wasip1` target (`rustup target add wasm32-wasip1`)

### TinyGo

TinyGo plugins are larger (325-423KB) but familiar to Go developers. TinyGo compiles a subset of Go to WASM with a minimal runtime.

**Structure:**
```
plugin-name/
  main.go           # Plugin implementation
  go.mod            # Go module
  Makefile           # Build target
```

**Build:** `tinygo build -o plugin.wasm -target=wasi -no-debug ./`

**Requires:** TinyGo (`brew install tinygo-org/tools/tinygo` or https://tinygo.org)

## Writing a New Plugin

See [PLUGIN_GUIDE.md](../PLUGIN_GUIDE.md) for the full developer guide. Every plugin exports three functions:

- **`describe()`** — returns plugin metadata, config fields, and view hints as JSON
- **`refresh(config)`** — fetches data from external APIs, returns a stream list as JSON
- **`interact(action)`** — handles dynamic config interactions (search, validate, etc.)

The host provides four functions to plugins:

- **`host_http_request`** — make HTTP requests
- **`host_log`** — write log messages
- **`host_kv_get`** / **`host_kv_set`** — plugin-scoped key-value cache
