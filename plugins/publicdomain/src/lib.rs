#![allow(dead_code)]

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
            0, 0,
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub(crate) struct Stream {
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
}

#[derive(Serialize, Deserialize)]
struct RefreshResponse {
    streams: Vec<Stream>,
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct SearchResult {
    pub identifier: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub year: Option<Value>,
    #[serde(default)]
    pub creator: Option<Value>,
}

#[derive(Deserialize)]
struct SearchResponseInner {
    docs: Vec<SearchResult>,
}

#[derive(Deserialize)]
struct SearchResponse {
    response: SearchResponseInner,
}

#[derive(Serialize)]
struct InteractSearchResult {
    id: String,
    title: String,
    subtitle: String,
}

// ============================================================
// Decade-based search queries
// ============================================================

/// Search queries for different decades and genres, each producing a batch
/// of results in a single HTTP request. Total: ~8 requests for 300-500 films.
const DECADE_QUERIES: &[(&str, &str, u32)] = &[
    // By decade -- feature_films collection with year ranges
    ("collection:feature_films AND year:[1920 TO 1929]", "1920s", 40),
    ("collection:feature_films AND year:[1930 TO 1939]", "1930s", 60),
    ("collection:feature_films AND year:[1940 TO 1949]", "1940s", 60),
    ("collection:feature_films AND year:[1950 TO 1959]", "1950s", 60),
    ("collection:feature_films AND year:[1960 TO 1969]", "1960s", 50),
    ("collection:feature_films AND year:[1970 TO 1979]", "1970s", 40),
    // Catch-all for films without a year or other decades
    ("collection:feature_films AND -year:[1920 TO 1979]", "Other", 50),
];

// ============================================================
// Pure logic (testable without host calls)
// ============================================================

/// Extract the year from Internet Archive's year field, which may be a number,
/// a string like "1931", or a string like "1931-01-01T00:00:00Z".
pub(crate) fn extract_year(year_val: &Option<Value>) -> Option<u32> {
    match year_val {
        Some(Value::Number(n)) => n.as_u64().map(|y| y as u32),
        Some(Value::String(s)) => {
            if s.len() >= 4 {
                s[..4].parse::<u32>().ok()
            } else {
                s.parse::<u32>().ok()
            }
        }
        _ => None,
    }
}

/// Determine the decade group string for a given year.
pub(crate) fn year_to_decade(year: u32) -> String {
    let decade_start = (year / 10) * 10;
    format!("{}s", decade_start)
}

/// Try to extract a year from a title string like "Millie (1931)".
pub(crate) fn year_from_title(title: &str) -> Option<u32> {
    if let Some(start) = title.rfind('(') {
        if let Some(end) = title[start..].find(')') {
            let candidate = &title[start + 1..start + end];
            if candidate.len() == 4 {
                return candidate.parse::<u32>().ok().filter(|&y| y >= 1800 && y <= 2100);
            }
        }
    }
    None
}

/// Build a heuristic download URL for a video item.
/// The Internet Archive commonly derives an .mp4 file named `{identifier}.mp4`.
pub(crate) fn heuristic_video_url(identifier: &str) -> String {
    format!("https://archive.org/download/{}/{}.mp4", identifier, identifier)
}

/// Parse the Internet Archive search response JSON and extract items.
pub(crate) fn parse_search_response(data: &[u8]) -> Vec<SearchResult> {
    match serde_json::from_slice::<SearchResponse>(data) {
        Ok(resp) => resp.response.docs,
        Err(_) => vec![],
    }
}

/// Build a stream from search result data using heuristic URL.
/// No per-item metadata fetch required.
pub(crate) fn build_stream_from_search(
    item: &SearchResult,
    default_group: &str,
) -> Stream {
    let identifier = &item.identifier;
    let title = item.title.as_deref().unwrap_or(identifier);

    let year = extract_year(&item.year).or_else(|| year_from_title(title));

    let group = match year {
        Some(y) => year_to_decade(y),
        None => default_group.to_string(),
    };

    let year_str = year.map(|y| y.to_string());

    let logo = format!("https://archive.org/services/img/{}", identifier);

    let display_name = match year {
        Some(y) if !title.contains(&format!("({})", y)) => format!("{} ({})", title, y),
        _ => title.to_string(),
    };

    let url = heuristic_video_url(identifier);

    Stream {
        id: identifier.to_string(),
        name: display_name,
        url,
        group,
        logo: Some(logo),
        vod_type: "movie".to_string(),
        year: year_str,
        tags: Some(vec!["public domain".to_string()]),
    }
}

/// Simple percent-encoding for URL path segments.
fn url_encode(input: &str) -> String {
    let mut encoded = String::with_capacity(input.len() * 2);
    for byte in input.bytes() {
        match byte {
            b'A'..=b'Z'
            | b'a'..=b'z'
            | b'0'..=b'9'
            | b'-'
            | b'_'
            | b'.'
            | b'~' => encoded.push(byte as char),
            b' ' => encoded.push_str("%20"),
            _ => {
                encoded.push('%');
                encoded.push_str(&format!("{:02X}", byte));
            }
        }
    }
    encoded
}

/// Parse max_films from config, with default of 300.
fn parse_max_films(config: &serde_json::Map<String, Value>) -> u32 {
    config
        .get("max_films")
        .and_then(|v| match v {
            Value::Number(n) => n.as_u64().map(|n| n as u32),
            Value::String(s) => s.parse::<u32>().ok(),
            _ => None,
        })
        .unwrap_or(300)
}

fn build_decade_search_url(query: &str, rows: u32) -> String {
    let encoded_query = query.replace(' ', "+");
    format!(
        "https://archive.org/advancedsearch.php?q={}&fl[]=identifier&fl[]=title&fl[]=description&fl[]=year&fl[]=creator&sort=downloads+desc&rows={}&output=json",
        encoded_query, rows
    )
}

// ============================================================
// Plugin exports
// ============================================================

#[no_mangle]
pub extern "C" fn describe() -> u64 {
    let desc = Descriptor {
        r#type: "publicdomain",
        label: "Public Domain Movies",
        short_label: "PD",
        color: "#8d6e63",
        version: "2.0.0",
        description: "Classic public domain feature films from Internet Archive, grouped by decade",
        config_fields: vec![
            serde_json::json!({
                "key": "max_films",
                "label": "Max films to load",
                "type": "number",
                "default": 300
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
                "id": "search",
                "label": "Search Films",
                "type": "search",
                "target_field": "query"
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

    // Check KV cache first
    if let Some(cached) = kv_get("pd_streams_cache_v2") {
        if let Ok(resp) = serde_json::from_str::<RefreshResponse>(&cached) {
            if !resp.streams.is_empty() {
                log_info(&format!("returning {} cached streams", resp.streams.len()));
                return return_json(&resp);
            }
        }
    }

    let max_films = parse_max_films(&config);

    let mut seen = std::collections::HashSet::new();
    let mut streams: Vec<Stream> = Vec::new();

    // Execute decade-based queries (one HTTP request per decade)
    for &(query, default_group, rows) in DECADE_QUERIES {
        if streams.len() >= max_films as usize {
            break;
        }

        let url = build_decade_search_url(query, rows);
        log_info(&format!("fetching: {}", url));

        let body = match http_get(&url) {
            Some(b) => b,
            None => {
                log_error(&format!("failed to fetch: {}", query));
                continue;
            }
        };

        let items = parse_search_response(&body);
        log_info(&format!("query returned {} items for {}", items.len(), default_group));

        for item in &items {
            if seen.contains(&item.identifier) {
                continue;
            }
            seen.insert(item.identifier.clone());
            streams.push(build_stream_from_search(item, default_group));

            if streams.len() >= max_films as usize {
                break;
            }
        }
    }

    log_info(&format!(
        "refresh complete: {} streams from {} HTTP requests",
        streams.len(),
        DECADE_QUERIES.len()
    ));

    // Cache results
    let resp = RefreshResponse { streams };
    if let Ok(cache_data) = serde_json::to_string(&resp) {
        kv_set("pd_streams_cache_v2", &cache_data);
    }

    return_json(&resp)
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

    if req.action != "search" {
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

    // Search Internet Archive with the user's query -- single HTTP request
    let encoded_query = url_encode(query);
    let search_url = format!(
        "https://archive.org/advancedsearch.php?q=collection:feature_films+AND+title:({})&fl[]=identifier&fl[]=title&fl[]=description&fl[]=year&fl[]=creator&rows=20&sort=downloads+desc&output=json",
        encoded_query
    );

    log_info(&format!("searching: {}", search_url));

    let body = match http_get(&search_url) {
        Some(b) => b,
        None => {
            log_error("search request failed");
            let empty: Vec<Value> = vec![];
            return return_json(&serde_json::json!({ "results": empty }));
        }
    };

    let items = parse_search_response(&body);

    let results: Vec<InteractSearchResult> = items
        .iter()
        .take(20)
        .map(|item| {
            let title = item.title.as_deref().unwrap_or(&item.identifier);
            let year = extract_year(&item.year);
            let subtitle = match year {
                Some(y) => format!("{} - {}", y, year_to_decade(y)),
                None => "Unknown year".to_string(),
            };
            InteractSearchResult {
                id: item.identifier.clone(),
                title: title.to_string(),
                subtitle,
            }
        })
        .collect();

    return_json(&serde_json::json!({ "results": results }))
}
