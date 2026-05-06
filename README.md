# MediaHub Plugins

WASM source plugins for [MediaHub](https://github.com/mcnairstudios/mediahub). Each plugin is a standalone `.wasm` file that provides streams to MediaHub — no recompilation of MediaHub required.

## Plugins

| Plugin | Language | Description | Streams |
|--------|----------|-------------|---------|
| [demo](demo/) | TinyGo | Test streams (NASA Live, Big Buck Bunny, etc.) | 6 |
| [space](space/) | Rust | Space launches from all agencies (Launch Library 2) | ~700 |
| [radiogarden](radiogarden/) | TinyGo | Live radio stations worldwide (Radio Garden) | 1000+ |
| [trailers](trailers/) | TinyGo | Movie trailers from TMDB | ~40 |

## Installation

1. Build the plugin (see each plugin's directory for build instructions)
2. Copy the `.wasm` file to MediaHub's plugins directory:
   ```bash
   cp demo.wasm $MEDIAHUB_PLUGINS_DIR/
   # Default: $MEDIAHUB_DATA_DIR/plugins/
   ```
3. Restart MediaHub — the plugin loads automatically
4. Add a source of that type via the Streams page (+) button

## Building Plugins

### TinyGo plugins (demo, radiogarden, trailers)

```bash
# Install TinyGo: https://tinygo.org/getting-started/install/
cd demo/
tinygo build -o demo.wasm -target=wasi -no-debug ./
```

### Rust plugins (space)

```bash
# Install Rust: https://rustup.rs/
# Add WASM target:
rustup target add wasm32-wasip1

cd space/
cargo build --release --target wasm32-wasip1
# Output: target/wasm32-wasip1/release/space_rust.wasm
```

## Writing Your Own Plugin

See [PLUGIN_GUIDE.md](PLUGIN_GUIDE.md) for the complete developer guide, including:

- The WASM plugin contract (describe, refresh, interact)
- Host functions available (HTTP, logging, key-value store)
- Memory management conventions
- Complete examples in both TinyGo and Rust
- Config field types for the UI

### Quick Start (TinyGo)

```go
package main

import "unsafe"

//go:wasmimport env host_log
func hostLog(level uint32, msgPtr uint32, msgLen uint32)

//go:wasmimport env host_http_request
func hostHTTPRequest(urlPtr, urlLen, methodPtr, methodLen, headersPtr, headersLen, bodyPtr, bodyLen uint32) uint64

//export alloc
func alloc(size uint32) uint32 {
    buf := make([]byte, size)
    return uint32(uintptr(unsafe.Pointer(&buf[0])))
}

//export dealloc
func dealloc(ptr uint32, size uint32) {}

//export describe
func describe() uint64 {
    // Return JSON: {"type":"myplugin","label":"My Plugin",...}
}

//export refresh
func refresh(configPtr, configLen uint32) uint64 {
    // Fetch data via host_http_request, return streams as JSON
}

//export interact
func interact(actionPtr, actionLen uint32) uint64 {
    // Handle dynamic config interactions (search, validate, etc.)
}

func main() {}
```

### Quick Start (Rust)

```rust
extern "C" {
    fn host_log(level: u32, msg_ptr: u32, msg_len: u32);
    fn host_http_request(
        url_ptr: u32, url_len: u32, method_ptr: u32, method_len: u32,
        headers_ptr: u32, headers_len: u32, body_ptr: u32, body_len: u32,
    ) -> u64;
}

#[no_mangle]
pub extern "C" fn alloc(size: u32) -> u32 { /* allocate memory */ }

#[no_mangle]
pub extern "C" fn dealloc(ptr: u32, size: u32) { /* no-op in WASM */ }

#[no_mangle]
pub extern "C" fn describe() -> u64 { /* return JSON descriptor */ }

#[no_mangle]
pub extern "C" fn refresh(config_ptr: u32, config_len: u32) -> u64 { /* fetch and return streams */ }

#[no_mangle]
pub extern "C" fn interact(action_ptr: u32, action_len: u32) -> u64 { /* handle interactions */ }
```

## Plugin Contract

### Exports (your plugin provides)

| Function | Signature | Description |
|----------|-----------|-------------|
| `alloc` | `(size: u32) -> u32` | Allocate memory for host to write data |
| `dealloc` | `(ptr: u32, size: u32)` | Free previously allocated memory |
| `describe` | `() -> u64` | Return plugin metadata as JSON |
| `refresh` | `(config_ptr: u32, config_len: u32) -> u64` | Fetch streams, return as JSON |
| `interact` | `(action_ptr: u32, action_len: u32) -> u64` | Handle dynamic config actions |

Return values are packed `ptr+len` as `uint64`: `(ptr << 32) | len`

### Host Functions (MediaHub provides)

| Function | Description |
|----------|-------------|
| `host_http_request` | Make HTTP requests (GET, POST, etc.) |
| `host_log` | Log messages (debug, info, warn, error) |
| `host_kv_get` | Read from plugin-scoped key-value cache |
| `host_kv_set` | Write to plugin-scoped key-value cache |

### Stream JSON Format

```json
{
  "streams": [
    {
      "id": "unique-within-plugin",
      "name": "Stream Name",
      "url": "https://example.com/stream.m3u8",
      "group": "Group Name",
      "logo": "https://example.com/logo.png",
      "vod_type": "",
      "year": "2024",
      "tags": ["live", "hd"],
      "episode_name": "Optional description"
    }
  ]
}
```

## Supported Languages

Any language that compiles to WASM with WASI support works:

| Language | Binary Size | Notes |
|----------|------------|-------|
| **Rust** | ~90KB | Best WASM support, smallest binaries |
| **TinyGo** | ~300-450KB | Easy for Go developers |
| **C/C++** | ~50-200KB | Via wasi-sdk or Emscripten |
| **Zig** | ~50-100KB | Excellent WASM target |
| **AssemblyScript** | ~50-200KB | TypeScript-like syntax |

## License

Same as [MediaHub](https://github.com/mcnairstudios/mediahub).
