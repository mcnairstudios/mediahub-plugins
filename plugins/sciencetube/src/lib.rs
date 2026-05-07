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
    let headers = br#"{"User-Agent":"Mozilla/5.0"}"#;

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
// Channel definitions
// ============================================================

struct Channel {
    name: &'static str,
    id: &'static str,
}

const CHANNELS: &[Channel] = &[
    Channel { name: "PBS Space Time", id: "UC7_gcs09iThXybpVgjHZ_7g" },
    Channel { name: "Veritasium", id: "UCHnyfMqiRRG1u-2MsSQLbXA" },
    Channel { name: "3Blue1Brown", id: "UCYO_jab_esuFRV4b17AJtAw" },
    Channel { name: "Numberphile", id: "UCoxcjq-8xIDTYp3uz647V5A" },
    Channel { name: "Computerphile", id: "UC9-y-6csu5WGm29I7JiwpnA" },
    Channel { name: "SmarterEveryDay", id: "UC6107grRI4m0o2-emgoDnAA" },
    Channel { name: "MinutePhysics", id: "UC5VwxFds7w_8rXmeQmhhjzw" },
    Channel { name: "PBS Eons", id: "UCzR-rom72PHN9Zg7RML9EbA" },
    Channel { name: "Kurzgesagt", id: "UCsXVk37bltHxD1rDPwtNM8Q" },
    Channel { name: "Royal Institution", id: "UCYeF244yNGuFefuFKqxIAXw" },
    Channel { name: "MIT OpenCourseWare", id: "UCEBb1b_L6zDS3xTUrIALZOw" },
    Channel { name: "Perimeter Institute", id: "UCpHvNclapYpq8b0ZnGRoMeg" },
    Channel { name: "Domain of Science", id: "UCxqAWLTk1CmBvZFPzeZMd9A" },
    Channel { name: "freeCodeCamp", id: "UC8butISFwT-Wl7EV0hUK0BQ" },
];

// ============================================================
// Piped API URLs
// ============================================================

const PIPED_BASE_PRIMARY: &str = "https://api.piped.private.coffee";
const PIPED_BASE_BACKUP: &str = "https://pipedapi.in.projectsegfau.lt";

/// Build the Piped API channel URL for a given channel ID.
pub fn channel_api_url(base: &str, channel_id: &str) -> String {
    format!("{}/channel/{}", base, channel_id)
}

pub fn video_url(video_id: &str) -> String {
    format!("https://www.youtube.com/watch?v={}", video_id)
}

pub fn thumbnail_url(video_id: &str) -> String {
    format!("https://i.ytimg.com/vi/{}/hqdefault.jpg", video_id)
}

// ============================================================
// Piped JSON parsing
// ============================================================

/// Extract the video ID from a Piped `url` field like "/watch?v=VIDEO_ID".
pub fn extract_video_id(url_field: &str) -> Option<String> {
    let prefix = "/watch?v=";
    if let Some(pos) = url_field.find(prefix) {
        let id_start = pos + prefix.len();
        let rest = &url_field[id_start..];
        // Video ID goes until end of string or next '&'
        let id = match rest.find('&') {
            Some(end) => &rest[..end],
            None => rest,
        };
        if id.is_empty() {
            None
        } else {
            Some(id.to_string())
        }
    } else {
        None
    }
}

/// Parse the Piped API JSON response for a channel into a list of Streams.
/// `channel_name` is the fallback name if the JSON `name` field is missing.
pub fn parse_piped_response(json_str: &str, channel_name: &str) -> Vec<Stream> {
    let parsed: Value = match serde_json::from_str(json_str) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };

    // Use the channel name from the API response if available, otherwise fallback.
    let group = parsed["name"]
        .as_str()
        .unwrap_or(channel_name)
        .to_string();

    let streams_array = match parsed["relatedStreams"].as_array() {
        Some(arr) => arr,
        None => return Vec::new(),
    };

    let mut streams = Vec::new();

    for item in streams_array {
        // Filter out Shorts
        if item["isShort"].as_bool().unwrap_or(false) {
            continue;
        }

        let url_field = match item["url"].as_str() {
            Some(u) => u,
            None => continue,
        };

        let vid_id = match extract_video_id(url_field) {
            Some(id) => id,
            None => continue,
        };

        let title = item["title"]
            .as_str()
            .unwrap_or(&vid_id)
            .to_string();

        let thumb = item["thumbnail"]
            .as_str()
            .map(|s| s.to_string())
            .unwrap_or_else(|| thumbnail_url(&vid_id));

        let episode_name = item["uploadedDate"]
            .as_str()
            .map(|s| s.to_string());

        streams.push(Stream {
            id: format!("yt-{}", vid_id),
            name: title,
            url: video_url(&vid_id),
            group: group.clone(),
            logo: Some(thumb),
            vod_type: "movie".to_string(),
            year: None,
            tags: Some(vec!["youtube".to_string()]),
            episode_name,
        });
    }

    streams
}

// ============================================================
// Plugin exports
// ============================================================

#[no_mangle]
pub extern "C" fn describe() -> u64 {
    let desc = Descriptor {
        r#type: "sciencetube",
        label: "Science & Tech",
        short_label: "SCI",
        color: "#1b5e20",
        version: "2.0.0",
        description: "Science and technology videos from top YouTube channels via Piped API",
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

    let mut streams: Vec<Stream> = Vec::new();

    for channel in CHANNELS {
        let primary_url = channel_api_url(PIPED_BASE_PRIMARY, channel.id);
        log_info(&format!("fetching channel {} from Piped: {}", channel.name, primary_url));

        let body = match http_get(&primary_url) {
            Some(b) => b,
            None => {
                // Try backup instance
                let backup_url = channel_api_url(PIPED_BASE_BACKUP, channel.id);
                log_info(&format!(
                    "primary failed for {}, trying backup: {}",
                    channel.name, backup_url
                ));
                match http_get(&backup_url) {
                    Some(b) => b,
                    None => {
                        log_error(&format!("failed to fetch channel {} from both instances", channel.name));
                        continue;
                    }
                }
            }
        };

        let json_str = String::from_utf8_lossy(&body);
        let channel_streams = parse_piped_response(&json_str, channel.name);
        log_info(&format!(
            "parsed {} videos from {}",
            channel_streams.len(),
            channel.name
        ));
        streams.extend(channel_streams);
    }

    log_info(&format!(
        "refresh complete: {} total streams from {} channels",
        streams.len(),
        CHANNELS.len()
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
