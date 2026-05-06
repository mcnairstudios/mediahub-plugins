use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
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
    // No-op in WASM — memory reclaimed on module close.
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
    let body = b"";

    let result = unsafe {
        host_http_request(
            url_bytes.as_ptr() as u32, url_bytes.len() as u32,
            method.as_ptr() as u32, method.len() as u32,
            headers.as_ptr() as u32, headers.len() as u32,
            body.as_ptr() as u32, body.len() as u32,
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

#[derive(Serialize)]
struct RefreshResponse {
    streams: Vec<Stream>,
}

#[derive(Serialize, Clone, Debug)]
struct Stream {
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

// ============================================================
// Catalog parsing types
// ============================================================

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct VideoEntry {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(rename = "accessibilityLabel", default)]
    pub accessibility_label: String,
    #[serde(rename = "type", default)]
    pub video_type: String,
    #[serde(rename = "timeOfDay", default)]
    pub time_of_day: String,
    #[serde(rename = "pointsOfInterest", default)]
    pub points_of_interest: HashMap<String, String>,
    #[serde(default)]
    pub src: VideoSources,
}

#[derive(Deserialize, Debug, Clone, Default)]
pub(crate) struct VideoSources {
    #[serde(rename = "H2641080p", default)]
    pub h264_1080p: String,
    #[serde(rename = "H2651080p", default)]
    pub h265_1080p: String,
    #[serde(rename = "H2654k", default)]
    pub h265_4k: String,
}

// ============================================================
// Catalog URL
// ============================================================

const CATALOG_URL: &str =
    "https://raw.githubusercontent.com/OrangeJedi/Aerial/master/videos.json";

// ============================================================
// Parsing and conversion logic (testable without host calls)
// ============================================================

/// Parse the raw JSON catalog into a list of VideoEntry structs.
pub(crate) fn parse_catalog(data: &[u8]) -> Result<Vec<VideoEntry>, String> {
    serde_json::from_slice::<Vec<VideoEntry>>(data)
        .map_err(|e| format!("failed to parse catalog JSON: {}", e))
}

/// Capitalize the first letter of a string for display as a group name.
fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

/// Map a video type string to a human-readable group name.
pub(crate) fn group_name(video_type: &str) -> String {
    match video_type.to_lowercase().as_str() {
        "space" => "Space".to_string(),
        "underwater" => "Underwater".to_string(),
        "landscape" => "Landscape".to_string(),
        "cityscape" => "Cityscape".to_string(),
        other if other.is_empty() => "Other".to_string(),
        other => capitalize_first(other),
    }
}

/// Build the best display name for a video entry.
/// Prefer accessibilityLabel, fall back to name.
pub(crate) fn display_name(entry: &VideoEntry) -> String {
    if !entry.accessibility_label.is_empty() {
        entry.accessibility_label.clone()
    } else if !entry.name.is_empty() {
        entry.name.clone()
    } else {
        format!("Aerial {}", &entry.id[..8.min(entry.id.len())])
    }
}

/// Build the description from points of interest, sorted by timestamp.
pub(crate) fn build_description(entry: &VideoEntry) -> Option<String> {
    if entry.points_of_interest.is_empty() {
        return None;
    }

    let mut pois: Vec<(&String, &String)> = entry.points_of_interest.iter().collect();
    pois.sort_by_key(|(k, _)| k.parse::<u64>().unwrap_or(0));

    let desc = pois
        .iter()
        .map(|(_, v)| v.as_str())
        .collect::<Vec<&str>>()
        .join(" — ");

    Some(desc)
}

/// Convert a VideoEntry into a Stream, using H.264 1080p URL.
pub(crate) fn video_to_stream(entry: &VideoEntry) -> Option<Stream> {
    let url = if !entry.src.h264_1080p.is_empty() {
        entry.src.h264_1080p.clone()
    } else if !entry.src.h265_1080p.is_empty() {
        entry.src.h265_1080p.clone()
    } else if !entry.src.h265_4k.is_empty() {
        entry.src.h265_4k.clone()
    } else {
        return None;
    };

    let name = display_name(entry);
    let group = group_name(&entry.video_type);

    let mut tags = Vec::new();
    if !entry.time_of_day.is_empty() {
        tags.push(entry.time_of_day.to_lowercase());
    }

    let episode_name = build_description(entry);

    Some(Stream {
        id: entry.id.clone(),
        name,
        url,
        group,
        logo: None,
        vod_type: "movie".to_string(),
        year: None,
        tags: if tags.is_empty() { None } else { Some(tags) },
        episode_name,
    })
}

/// Convert an entire catalog into streams.
pub(crate) fn catalog_to_streams(entries: &[VideoEntry]) -> Vec<Stream> {
    entries.iter().filter_map(video_to_stream).collect()
}

// ============================================================
// Plugin exports
// ============================================================

#[no_mangle]
pub extern "C" fn describe() -> u64 {
    let desc = Descriptor {
        r#type: "aerials",
        label: "Apple TV Aerials",
        short_label: "AERIAL",
        color: "#0071e3",
        version: "1.0.0",
        description: "Apple TV 4K aerial screensaver videos — cityscapes, landscapes, underwater scenes, and Earth from the ISS",
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

    log_info("aerials: fetching video catalog");

    let body = match http_get(CATALOG_URL) {
        Some(b) => b,
        None => {
            log_error("aerials: failed to fetch catalog");
            return return_json(&RefreshResponse { streams: vec![] });
        }
    };

    let entries = match parse_catalog(&body) {
        Ok(e) => e,
        Err(msg) => {
            log_error(&format!("aerials: {}", msg));
            return return_json(&RefreshResponse { streams: vec![] });
        }
    };

    let streams = catalog_to_streams(&entries);
    log_info(&format!("aerials: emitting {} streams from {} entries", streams.len(), entries.len()));

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
