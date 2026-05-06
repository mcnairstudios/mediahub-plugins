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
pub struct Stream {
    id: String,
    name: String,
    url: String,
    group: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    logo: Option<String>,
    vod_type: String,
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
// Internet Archive search response types
// ============================================================

#[derive(Deserialize, Debug)]
pub struct SearchResponse {
    pub response: SearchResponseInner,
}

#[derive(Deserialize, Debug)]
pub struct SearchResponseInner {
    #[serde(default)]
    pub docs: Vec<SearchDoc>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct SearchDoc {
    #[serde(default)]
    pub identifier: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub year: Option<Value>,
    #[serde(default)]
    pub creator: Option<Value>,
}

// ============================================================
// Parsing helpers
// ============================================================

/// Parse search API JSON into docs.
pub fn parse_search_response(data: &[u8]) -> Option<Vec<SearchDoc>> {
    let resp: SearchResponse = serde_json::from_slice(data).ok()?;
    Some(resp.response.docs)
}

/// Extract a year string from a Value that may be a string or number.
pub fn extract_year(v: &Option<Value>) -> Option<String> {
    match v {
        Some(Value::String(s)) => {
            let trimmed = s.trim();
            if trimmed.len() >= 4 {
                Some(trimmed[..4].to_string())
            } else if !trimmed.is_empty() {
                Some(trimmed.to_string())
            } else {
                None
            }
        }
        Some(Value::Number(n)) => Some(n.to_string()),
        _ => None,
    }
}

/// Extract creator from a Value that may be a string or array.
pub fn extract_creator(v: &Option<Value>) -> Option<String> {
    match v {
        Some(Value::String(s)) if !s.is_empty() => Some(s.clone()),
        Some(Value::Array(arr)) => arr.first().and_then(|v| v.as_str()).map(|s| s.to_string()),
        _ => None,
    }
}

// ============================================================
// Character/creator grouping
// ============================================================

/// Known character/series keywords for grouping cartoons.
const CHARACTER_GROUPS: &[(&str, &str)] = &[
    ("popeye", "Popeye"),
    ("betty boop", "Betty Boop"),
    ("superman", "Superman"),
    ("felix", "Felix the Cat"),
    ("woody woodpecker", "Woody Woodpecker"),
    ("bugs bunny", "Bugs Bunny"),
    ("daffy duck", "Daffy Duck"),
    ("porky pig", "Porky Pig"),
    ("tom and jerry", "Tom and Jerry"),
    ("mickey mouse", "Mickey Mouse"),
    ("donald duck", "Donald Duck"),
    ("casper", "Casper"),
    ("mighty mouse", "Mighty Mouse"),
    ("little lulu", "Little Lulu"),
    ("koko the clown", "Koko the Clown"),
    ("fleischer", "Fleischer Studios"),
    ("looney tunes", "Looney Tunes"),
    ("merrie melodies", "Merrie Melodies"),
    ("silly symphon", "Silly Symphonies"),
    ("terrytoons", "Terrytoons"),
];

/// Determine the group for a cartoon based on title, description, and creator.
pub fn determine_group(title: &str, description: &Option<String>, creator: &Option<String>) -> String {
    let title_lower = title.to_lowercase();
    let desc_lower = description
        .as_ref()
        .map(|d| d.to_lowercase())
        .unwrap_or_default();
    let creator_lower = creator
        .as_ref()
        .map(|c| c.to_lowercase())
        .unwrap_or_default();

    // Check title first (most reliable), then description, then creator
    for &(keyword, group_name) in CHARACTER_GROUPS {
        if title_lower.contains(keyword)
            || desc_lower.contains(keyword)
            || creator_lower.contains(keyword)
        {
            return group_name.to_string();
        }
    }

    // Fallback: group by decade if year is available
    // Otherwise use creator or "Classic Cartoons"
    if let Some(c) = creator {
        if !c.is_empty() && c.len() < 40 {
            return c.clone();
        }
    }

    "Classic Cartoons".to_string()
}

/// Determine the decade group from a year string.
pub fn decade_from_year(year: &Option<String>) -> Option<String> {
    let y = year.as_ref()?;
    if y.len() < 4 {
        return None;
    }
    let num: u32 = y[..4].parse().ok()?;
    let decade = (num / 10) * 10;
    Some(format!("{}s", decade))
}

// ============================================================
// URL construction
// ============================================================

pub fn search_url() -> String {
    "https://archive.org/advancedsearch.php?q=collection:pdcartooncollection&fl[]=identifier&fl[]=title&fl[]=description&fl[]=year&fl[]=creator&rows=200&sort=downloads+desc&output=json".to_string()
}

pub fn video_url(identifier: &str) -> String {
    format!(
        "https://archive.org/download/{}/{}.mp4",
        identifier, identifier
    )
}

pub fn thumbnail_url(identifier: &str) -> String {
    format!("https://archive.org/services/img/{}", identifier)
}

/// Convert a search doc to a stream.
pub fn doc_to_stream(doc: &SearchDoc) -> Stream {
    let name = if !doc.title.is_empty() {
        doc.title.clone()
    } else {
        doc.identifier.clone()
    };

    let year = extract_year(&doc.year);
    let creator = extract_creator(&doc.creator);
    let group = determine_group(&name, &doc.description, &creator);

    Stream {
        id: doc.identifier.clone(),
        name,
        url: video_url(&doc.identifier),
        group,
        logo: Some(thumbnail_url(&doc.identifier)),
        vod_type: "movie".to_string(),
        year,
        tags: Some(vec!["cartoon".to_string()]),
        episode_name: None,
    }
}

// ============================================================
// Plugin exports
// ============================================================

#[no_mangle]
pub extern "C" fn describe() -> u64 {
    let desc = Descriptor {
        r#type: "cartoons",
        label: "Classic Cartoons",
        short_label: "TOONS",
        color: "#ff6f00",
        version: "1.0.0",
        description: "Public domain classic cartoons from the Internet Archive",
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

    let url = search_url();
    log_info(&format!("fetching cartoons: {}", url));

    let body = match http_get(&url) {
        Some(b) => b,
        None => {
            log_error("failed to fetch cartoon collection");
            return return_json(&RefreshResponse { streams: vec![] });
        }
    };

    let docs = match parse_search_response(&body) {
        Some(d) => d,
        None => {
            log_error("failed to parse cartoon search response");
            return return_json(&RefreshResponse { streams: vec![] });
        }
    };

    log_info(&format!("found {} cartoons", docs.len()));

    let streams: Vec<Stream> = docs
        .iter()
        .filter(|doc| !doc.identifier.is_empty())
        .map(|doc| doc_to_stream(doc))
        .collect();

    log_info(&format!(
        "refresh complete: {} cartoon streams",
        streams.len()
    ));

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
