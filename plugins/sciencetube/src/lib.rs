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
// YouTube RSS feed URL
// ============================================================

pub fn feed_url(channel_id: &str) -> String {
    format!(
        "https://www.youtube.com/feeds/videos.xml?channel_id={}",
        channel_id
    )
}

pub fn video_url(video_id: &str) -> String {
    format!("https://www.youtube.com/watch?v={}", video_id)
}

pub fn thumbnail_url(video_id: &str) -> String {
    format!("https://i.ytimg.com/vi/{}/hqdefault.jpg", video_id)
}

// ============================================================
// Atom XML parsing (string matching)
// ============================================================

/// Extract all `<entry>...</entry>` blocks from Atom XML.
pub fn extract_entries(xml: &str) -> Vec<&str> {
    let mut entries = Vec::new();
    let mut search_from = 0;

    loop {
        let start = match xml[search_from..].find("<entry>") {
            Some(pos) => search_from + pos,
            None => break,
        };
        let end = match xml[start..].find("</entry>") {
            Some(pos) => start + pos + "</entry>".len(),
            None => break,
        };
        entries.push(&xml[start..end]);
        search_from = end;
    }

    entries
}

/// Extract the text content between `<tag>` and `</tag>`.
pub fn extract_tag(xml: &str, tag: &str) -> Option<String> {
    let open = format!("<{}", tag);
    let close = format!("</{}>", tag.split_whitespace().next().unwrap_or(tag));

    let start_tag = xml.find(&open)?;
    // Find the end of the opening tag (handle attributes)
    let content_start = xml[start_tag..].find('>')? + start_tag + 1;
    let content_end = xml[content_start..].find(&close)? + content_start;

    Some(xml[content_start..content_end].trim().to_string())
}

/// Extract an attribute value from a tag, e.g. `<media:thumbnail url="...">`.
pub fn extract_attr(xml: &str, tag: &str, attr: &str) -> Option<String> {
    let tag_start = xml.find(&format!("<{}", tag))?;
    let tag_end = xml[tag_start..].find('>')? + tag_start;
    let tag_content = &xml[tag_start..=tag_end];

    let attr_pattern = format!("{}=\"", attr);
    let attr_start = tag_content.find(&attr_pattern)? + attr_pattern.len();
    let attr_end = tag_content[attr_start..].find('"')? + attr_start;

    Some(tag_content[attr_start..attr_end].to_string())
}

/// Parse a single `<entry>` block into a Stream.
pub fn entry_to_stream(entry: &str, channel_name: &str) -> Option<Stream> {
    // Extract video ID from <yt:videoId>
    let vid_id = extract_tag(entry, "yt:videoId")?;

    // Extract title
    let title = extract_tag(entry, "title").unwrap_or_else(|| vid_id.clone());

    // Extract published date for year
    let published = extract_tag(entry, "published");
    let year = published.as_ref().and_then(|p| {
        if p.len() >= 4 {
            Some(p[..4].to_string())
        } else {
            None
        }
    });

    // Extract thumbnail URL -- prefer media:thumbnail, fallback to heuristic
    let thumb = extract_attr(entry, "media:thumbnail", "url")
        .unwrap_or_else(|| thumbnail_url(&vid_id));

    // Build episode name from published date
    let episode_name = published.as_ref().and_then(|p| {
        if p.len() >= 10 {
            Some(format_date(&p[..10]))
        } else {
            None
        }
    });

    Some(Stream {
        id: format!("yt-{}", vid_id),
        name: unescape_xml(&title),
        url: video_url(&vid_id),
        group: channel_name.to_string(),
        logo: Some(thumb),
        vod_type: "movie".to_string(),
        year,
        tags: Some(vec!["youtube".to_string()]),
        episode_name,
    })
}

/// Parse all entries from a feed into streams.
pub fn parse_feed(xml: &str, channel_name: &str) -> Vec<Stream> {
    let entries = extract_entries(xml);
    entries
        .iter()
        .filter_map(|entry| entry_to_stream(entry, channel_name))
        .collect()
}

/// Format a date string like "2025-01-14" into "Jan 14, 2025".
pub fn format_date(date: &str) -> String {
    if date.len() < 10 {
        return date.to_string();
    }

    let month_str = &date[5..7];
    let day_str = &date[8..10];
    let year_str = &date[0..4];

    let month_name = match month_str {
        "01" => "Jan",
        "02" => "Feb",
        "03" => "Mar",
        "04" => "Apr",
        "05" => "May",
        "06" => "Jun",
        "07" => "Jul",
        "08" => "Aug",
        "09" => "Sep",
        "10" => "Oct",
        "11" => "Nov",
        "12" => "Dec",
        _ => return date.to_string(),
    };

    format!("{} {}, {}", month_name, day_str, year_str)
}

/// Unescape basic XML entities.
pub fn unescape_xml(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&#39;", "'")
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
        version: "1.0.0",
        description: "Science and technology videos from top YouTube channels via RSS feeds",
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
        let url = feed_url(channel.id);
        log_info(&format!("fetching feed for {}: {}", channel.name, url));

        let body = match http_get(&url) {
            Some(b) => b,
            None => {
                log_error(&format!("failed to fetch feed for {}", channel.name));
                continue;
            }
        };

        let xml = String::from_utf8_lossy(&body);
        let channel_streams = parse_feed(&xml, channel.name);
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
