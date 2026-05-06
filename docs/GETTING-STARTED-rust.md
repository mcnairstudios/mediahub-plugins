# Getting Started: Rust Plugin

Write a MediaHub source plugin in Rust. Produces the smallest binaries (~90-183KB) with Rust's memory safety guarantees.

## Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add WASM target
rustup target add wasm32-wasip1
```

## Create a New Plugin

```bash
mkdir my-plugin && cd my-plugin
cargo init --lib .
```

**Cargo.toml:**
```toml
[package]
name = "my-plugin"
version = "1.0.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"

[profile.release]
opt-level = "s"
lto = true
strip = true
```

## Plugin Skeleton

**src/lib.rs:**
```rust
use serde::Serialize;
use std::slice;

// ============================================================
// Host function imports (provided by MediaHub)
// ============================================================

extern "C" {
    fn host_log(level: u32, msg_ptr: u32, msg_len: u32);
    fn host_http_request(
        url_ptr: u32, url_len: u32,
        method_ptr: u32, method_len: u32,
        headers_ptr: u32, headers_len: u32,
        body_ptr: u32, body_len: u32,
    ) -> u64;
    fn host_kv_get(key_ptr: u32, key_len: u32) -> u64;
    fn host_kv_set(key_ptr: u32, key_len: u32, val_ptr: u32, val_len: u32);
}

// ============================================================
// Memory management (required exports)
// ============================================================

#[no_mangle]
pub extern "C" fn alloc(size: u32) -> u32 {
    let mut buf: Vec<u8> = Vec::with_capacity(size as usize);
    buf.resize(size as usize, 0);
    let ptr = buf.as_ptr() as u32;
    std::mem::forget(buf);
    ptr
}

#[no_mangle]
pub extern "C" fn dealloc(_ptr: u32, _size: u32) {}

// ============================================================
// Helpers
// ============================================================

fn pack_ptr_len(ptr: u32, len: u32) -> u64 {
    ((ptr as u64) << 32) | (len as u64)
}

fn unpack_ptr_len(packed: u64) -> (u32, u32) {
    ((packed >> 32) as u32, (packed & 0xFFFFFFFF) as u32)
}

fn read_input(ptr: u32, len: u32) -> Vec<u8> {
    unsafe { slice::from_raw_parts(ptr as *const u8, len as usize).to_vec() }
}

fn return_json<T: Serialize>(value: &T) -> u64 {
    match serde_json::to_vec(value) {
        Ok(data) => {
            let ptr = data.as_ptr() as u32;
            let len = data.len() as u32;
            std::mem::forget(data);
            pack_ptr_len(ptr, len)
        }
        Err(_) => 0,
    }
}

fn log_info(msg: &str) {
    let bytes = msg.as_bytes();
    unsafe { host_log(1, bytes.as_ptr() as u32, bytes.len() as u32) }
}

fn http_get(url: &str) -> Option<Vec<u8>> {
    let url_bytes = url.as_bytes();
    let method = b"GET";
    let headers = b"{}";
    let result = unsafe {
        host_http_request(
            url_bytes.as_ptr() as u32, url_bytes.len() as u32,
            method.as_ptr() as u32, method.len() as u32,
            headers.as_ptr() as u32, headers.len() as u32,
            0, 0,
        )
    };
    if result == 0 { return None; }
    let (ptr, len) = unpack_ptr_len(result);
    if len == 0 { return None; }
    Some(unsafe { slice::from_raw_parts(ptr as *const u8, len as usize).to_vec() })
}

// ============================================================
// Data types
// ============================================================

#[derive(Serialize)]
struct Stream {
    id: String,
    name: String,
    url: String,
    group: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    logo: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    vod_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    year: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    episode_name: Option<String>,
}

#[derive(Serialize)]
struct RefreshResponse {
    streams: Vec<Stream>,
}

// ============================================================
// Plugin exports
// ============================================================

#[no_mangle]
pub extern "C" fn describe() -> u64 {
    return_json(&serde_json::json!({
        "type": "my-plugin",
        "label": "My Plugin",
        "short_label": "MP",
        "color": "#4caf50",
        "version": "1.0.0",
        "description": "My custom source plugin",
        "config_fields": [],
        "view": {
            "layout": "grouped_list",
            "group_by": "group",
            "searchable": true
        }
    }))
}

#[no_mangle]
pub extern "C" fn refresh(config_ptr: u32, config_len: u32) -> u64 {
    let _config = read_input(config_ptr, config_len);
    log_info("my-plugin: refreshing");

    // Fetch data from your API
    let body = match http_get("https://api.example.com/streams") {
        Some(b) => b,
        None => return return_json(&RefreshResponse { streams: vec![] }),
    };

    // Parse and build streams
    // let data: YourApiResponse = serde_json::from_slice(&body).unwrap_or_default();

    let streams = vec![
        Stream {
            id: "example-1".into(),
            name: "Example Stream".into(),
            url: "https://example.com/stream.m3u8".into(),
            group: "Examples".into(),
            logo: None,
            vod_type: None,
            year: None,
            tags: Some(vec!["live".into()]),
            episode_name: None,
        },
    ];

    return_json(&RefreshResponse { streams })
}

#[no_mangle]
pub extern "C" fn interact(action_ptr: u32, action_len: u32) -> u64 {
    let _ = read_input(action_ptr, action_len);
    return_json(&serde_json::json!({}))
}
```

## Build

```bash
cargo build --release --target wasm32-wasip1
# Output: target/wasm32-wasip1/release/my_plugin.wasm (~90KB)
```

## Install

```bash
cp target/wasm32-wasip1/release/my_plugin.wasm $MEDIAHUB_PLUGINS_DIR/
# Restart MediaHub — plugin loads automatically
```

## Tips

- Use `serde_json::Value` for flexible API responses where field types vary
- Use `http_get_with_headers()` for APIs requiring auth headers (pass as JSON: `{"Authorization": "Bearer xxx"}`)
- Use `host_kv_get`/`host_kv_set` to cache API responses between refreshes
- Keep the binary small: use `opt-level = "s"`, `lto = true`, `strip = true` in Cargo.toml
- Pass `0, 0` for empty HTTP body — never dereference an empty slice pointer

## Reference

- [Space plugin](../plugins/space/) — real-world Rust plugin with pagination, date parsing, flexible JSON
- [Radio Garden plugin](../plugins/radiogarden/) — Rust plugin with KV caching and interact() for search
