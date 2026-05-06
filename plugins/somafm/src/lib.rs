use serde::Serialize;
use serde_json::Value;
use std::slice;

#[cfg(test)]
mod tests;

// ============================================================
// Host function imports
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
// Memory management exports
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
pub extern "C" fn dealloc(_ptr: u32, _size: u32) {
    // No-op in WASM -- memory reclaimed on module close.
}

// ============================================================
// Helpers
// ============================================================

fn pack_ptr_len(ptr: u32, len: u32) -> u64 {
    ((ptr as u64) << 32) | (len as u64)
}

fn unpack_ptr_len(packed: u64) -> (u32, u32) {
    let ptr = (packed >> 32) as u32;
    let len = (packed & 0xFFFFFFFF) as u32;
    (ptr, len)
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

fn log_error(msg: &str) {
    let bytes = msg.as_bytes();
    unsafe { host_log(3, bytes.as_ptr() as u32, bytes.len() as u32) }
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
            0, 0, // no body
        )
    };

    if result == 0 {
        return None;
    }

    let (ptr, len) = unpack_ptr_len(result);
    if len == 0 {
        return None;
    }

    Some(unsafe { slice::from_raw_parts(ptr as *const u8, len as usize).to_vec() })
}

#[allow(dead_code)]
fn kv_get(key: &str) -> Option<String> {
    let kb = key.as_bytes();
    let result = unsafe { host_kv_get(kb.as_ptr() as u32, kb.len() as u32) };
    if result == 0 {
        return None;
    }
    let (ptr, len) = unpack_ptr_len(result);
    if len == 0 {
        return None;
    }
    let data = unsafe { slice::from_raw_parts(ptr as *const u8, len as usize) };
    Some(String::from_utf8_lossy(data).to_string())
}

#[allow(dead_code)]
fn kv_set(key: &str, value: &str) {
    let kb = key.as_bytes();
    let vb = value.as_bytes();
    unsafe {
        host_kv_set(
            kb.as_ptr() as u32, kb.len() as u32,
            vb.as_ptr() as u32, vb.len() as u32,
        )
    }
}

// ============================================================
// Data types -- Plugin metadata
// ============================================================

#[derive(Serialize)]
struct Descriptor {
    r#type: &'static str,
    label: &'static str,
    short_label: &'static str,
    color: &'static str,
    version: &'static str,
    description: &'static str,
    config_fields: Vec<Value>,
    view: View,
    interactions: Vec<Value>,
}

#[derive(Serialize)]
struct View {
    layout: &'static str,
    group_by: &'static str,
    searchable: bool,
    sortable: bool,
}

// ============================================================
// Data types -- Refresh output
// ============================================================

#[derive(Serialize, Debug, PartialEq)]
pub(crate) struct RefreshResponse {
    pub streams: Vec<Stream>,
}

#[derive(Serialize, Debug, PartialEq)]
pub(crate) struct Stream {
    pub id: String,
    pub name: String,
    pub url: String,
    pub group: String,
    pub logo: String,
    pub vod_type: String,
    pub tags: Vec<String>,
}

// ============================================================
// Channel parsing logic
// ============================================================

/// Extract the first genre from a pipe-delimited genre string.
/// For example, "ambient|electronic" returns "ambient".
/// Returns "other" if the genre field is empty or missing.
pub(crate) fn extract_first_genre(genre: &str) -> String {
    let trimmed = genre.trim();
    if trimmed.is_empty() {
        return "other".to_string();
    }
    match trimmed.split('|').next() {
        Some(g) => {
            let g = g.trim();
            if g.is_empty() {
                "other".to_string()
            } else {
                g.to_string()
            }
        }
        None => "other".to_string(),
    }
}

/// Split a pipe-delimited genre string into a vector of tags.
pub(crate) fn split_genre_tags(genre: &str) -> Vec<String> {
    let trimmed = genre.trim();
    if trimmed.is_empty() {
        return vec![];
    }
    trimmed
        .split('|')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Pick the best MP3 stream URL from the playlists array.
/// Looks for entries with format "mp3" and picks the one with the highest
/// quality (bitrate). Falls back to any available URL if no MP3 is found.
pub(crate) fn pick_best_mp3_url(playlists: &[Value]) -> String {
    let mut best_url = String::new();
    let mut best_quality: &str = "";

    for entry in playlists {
        let format = entry
            .get("format")
            .and_then(|f| f.as_str())
            .unwrap_or("");
        let quality = entry
            .get("quality")
            .and_then(|q| q.as_str())
            .unwrap_or("");
        let url = entry
            .get("url")
            .and_then(|u| u.as_str())
            .unwrap_or("");

        if url.is_empty() {
            continue;
        }

        if format == "mp3" {
            // Compare quality strings like "highest", "high", "low".
            // Also handles numeric-like quality such as "256" vs "128".
            if best_url.is_empty() || quality_rank(quality) > quality_rank(best_quality) {
                best_url = url.to_string();
                best_quality = quality;
            }
        }
    }

    // If no MP3 found, take any URL as fallback
    if best_url.is_empty() {
        for entry in playlists {
            let url = entry
                .get("url")
                .and_then(|u| u.as_str())
                .unwrap_or("");
            if !url.is_empty() {
                return url.to_string();
            }
        }
    }

    best_url
}

/// Rank quality strings. Higher is better.
/// Handles SomaFM's quality values like "highest", "high", "low".
fn quality_rank(quality: &str) -> u32 {
    match quality.to_lowercase().as_str() {
        "highest" => 4,
        "high" => 3,
        "medium" | "med" => 2,
        "low" => 1,
        other => {
            // Try to parse as a numeric bitrate
            other.parse::<u32>().unwrap_or(0)
        }
    }
}

/// Parse a single channel JSON value into a Stream.
pub(crate) fn channel_to_stream(channel: &Value) -> Option<Stream> {
    let id = channel.get("id").and_then(|v| v.as_str())?;
    let title = channel
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or(id);
    let genre = channel
        .get("genre")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let largeimage = channel
        .get("largeimage")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let playlists = channel
        .get("playlists")
        .and_then(|v| v.as_array())
        .map(|a| a.as_slice())
        .unwrap_or(&[]);

    let url = pick_best_mp3_url(playlists);

    let group = extract_first_genre(genre);
    let tags = split_genre_tags(genre);

    Some(Stream {
        id: id.to_string(),
        name: title.to_string(),
        url,
        group,
        logo: largeimage.to_string(),
        vod_type: String::new(),
        tags,
    })
}

/// Parse the full SomaFM channels.json response body into streams.
pub(crate) fn parse_channels_response(body: &[u8]) -> Vec<Stream> {
    let parsed: Value = match serde_json::from_slice(body) {
        Ok(v) => v,
        Err(_) => return vec![],
    };

    let channels = match parsed.get("channels").and_then(|c| c.as_array()) {
        Some(arr) => arr,
        None => return vec![],
    };

    channels
        .iter()
        .filter_map(|ch| channel_to_stream(ch))
        .collect()
}

// ============================================================
// Plugin exports
// ============================================================

#[no_mangle]
pub extern "C" fn describe() -> u64 {
    let desc = Descriptor {
        r#type: "somafm",
        label: "SomaFM",
        short_label: "SOMA",
        color: "#2196f3",
        version: "1.0.0",
        description: "Curated internet radio channels from SomaFM",
        config_fields: vec![],
        view: View {
            layout: "grouped_list",
            group_by: "group",
            searchable: true,
            sortable: true,
        },
        interactions: vec![],
    };
    return_json(&desc)
}

#[no_mangle]
pub extern "C" fn refresh(config_ptr: u32, config_len: u32) -> u64 {
    let _ = read_input(config_ptr, config_len);

    log_info("fetching SomaFM channels");

    let body = match http_get("https://api.somafm.com/channels.json") {
        Some(b) => b,
        None => {
            log_error("failed to fetch SomaFM channels");
            return return_json(&RefreshResponse { streams: vec![] });
        }
    };

    let streams = parse_channels_response(&body);
    log_info(&format!("parsed {} SomaFM channels", streams.len()));

    return_json(&RefreshResponse { streams })
}

#[no_mangle]
pub extern "C" fn interact(action_ptr: u32, action_len: u32) -> u64 {
    let _ = read_input(action_ptr, action_len);
    let data = b"{}";
    let ptr = data.as_ptr() as u32;
    let len = data.len() as u32;
    pack_ptr_len(ptr, len)
}
