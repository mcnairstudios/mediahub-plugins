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

fn http_get_with_headers(url: &str, headers_json: &str) -> Option<Vec<u8>> {
    let url_bytes = url.as_bytes();
    let method = b"GET";
    let headers = headers_json.as_bytes();
    let body = b"";

    let result = unsafe {
        host_http_request(
            url_bytes.as_ptr() as u32, url_bytes.len() as u32,
            method.as_ptr() as u32, method.len() as u32,
            headers.as_ptr() as u32, headers.len() as u32,
            body.as_ptr() as u32, body.len() as u32,
        )
    };

    log_info(&format!("http result raw: {}", result));

    if result == 0 {
        log_info("http result is 0");
        return None;
    }

    let (ptr, len) = unpack_ptr_len(result);
    log_info(&format!("http unpacked ptr={} len={}", ptr, len));
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

#[derive(Serialize, Debug, Clone)]
struct RefreshResponse {
    streams: Vec<Stream>,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
struct Stream {
    id: String,
    name: String,
    url: String,
    group: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    logo: Option<String>,
    vod_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    tags: Option<Vec<String>>,
}

// ============================================================
// Constants
// ============================================================

const PIPED_API_PRIMARY: &str =
    "https://api.piped.private.coffee/channel/UCBTlXPAfOx300RZfWNw8-qg";

const PIPED_API_BACKUP: &str =
    "https://pipedapi.in.projectsegfau.lt/channel/UCBTlXPAfOx300RZfWNw8-qg";

const CACHE_KEY: &str = "operavision_streams";

// ============================================================
// Parsing helpers (pure functions, testable without host)
// ============================================================

/// Build a YouTube watch URL from a video ID.
fn build_youtube_url(video_id: &str) -> String {
    format!("https://www.youtube.com/watch?v={}", video_id)
}

/// Build a YouTube thumbnail URL from a video ID.
fn build_thumbnail_url(video_id: &str) -> String {
    format!("https://i.ytimg.com/vi/{}/hqdefault.jpg", video_id)
}

/// Parse a performance title into (name, group).
///
/// OperaVision titles typically follow the pattern:
///   "TITLE Composer \u2013 Opera House"
/// where \u2013 is an en-dash. Some titles use " - " (hyphen) or " -- " instead.
/// If no delimiter is found, the full title is the name and "OperaVision" is the group.
fn parse_title(title: &str) -> (String, String) {
    // Try en-dash first (most common in OperaVision titles)
    if let Some(pos) = title.find(" \u{2013} ") {
        let name = title[..pos].trim().to_string();
        let group = title[pos + 4..].trim().to_string();
        if !name.is_empty() && !group.is_empty() {
            return (name, group);
        }
    }

    // Try " -- " (double hyphen, as mentioned in plan)
    if let Some(pos) = title.find(" -- ") {
        let name = title[..pos].trim().to_string();
        let group = title[pos + 4..].trim().to_string();
        if !name.is_empty() && !group.is_empty() {
            return (name, group);
        }
    }

    // Try " - " (single hyphen) as last resort
    // Only split on the last occurrence to avoid splitting compound titles
    if let Some(pos) = title.rfind(" - ") {
        let name = title[..pos].trim().to_string();
        let group = title[pos + 3..].trim().to_string();
        if !name.is_empty() && !group.is_empty() {
            return (name, group);
        }
    }

    // Fallback: full title as name, "OperaVision" as group
    (title.trim().to_string(), "OperaVision".to_string())
}

/// Detect genre tags from a title string.
/// Returns a list of lowercase tags like "opera", "ballet", "concert".
fn detect_tags(title: &str) -> Vec<String> {
    let lower = title.to_lowercase();
    let mut tags = Vec::new();

    if lower.contains("ballet") {
        tags.push("ballet".to_string());
    }
    if lower.contains("opera") || lower.contains("oper ") {
        tags.push("opera".to_string());
    }
    if lower.contains("concert") || lower.contains("recital") || lower.contains("gala") {
        tags.push("concert".to_string());
    }
    if lower.contains("symphony") || lower.contains("orchestra") {
        tags.push("orchestral".to_string());
    }

    // Default to "performance" if no genre detected
    if tags.is_empty() {
        tags.push("performance".to_string());
    }

    tags
}

/// Parse the Piped API JSON response into a list of (video_id, title) pairs.
///
/// The Piped API returns JSON like:
/// ```json
/// {
///   "name": "OperaVision",
///   "relatedStreams": [
///     { "url": "/watch?v=VIDEO_ID", "title": "...", "thumbnail": "...", "duration": 1234 }
///   ]
/// }
/// ```
fn parse_piped_json(json_str: &str) -> Vec<(String, String)> {
    let parsed: Value = match serde_json::from_str(json_str) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };

    let streams = match parsed.get("relatedStreams").and_then(|s| s.as_array()) {
        Some(arr) => arr,
        None => return Vec::new(),
    };

    let mut results = Vec::new();
    let mut seen_ids: Vec<String> = Vec::new();

    for stream in streams {
        let url = match stream.get("url").and_then(|u| u.as_str()) {
            Some(u) => u,
            None => continue,
        };

        let video_id = match extract_video_id_from_path(url) {
            Some(id) => id,
            None => continue,
        };

        if seen_ids.contains(&video_id) {
            continue;
        }

        let title = stream
            .get("title")
            .and_then(|t| t.as_str())
            .unwrap_or("")
            .to_string();

        seen_ids.push(video_id.clone());
        results.push((video_id, title));
    }

    results
}

/// Extract a YouTube video ID from a Piped URL path like "/watch?v=VIDEO_ID".
fn extract_video_id_from_path(path: &str) -> Option<String> {
    let marker = "v=";
    let start = path.find(marker)?;
    let id_start = start + marker.len();
    let rest = &path[id_start..];
    // Video ID ends at '&' or end of string
    let id = match rest.find('&') {
        Some(pos) => &rest[..pos],
        None => rest,
    };
    if id.is_empty() || id.len() > 20 {
        return None;
    }
    Some(id.to_string())
}

/// Convert a list of (video_id, title) pairs into Stream objects.
fn build_streams(items: &[(String, String)]) -> Vec<Stream> {
    items
        .iter()
        .map(|(video_id, title)| {
            let display_title = if title.is_empty() {
                video_id.clone()
            } else {
                title.clone()
            };
            let (name, group) = parse_title(&display_title);
            let tags = detect_tags(&display_title);

            Stream {
                id: video_id.clone(),
                name,
                url: build_youtube_url(video_id),
                group,
                logo: Some(build_thumbnail_url(video_id)),
                vod_type: "youtube".to_string(),
                tags: Some(tags),
            }
        })
        .collect()
}

/// Fetch channel data from a Piped API instance.
/// Returns the parsed (video_id, title) pairs, or None on failure.
fn fetch_piped(url: &str) -> Option<Vec<(String, String)>> {
    let headers = r#"{"User-Agent":"Mozilla/5.0"}"#;
    let body = http_get_with_headers(url, headers)?;
    let json_str = String::from_utf8_lossy(&body).to_string();
    let items = parse_piped_json(&json_str);
    if items.is_empty() {
        None
    } else {
        Some(items)
    }
}

// ============================================================
// Plugin exports
// ============================================================

#[no_mangle]
pub extern "C" fn describe() -> u64 {
    let desc = Descriptor {
        r#type: "operavision",
        label: "OperaVision",
        short_label: "OPERA",
        color: "#1a237e",
        version: "1.0.0",
        description: "Free opera, ballet, and concert performances from European opera houses via OperaVision",
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

    // Check cache first
    if let Some(cached) = kv_get(CACHE_KEY) {
        if !cached.is_empty() {
            log_info("returning cached operavision streams");
            let data = cached.into_bytes();
            let ptr = data.as_ptr() as u32;
            let len = data.len() as u32;
            std::mem::forget(data);
            return pack_ptr_len(ptr, len);
        }
    }

    // Fetch from Piped API (primary instance, with backup fallback)
    let items = match fetch_piped(PIPED_API_PRIMARY) {
        Some(items) => {
            log_info(&format!("parsed {} items from primary Piped API", items.len()));
            items
        }
        None => {
            log_error("primary Piped API failed, trying backup");
            match fetch_piped(PIPED_API_BACKUP) {
                Some(items) => {
                    log_info(&format!("parsed {} items from backup Piped API", items.len()));
                    items
                }
                None => {
                    log_error("backup Piped API also failed");
                    Vec::new()
                }
            }
        }
    };

    let streams = build_streams(&items);
    log_info(&format!("total streams: {}", streams.len()));

    let response = RefreshResponse { streams };

    // Cache the response
    if let Ok(json_str) = serde_json::to_string(&response) {
        kv_set(CACHE_KEY, &json_str);
        log_info("cached operavision streams");
    }

    return_json(&response)
}

#[no_mangle]
pub extern "C" fn interact(action_ptr: u32, action_len: u32) -> u64 {
    let _ = read_input(action_ptr, action_len);
    let data = b"{}";
    let ptr = data.as_ptr() as u32;
    let len = data.len() as u32;
    pack_ptr_len(ptr, len)
}
