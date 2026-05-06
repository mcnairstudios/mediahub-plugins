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

fn http_get(url: &str) -> Option<Vec<u8>> {
    let url_bytes = url.as_bytes();
    let method = b"GET";
    let headers = b"{\"User-Agent\":\"Mozilla/5.0\"}";

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
// Constants
// ============================================================

/// Piped API instances ordered by reliability. The first working instance
/// is cached in KV so subsequent calls try it first.
const PIPED_INSTANCES: &[&str] = &[
    "https://pipedapi.kavin.rocks",
    "https://api.piped.private.coffee",
    "https://pipedapi.adminforge.de",
    "https://pipedapi.darkness.services",
];

const KV_KEY_INSTANCE: &str = "piped_instance";

/// Trending categories: (type parameter, group label).
/// An empty type string means "general" trending (no type param).
const TRENDING_CATEGORIES: &[(&str, &str)] = &[
    ("", "Trending"),
    ("music", "Trending Music"),
    ("gaming", "Trending Gaming"),
    ("movies", "Trending Movies"),
];

// ============================================================
// Data types
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

#[derive(Serialize, Clone, Debug, PartialEq)]
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
    #[serde(skip_serializing_if = "Option::is_none")]
    episode_name: Option<String>,
}

#[derive(Serialize)]
struct RefreshResponse {
    streams: Vec<Stream>,
}

#[derive(Deserialize)]
struct InteractRequest {
    action: String,
    #[serde(default)]
    params: serde_json::Map<String, Value>,
}

#[derive(Serialize)]
#[allow(dead_code)]
struct SearchResult {
    id: String,
    title: String,
    subtitle: String,
}

// ============================================================
// Piped API helpers
// ============================================================

/// Extract video ID from a Piped relative URL like "/watch?v=dQw4w9WgXcQ".
fn extract_video_id(url_path: &str) -> Option<String> {
    // Look for "v=" parameter
    let marker = "v=";
    if let Some(idx) = url_path.find(marker) {
        let start = idx + marker.len();
        let rest = &url_path[start..];
        // Video ID ends at '&' or end of string
        let end = rest.find('&').unwrap_or(rest.len());
        let id = &rest[..end];
        if !id.is_empty() {
            return Some(id.to_string());
        }
    }
    None
}

/// Truncate a string to at most `max_len` characters, appending "..." if truncated.
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len])
    }
}

/// Parse a single Piped video JSON object into a Stream, returning None if it
/// should be filtered out (e.g. shorts, missing video ID).
fn piped_video_to_stream(video: &Value, group: &str) -> Option<Stream> {
    // Filter out shorts
    if video.get("isShort").and_then(|v| v.as_bool()).unwrap_or(false) {
        return None;
    }

    let url_path = video.get("url").and_then(|v| v.as_str()).unwrap_or("");
    let video_id = extract_video_id(url_path)?;

    let title = video.get("title").and_then(|v| v.as_str()).unwrap_or("Untitled");
    let thumbnail = video.get("thumbnail").and_then(|v| v.as_str()).map(|s| s.to_string());
    let uploader = video.get("uploaderName").and_then(|v| v.as_str()).unwrap_or("");
    let short_desc = video.get("shortDescription").and_then(|v| v.as_str()).unwrap_or("");

    let tags = if !uploader.is_empty() {
        Some(vec![uploader.to_string()])
    } else {
        None
    };

    let episode_name = if !short_desc.is_empty() {
        Some(truncate(short_desc, 200))
    } else {
        None
    };

    Some(Stream {
        id: video_id.clone(),
        name: title.to_string(),
        url: format!("https://www.youtube.com/watch?v={}", video_id),
        group: group.to_string(),
        logo: thumbnail,
        vod_type: "movie".to_string(),
        tags,
        episode_name,
    })
}

/// Parse a Piped search response and return streams.
fn parse_search_results(body: &[u8]) -> Vec<Stream> {
    let resp: Value = match serde_json::from_slice(body) {
        Ok(v) => v,
        Err(_) => return vec![],
    };

    let items = match resp.get("items").and_then(|v| v.as_array()) {
        Some(arr) => arr,
        None => return vec![],
    };

    items
        .iter()
        .filter_map(|item| piped_video_to_stream(item, "Search Results"))
        .collect()
}

/// Build the URL for a trending request on a given Piped instance.
fn build_trending_url(instance: &str, region: &str, category_type: &str) -> String {
    if category_type.is_empty() {
        format!("{}/trending?region={}", instance, region)
    } else {
        format!("{}/trending?region={}&type={}", instance, region, category_type)
    }
}

/// Try to fetch JSON from a URL. Returns the raw bytes on success, None on failure.
fn try_fetch(url: &str) -> Option<Vec<u8>> {
    log_info(&format!("fetching: {}", url));
    http_get(url)
}

/// Get an ordered list of instances to try, with the cached instance first.
fn get_instance_order() -> Vec<String> {
    let mut instances: Vec<String> = Vec::new();

    // Try cached instance first
    if let Some(cached) = kv_get(KV_KEY_INSTANCE) {
        if !cached.is_empty() {
            instances.push(cached.clone());
        }
    }

    // Add the rest, skipping the cached one to avoid duplicates
    for &inst in PIPED_INSTANCES {
        let s = inst.to_string();
        if !instances.contains(&s) {
            instances.push(s);
        }
    }

    instances
}

/// Fetch trending videos for one category from the first working Piped instance.
/// Returns (streams, working_instance) on success.
fn fetch_trending_category(
    instances: &[String],
    region: &str,
    category_type: &str,
    group_label: &str,
) -> (Vec<Stream>, Option<String>) {
    for instance in instances {
        let url = build_trending_url(instance, region, category_type);
        let body = match try_fetch(&url) {
            Some(b) => b,
            None => {
                log_error(&format!("instance failed: {}", instance));
                continue;
            }
        };

        // Piped trending returns a JSON array directly
        let videos: Vec<Value> = match serde_json::from_slice(&body) {
            Ok(v) => v,
            Err(_) => {
                log_error(&format!("json parse failed for {}", instance));
                continue;
            }
        };

        let streams: Vec<Stream> = videos
            .iter()
            .filter_map(|v| piped_video_to_stream(v, group_label))
            .collect();

        log_info(&format!(
            "fetched {} videos for '{}' from {}",
            streams.len(),
            group_label,
            instance
        ));

        return (streams, Some(instance.clone()));
    }

    log_error(&format!("all instances failed for category '{}'", group_label));
    (vec![], None)
}

// ============================================================
// Plugin exports
// ============================================================

#[no_mangle]
pub extern "C" fn describe() -> u64 {
    let desc = Descriptor {
        r#type: "trending",
        label: "Trending Videos",
        short_label: "TREND",
        color: "#ff0000",
        version: "1.0.0",
        description: "YouTube trending videos via Piped API",
        config_fields: vec![
            serde_json::json!({
                "key": "region",
                "label": "Trending Region",
                "type": "select",
                "required": false,
                "default": "US",
                "options": [
                    {"value": "US", "label": "United States"},
                    {"value": "GB", "label": "United Kingdom"},
                    {"value": "CA", "label": "Canada"},
                    {"value": "AU", "label": "Australia"},
                    {"value": "DE", "label": "Germany"},
                    {"value": "FR", "label": "France"},
                    {"value": "JP", "label": "Japan"},
                    {"value": "BR", "label": "Brazil"},
                    {"value": "IN", "label": "India"},
                    {"value": "KR", "label": "South Korea"},
                    {"value": "MX", "label": "Mexico"},
                    {"value": "IT", "label": "Italy"},
                    {"value": "ES", "label": "Spain"},
                    {"value": "NL", "label": "Netherlands"},
                    {"value": "SE", "label": "Sweden"}
                ]
            }),
        ],
        view: View {
            layout: "grouped_list",
            group_by: "group",
            searchable: true,
            sortable: true,
        },
        interactions: vec![
            serde_json::json!({
                "id": "search_videos",
                "label": "Search Videos",
                "type": "search"
            }),
        ],
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

    let region = config
        .get("region")
        .and_then(|v| v.as_str())
        .unwrap_or("US");

    let instances = get_instance_order();
    let mut all_streams: Vec<Stream> = Vec::new();
    let mut working_instance: Option<String> = None;

    for &(category_type, group_label) in TRENDING_CATEGORIES {
        // If we already found a working instance, put it first
        let ordered: Vec<String> = if let Some(ref inst) = working_instance {
            let mut v = vec![inst.clone()];
            for i in &instances {
                if i != inst {
                    v.push(i.clone());
                }
            }
            v
        } else {
            instances.clone()
        };

        let (streams, inst) = fetch_trending_category(&ordered, region, category_type, group_label);
        all_streams.extend(streams);

        if let Some(inst) = inst {
            working_instance = Some(inst);
        }
    }

    // Cache the working instance for next time
    if let Some(ref inst) = working_instance {
        kv_set(KV_KEY_INSTANCE, inst);
    }

    log_info(&format!(
        "refresh complete: {} streams across {} categories",
        all_streams.len(),
        TRENDING_CATEGORIES.len()
    ));

    return_json(&RefreshResponse { streams: all_streams })
}

#[no_mangle]
pub extern "C" fn interact(action_ptr: u32, action_len: u32) -> u64 {
    let input = read_input(action_ptr, action_len);

    let req: InteractRequest = match serde_json::from_slice(&input) {
        Ok(r) => r,
        Err(e) => {
            log_error(&format!("failed to parse interact request: {}", e));
            return return_json(&serde_json::json!({}));
        }
    };

    if req.action != "search_videos" {
        return return_json(&serde_json::json!({}));
    }

    let query = req
        .params
        .get("query")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    if query.is_empty() {
        let empty: Vec<Value> = vec![];
        return return_json(&serde_json::json!({ "streams": empty }));
    }

    let instances = get_instance_order();

    for instance in &instances {
        let encoded_query = query.replace(' ', "+");
        let url = format!(
            "{}/search?q={}&filter=videos",
            instance, encoded_query
        );

        let body = match try_fetch(&url) {
            Some(b) => b,
            None => {
                log_error(&format!("search failed on instance: {}", instance));
                continue;
            }
        };

        let streams = parse_search_results(&body);

        log_info(&format!(
            "search '{}' returned {} results from {}",
            query,
            streams.len(),
            instance
        ));

        // Cache this working instance
        kv_set(KV_KEY_INSTANCE, instance);

        return return_json(&RefreshResponse { streams });
    }

    log_error("all instances failed for search");
    return_json(&RefreshResponse { streams: vec![] })
}
