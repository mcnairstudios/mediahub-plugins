use serde::{Deserialize, Serialize};
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

const CCMIXTER_HEADERS: &str =
    r#"{"User-Agent":"Mozilla/5.0","Referer":"https://ccmixter.org/"}"#;

fn http_get(url: &str) -> Option<Vec<u8>> {
    let url_bytes = url.as_bytes();
    let method = b"GET";
    let headers = CCMIXTER_HEADERS.as_bytes();

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

#[derive(Serialize, Clone, Debug, PartialEq)]
struct Stream {
    id: String,
    name: String,
    url: String,
    group: String,
    logo: String,
    vod_type: String,
    tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    http_headers: Option<Value>,
}

#[derive(Serialize)]
struct RefreshResponse {
    streams: Vec<Stream>,
}

// ============================================================
// Data types -- Search results
// ============================================================

#[derive(Serialize)]
struct SearchResult {
    id: String,
    title: String,
    subtitle: String,
}

// ============================================================
// API parsing helpers (pure functions, testable without host)
// ============================================================

/// Find the best audio file URL from the upload's files array.
/// Prefers MP3 (audio/mpeg), falls back to any audio file.
fn find_audio_url(files: &[Value]) -> Option<String> {
    // First pass: look for MP3
    for file in files {
        let mime = file
            .get("file_format_info")
            .and_then(|info| info.get("mime_type"))
            .and_then(|m| m.as_str())
            .unwrap_or("");
        if mime == "audio/mpeg" {
            if let Some(url) = file.get("download_url").and_then(|u| u.as_str()) {
                if !url.is_empty() {
                    return Some(url.to_string());
                }
            }
        }
    }

    // Second pass: any audio file
    for file in files {
        let mime = file
            .get("file_format_info")
            .and_then(|info| info.get("mime_type"))
            .and_then(|m| m.as_str())
            .unwrap_or("");
        if mime.starts_with("audio/") {
            if let Some(url) = file.get("download_url").and_then(|u| u.as_str()) {
                if !url.is_empty() {
                    return Some(url.to_string());
                }
            }
        }
    }

    None
}

/// Extract the first tag from the upload_tags comma-separated string.
/// Returns a cleaned, title-cased group name.
fn extract_group(upload: &Value) -> String {
    // Try upload_extra.ccud first (content type: remix, sample, a_cappella)
    if let Some(extra) = upload.get("upload_extra") {
        if let Some(ccud) = extra.get("ccud").and_then(|v| v.as_str()) {
            if !ccud.is_empty() {
                return format_group_name(ccud);
            }
        }
    }

    // Fall back to first tag from upload_tags
    let tags_str = upload
        .get("upload_tags")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    if let Some(first_tag) = tags_str.split(',').next() {
        let trimmed = first_tag.trim();
        if !trimmed.is_empty() {
            return format_group_name(trimmed);
        }
    }

    "Other".to_string()
}

/// Format a group name: replace underscores, title-case, pluralize known types.
fn format_group_name(raw: &str) -> String {
    let lower = raw.to_lowercase().replace('_', " ");
    match lower.as_str() {
        "remix" => "Remixes".to_string(),
        "remixes" => "Remixes".to_string(),
        "sample" => "Samples".to_string(),
        "samples" => "Samples".to_string(),
        "a cappella" | "acappella" | "a_cappella" => "A Cappellas".to_string(),
        "editorial pick" | "editorial_pick" => "Editorial Picks".to_string(),
        other => {
            // Title-case: capitalize first letter of each word
            other
                .split_whitespace()
                .map(|word| {
                    let mut chars = word.chars();
                    match chars.next() {
                        Some(c) => {
                            let upper: String = c.to_uppercase().collect();
                            format!("{}{}", upper, chars.as_str())
                        }
                        None => String::new(),
                    }
                })
                .collect::<Vec<_>>()
                .join(" ")
        }
    }
}

/// Extract user-defined tags from the upload.
fn extract_tags(upload: &Value) -> Vec<String> {
    // Try upload_extra.usertags first
    if let Some(extra) = upload.get("upload_extra") {
        if let Some(usertags) = extra.get("usertags").and_then(|v| v.as_str()) {
            let tags: Vec<String> = usertags
                .split(',')
                .map(|t| t.trim().to_string())
                .filter(|t| !t.is_empty())
                .collect();
            if !tags.is_empty() {
                return tags;
            }
        }
    }

    // Fall back to upload_tags
    let tags_str = upload
        .get("upload_tags")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    tags_str
        .split(',')
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty())
        .collect()
}

/// Convert a ccMixter upload JSON object into a Stream.
fn upload_to_stream(upload: &Value) -> Option<Stream> {
    let upload_id = upload
        .get("upload_id")
        .map(|v| match v {
            Value::Number(n) => n.to_string(),
            Value::String(s) => s.clone(),
            _ => String::new(),
        })
        .unwrap_or_default();

    if upload_id.is_empty() {
        return None;
    }

    let upload_name = upload
        .get("upload_name")
        .and_then(|v| v.as_str())
        .unwrap_or("Untitled")
        .to_string();

    let user_name = upload
        .get("user_real_name")
        .and_then(|v| v.as_str())
        .or_else(|| upload.get("user_name").and_then(|v| v.as_str()))
        .unwrap_or("Unknown")
        .to_string();

    let files = upload
        .get("files")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let audio_url = find_audio_url(&files)?;

    let license_logo = upload
        .get("license_logo_url")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let group = extract_group(upload);
    let tags = extract_tags(upload);

    let name = format!("{} - {}", upload_name, user_name);

    // Include required HTTP headers for audio playback (Referer needed for hotlink protection)
    let http_headers = Some(serde_json::json!({
        "Referer": "https://ccmixter.org/",
        "User-Agent": "Mozilla/5.0"
    }));

    Some(Stream {
        id: upload_id,
        name,
        url: audio_url,
        group,
        logo: license_logo,
        vod_type: String::new(),
        tags,
        http_headers,
    })
}

// ============================================================
// Plugin exports
// ============================================================

#[no_mangle]
pub extern "C" fn describe() -> u64 {
    let desc = Descriptor {
        r#type: "ccmixter",
        label: "ccMixter",
        short_label: "CC",
        color: "#7b1fa2",
        version: "1.0.0",
        description: "Creative Commons remixes, samples, and music",
        config_fields: vec![
            serde_json::json!({
                "key": "tags",
                "label": "Filter by tags",
                "type": "text",
                "default": "remix"
            }),
            serde_json::json!({
                "key": "limit",
                "label": "Number of tracks",
                "type": "number",
                "default": 50
            }),
        ],
        view: View {
            layout: "grouped_list",
            group_by: "group",
            searchable: true,
            sortable: true,
        },
        interactions: vec![serde_json::json!({
            "id": "search_tracks",
            "label": "Search Tracks",
            "type": "search",
            "params": [
                {
                    "key": "query",
                    "label": "Search query",
                    "type": "text"
                }
            ]
        })],
    };
    return_json(&desc)
}

#[no_mangle]
pub extern "C" fn refresh(config_ptr: u32, config_len: u32) -> u64 {
    let input = read_input(config_ptr, config_len);

    let config: serde_json::Map<String, Value> = match serde_json::from_slice(&input) {
        Ok(c) => c,
        Err(e) => {
            log_error(&format!("failed to parse config: {}", e));
            return return_json(&RefreshResponse { streams: vec![] });
        }
    };

    // Read config values with defaults
    let tags = config
        .get("tags")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let limit = config
        .get("limit")
        .and_then(|v| v.as_u64())
        .unwrap_or(50);

    let mut all_streams: Vec<Stream> = Vec::new();
    let mut seen_ids: std::collections::HashSet<String> = std::collections::HashSet::new();

    // If user specified tags, use them; otherwise fetch across multiple content types
    let tag_list: Vec<&str> = if tags.is_empty() {
        vec!["remix", "sample", "a_cappella", "editorial_pick"]
    } else {
        tags.split(',').map(|t| t.trim()).filter(|t| !t.is_empty()).collect()
    };

    let per_tag_limit = if tag_list.len() > 1 {
        std::cmp::max(limit / tag_list.len() as u64, 10)
    } else {
        limit
    };

    for tag in &tag_list {
        let url = format!(
            "http://ccmixter.org/api/query?f=json&tags={}&limit={}&sort=rank&ord=desc",
            tag, per_tag_limit
        );

        log_info(&format!("fetching ccmixter tracks: {}", url));

        let body = match http_get(&url) {
            Some(b) => b,
            None => {
                log_error(&format!("failed to fetch ccmixter API for tag '{}'", tag));
                continue;
            }
        };

        let uploads: Vec<Value> = match serde_json::from_slice(&body) {
            Ok(v) => v,
            Err(e) => {
                log_error(&format!("failed to parse ccmixter response for tag '{}': {}", tag, e));
                continue;
            }
        };

        log_info(&format!("parsed {} uploads from ccmixter for tag '{}'", uploads.len(), tag));

        for upload in &uploads {
            if let Some(stream) = upload_to_stream(upload) {
                if seen_ids.insert(stream.id.clone()) {
                    all_streams.push(stream);
                }
            }
        }
    }

    let streams = all_streams;

    log_info(&format!("returning {} streams", streams.len()));

    return_json(&RefreshResponse { streams })
}

#[no_mangle]
pub extern "C" fn interact(action_ptr: u32, action_len: u32) -> u64 {
    let input = read_input(action_ptr, action_len);

    #[derive(Deserialize)]
    struct InteractRequest {
        action: String,
        #[serde(default)]
        params: serde_json::Map<String, Value>,
    }

    let req: InteractRequest = match serde_json::from_slice(&input) {
        Ok(r) => r,
        Err(e) => {
            log_error(&format!("failed to parse interact request: {}", e));
            return return_json(&serde_json::json!({}));
        }
    };

    if req.action != "search_tracks" {
        return return_json(&serde_json::json!({}));
    }

    let query = req
        .params
        .get("query")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    if query.is_empty() {
        let empty: Vec<Value> = vec![];
        return return_json(&serde_json::json!({ "results": empty }));
    }

    let url = format!(
        "http://ccmixter.org/api/query?f=json&search={}&limit=20&sort=rank&ord=desc",
        query
    );

    log_info(&format!("searching ccmixter: {}", url));

    let body = match http_get(&url) {
        Some(b) => b,
        None => {
            log_error("failed to fetch ccmixter search");
            let empty: Vec<Value> = vec![];
            return return_json(&serde_json::json!({ "results": empty }));
        }
    };

    let uploads: Vec<Value> = match serde_json::from_slice(&body) {
        Ok(v) => v,
        Err(e) => {
            log_error(&format!("failed to parse search response: {}", e));
            let empty: Vec<Value> = vec![];
            return return_json(&serde_json::json!({ "results": empty }));
        }
    };

    let results: Vec<SearchResult> = uploads
        .iter()
        .filter_map(|upload| {
            let upload_id = upload.get("upload_id").map(|v| match v {
                Value::Number(n) => n.to_string(),
                Value::String(s) => s.clone(),
                _ => String::new(),
            })?;

            let upload_name = upload
                .get("upload_name")
                .and_then(|v| v.as_str())
                .unwrap_or("Untitled");

            let user_name = upload
                .get("user_real_name")
                .and_then(|v| v.as_str())
                .or_else(|| upload.get("user_name").and_then(|v| v.as_str()))
                .unwrap_or("Unknown");

            let license_name = upload
                .get("license_name")
                .and_then(|v| v.as_str())
                .unwrap_or("CC");

            Some(SearchResult {
                id: upload_id,
                title: upload_name.to_string(),
                subtitle: format!("{} | {}", user_name, license_name),
            })
        })
        .collect();

    log_info(&format!("search returned {} results", results.len()));

    return_json(&serde_json::json!({ "results": results }))
}
