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

#[derive(Serialize, Clone)]
pub struct Stream {
    pub id: String,
    pub name: String,
    pub url: String,
    pub group: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logo: Option<String>,
    pub vod_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub year: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Serialize)]
struct RefreshResponse {
    streams: Vec<Stream>,
}

// ============================================================
// NASA API response parsing
// ============================================================

const DEFAULT_TOPICS: &str = "launch,ISS,Mars,Moon";
const DEFAULT_PAGE_SIZE: u32 = 50;

/// Build the search URL for a given topic and page size.
pub fn build_search_url(topic: &str, page_size: u32) -> String {
    let encoded_topic = topic.replace(' ', "%20");
    format!(
        "https://images-api.nasa.gov/search?media_type=video&q={}&page_size={}",
        encoded_topic, page_size
    )
}

/// Build the deterministic MP4 URL for a given nasa_id.
pub fn build_mp4_url(nasa_id: &str) -> String {
    format!(
        "https://images-assets.nasa.gov/video/{}/{}~mobile.mp4",
        nasa_id, nasa_id
    )
}

/// Build the thumbnail URL for a given nasa_id.
pub fn build_thumb_url(nasa_id: &str) -> String {
    format!(
        "https://images-assets.nasa.gov/video/{}/{}~thumb.jpg",
        nasa_id, nasa_id
    )
}

/// Extract the year from a date string like "2023-05-14T00:00:00Z".
pub fn extract_year(date_str: &str) -> Option<String> {
    if date_str.len() >= 4 {
        let year = &date_str[0..4];
        if year.chars().all(|c| c.is_ascii_digit()) {
            return Some(year.to_string());
        }
    }
    None
}

/// Parse a single item from the NASA search API collection.
pub fn parse_nasa_item(item: &Value, group: &str) -> Option<Stream> {
    let data = item.get("data")?.as_array()?;
    let first = data.first()?;

    let nasa_id = first.get("nasa_id")?.as_str()?;
    if nasa_id.is_empty() {
        return None;
    }

    let title = first
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("Untitled")
        .to_string();

    let description = first
        .get("description")
        .and_then(|v| v.as_str())
        .map(|s| {
            if s.len() > 200 {
                format!("{}...", &s[..197])
            } else {
                s.to_string()
            }
        });

    let date_created = first
        .get("date_created")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let year = extract_year(date_created);

    let keywords: Option<Vec<String>> = first.get("keywords").and_then(|v| {
        v.as_array().map(|arr| {
            arr.iter()
                .filter_map(|k| k.as_str().map(|s| s.to_lowercase()))
                .collect()
        })
    });

    let tags = if let Some(kw) = keywords {
        if kw.is_empty() { None } else { Some(kw) }
    } else {
        None
    };

    Some(Stream {
        id: nasa_id.to_string(),
        name: title,
        url: build_mp4_url(nasa_id),
        group: group.to_string(),
        logo: Some(build_thumb_url(nasa_id)),
        vod_type: "vod".to_string(),
        year,
        tags,
        description,
    })
}

/// Parse the full NASA search API response into a list of streams.
pub fn parse_search_response(body: &[u8], group: &str) -> Vec<Stream> {
    let json: Value = match serde_json::from_slice(body) {
        Ok(v) => v,
        Err(_) => return vec![],
    };

    let items = match json
        .get("collection")
        .and_then(|c| c.get("items"))
        .and_then(|i| i.as_array())
    {
        Some(arr) => arr,
        None => return vec![],
    };

    items.iter().filter_map(|item| parse_nasa_item(item, group)).collect()
}

/// Parse the topics config string (comma-separated) into a list of trimmed topics.
pub fn parse_topics(topics_str: &str) -> Vec<String> {
    topics_str
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

// ============================================================
// Plugin exports
// ============================================================

#[no_mangle]
pub extern "C" fn describe() -> u64 {
    let desc = Descriptor {
        r#type: "nasa",
        label: "NASA Video Library",
        short_label: "NASA",
        color: "#1e88e5",
        version: "1.0.0",
        description: "Video on demand from the NASA Image and Video Library",
        config_fields: vec![
            serde_json::json!({
                "key": "topics",
                "label": "Search Topics",
                "type": "text",
                "default": DEFAULT_TOPICS,
                "description": "Comma-separated list of topics to search (e.g. launch,ISS,Mars,Moon)"
            }),
            serde_json::json!({
                "key": "page_size",
                "label": "Results per Topic",
                "type": "number",
                "default": DEFAULT_PAGE_SIZE,
                "description": "Number of videos to fetch per topic (max 100)"
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

    let topics_str = config
        .get("topics")
        .and_then(|v| v.as_str())
        .unwrap_or(DEFAULT_TOPICS);

    let page_size = config
        .get("page_size")
        .and_then(|v| v.as_u64())
        .unwrap_or(DEFAULT_PAGE_SIZE as u64)
        .min(100) as u32;

    let topics = parse_topics(topics_str);

    if topics.is_empty() {
        log_info("no topics configured");
        return return_json(&RefreshResponse { streams: vec![] });
    }

    let mut streams: Vec<Stream> = Vec::new();

    for topic in &topics {
        let url = build_search_url(topic, page_size);
        log_info(&format!("fetching topic '{}': {}", topic, url));

        let body = match http_get(&url) {
            Some(b) => b,
            None => {
                log_error(&format!("http error fetching topic '{}'", topic));
                continue;
            }
        };

        let topic_streams = parse_search_response(&body, topic);
        log_info(&format!(
            "fetched {} videos for topic '{}'",
            topic_streams.len(),
            topic
        ));
        streams.extend(topic_streams);
    }

    log_info(&format!(
        "refresh complete: {} streams from {} topics",
        streams.len(),
        topics.len()
    ));

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
        return return_json(&serde_json::json!({ "results": empty }));
    }

    let url = build_search_url(query, 20);
    log_info(&format!("search interaction query='{}': {}", query, url));

    let body = match http_get(&url) {
        Some(b) => b,
        None => {
            log_error("http error during search interaction");
            let empty: Vec<Value> = vec![];
            return return_json(&serde_json::json!({ "results": empty }));
        }
    };

    let streams = parse_search_response(&body, query);

    #[derive(Serialize)]
    struct SearchResult {
        id: String,
        title: String,
        subtitle: String,
        url: String,
        thumb: String,
    }

    let results: Vec<SearchResult> = streams
        .iter()
        .take(20)
        .map(|s| SearchResult {
            id: s.id.clone(),
            title: s.name.clone(),
            subtitle: s.description.clone().unwrap_or_default(),
            url: s.url.clone(),
            thumb: s.logo.clone().unwrap_or_default(),
        })
        .collect();

    return_json(&serde_json::json!({ "results": results }))
}
