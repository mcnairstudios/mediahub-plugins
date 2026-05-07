use serde::Serialize;
use serde_json::Value;
use std::collections::HashSet;
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
// Data types -- Refresh / stream output
// ============================================================

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct Stream {
    pub id: String,
    pub name: String,
    pub url: String,
    pub group: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logo: Option<String>,
    pub vod_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub episode_name: Option<String>,
}

#[derive(Serialize)]
struct RefreshResponse {
    streams: Vec<Stream>,
}

// ============================================================
// RSS XML parsing (simple string matching, no XML library)
// ============================================================

/// Extract the text content between the first occurrence of `<tag>` and `</tag>`
/// within the given slice. Returns empty string if not found.
pub fn extract_tag<'a>(xml: &'a str, tag: &str) -> &'a str {
    let open = format!("<{}", tag);
    let close = format!("</{}>", tag);

    let start_pos = match xml.find(&open) {
        Some(pos) => pos,
        None => return "",
    };

    // Find the end of the opening tag (handle attributes like <tag attr="...">)
    let after_open = match xml[start_pos..].find('>') {
        Some(pos) => start_pos + pos + 1,
        None => return "",
    };

    let end_pos = match xml[after_open..].find(&close) {
        Some(pos) => after_open + pos,
        None => return "",
    };

    &xml[after_open..end_pos]
}

/// Extract the value of an attribute from a tag string.
/// e.g. extract_attr(r#"<enclosure url="http://example.com/v.mp4" />"#, "url")
pub fn extract_attr<'a>(tag_text: &'a str, attr: &str) -> &'a str {
    let needle = format!("{}=\"", attr);
    let start = match tag_text.find(&needle) {
        Some(pos) => pos + needle.len(),
        None => return "",
    };

    let end = match tag_text[start..].find('"') {
        Some(pos) => start + pos,
        None => return "",
    };

    &tag_text[start..end]
}

/// Extract the full `<enclosure .../>` or `<enclosure ...>` tag from an item block
/// and return the url attribute value.
pub fn extract_enclosure_url(item_xml: &str) -> &str {
    // Find the <enclosure tag
    let start = match item_xml.find("<enclosure") {
        Some(pos) => pos,
        None => return "",
    };

    // Find the end of this tag (either /> or >)
    let tag_end = match item_xml[start..].find('>') {
        Some(pos) => start + pos + 1,
        None => return "",
    };

    let tag_text = &item_xml[start..tag_end];
    extract_attr(tag_text, "url")
}

/// Decode common XML entities in a string.
pub fn decode_xml_entities(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&#39;", "'")
}

/// Strip CDATA wrapper if present: <![CDATA[...]]> -> ...
pub fn strip_cdata(s: &str) -> &str {
    let trimmed = s.trim();
    if trimmed.starts_with("<![CDATA[") && trimmed.ends_with("]]>") {
        &trimmed[9..trimmed.len() - 3]
    } else {
        trimmed
    }
}

/// Maximum number of items to parse from the feed.
/// The TED RSS feed contains ~2,700 items (9MB); we cap to keep things manageable.
const MAX_ITEMS: usize = 200;

/// Parse all <item> blocks from the RSS feed XML into Stream objects.
/// At most `MAX_ITEMS` items are returned.
pub fn parse_rss_items(xml: &str) -> Vec<Stream> {
    let mut streams = Vec::new();
    let mut seen_ids = HashSet::new();
    let mut search_from = 0;

    loop {
        if streams.len() >= MAX_ITEMS {
            break;
        }
        // Find next <item> block
        let item_start = match xml[search_from..].find("<item>") {
            Some(pos) => search_from + pos,
            None => {
                // Also try <item with attributes
                match xml[search_from..].find("<item ") {
                    Some(pos) => search_from + pos,
                    None => break,
                }
            }
        };

        let item_end = match xml[item_start..].find("</item>") {
            Some(pos) => item_start + pos + 7,
            None => break,
        };

        let item_xml = &xml[item_start..item_end];
        search_from = item_end;

        // Extract fields from this <item>
        let title_raw = extract_tag(item_xml, "title");
        let title = decode_xml_entities(strip_cdata(title_raw));
        if title.is_empty() {
            continue;
        }

        let video_url = extract_enclosure_url(item_xml);
        if video_url.is_empty() {
            continue;
        }

        // Extract optional fields
        let summary_raw = extract_tag(item_xml, "itunes:summary");
        let summary = decode_xml_entities(strip_cdata(summary_raw));

        let duration = extract_tag(item_xml, "itunes:duration").trim().to_string();

        // Extract image from <itunes:image href="..."/>
        let image_url = {
            let img_start = item_xml.find("<itunes:image");
            match img_start {
                Some(pos) => {
                    let tag_end = item_xml[pos..].find('>').map(|e| pos + e + 1).unwrap_or(pos);
                    let tag_text = &item_xml[pos..tag_end];
                    extract_attr(tag_text, "href").to_string()
                }
                None => String::new(),
            }
        };

        // Extract category from <category> if available
        let category_raw = extract_tag(item_xml, "category");
        let category = decode_xml_entities(strip_cdata(category_raw));

        // Extract link as fallback identifier
        let link = extract_tag(item_xml, "link").trim().to_string();

        // Generate a stable ID from the video URL
        let id = if !link.is_empty() {
            // Use the last path segment of the link as a readable ID
            link.rsplit('/')
                .find(|s| !s.is_empty())
                .unwrap_or(&link)
                .to_string()
        } else {
            // Fallback: hash-like from the video URL
            format!("ted-{}", streams.len())
        };

        // Deduplicate by ID
        if !seen_ids.insert(id.clone()) {
            continue;
        }

        let group = if !category.is_empty() {
            category
        } else {
            "TED Talks".to_string()
        };

        let logo = if !image_url.is_empty() {
            Some(image_url)
        } else {
            None
        };

        // Build tags from duration if available
        let tags = if !duration.is_empty() {
            Some(vec![format!("{}min", normalize_duration(&duration))])
        } else {
            None
        };

        let episode_name = if !summary.is_empty() {
            // Truncate long summaries (UTF-8 safe)
            if summary.len() > 300 {
                let truncated: String = summary.chars().take(297).collect();
                Some(format!("{}...", truncated))
            } else {
                Some(summary)
            }
        } else {
            None
        };

        streams.push(Stream {
            id,
            name: title,
            url: video_url.to_string(),
            group,
            logo,
            vod_type: "podcast".to_string(),
            tags,
            episode_name,
        });
    }

    streams
}

/// Normalize an itunes:duration value to minutes.
/// Handles formats: "00:18:30" (H:M:S), "18:30" (M:S), "1110" (seconds), "18" (minutes).
pub fn normalize_duration(dur: &str) -> String {
    let parts: Vec<&str> = dur.split(':').collect();
    match parts.len() {
        3 => {
            // HH:MM:SS
            let h: u32 = parts[0].parse().unwrap_or(0);
            let m: u32 = parts[1].parse().unwrap_or(0);
            let s: u32 = parts[2].parse().unwrap_or(0);
            let total_min = h * 60 + m + if s >= 30 { 1 } else { 0 };
            total_min.to_string()
        }
        2 => {
            // MM:SS
            let m: u32 = parts[0].parse().unwrap_or(0);
            let s: u32 = parts[1].parse().unwrap_or(0);
            let total_min = m + if s >= 30 { 1 } else { 0 };
            total_min.to_string()
        }
        1 => {
            // Could be seconds as a plain number or already minutes
            let n: u32 = parts[0].parse().unwrap_or(0);
            if n > 300 {
                // Likely seconds
                let min = n / 60;
                min.to_string()
            } else {
                n.to_string()
            }
        }
        _ => dur.to_string(),
    }
}

// ============================================================
// Plugin exports
// ============================================================

#[no_mangle]
pub extern "C" fn describe() -> u64 {
    let desc = Descriptor {
        r#type: "ted",
        label: "TED Talks",
        short_label: "TED",
        color: "#e62b1e",
        version: "1.0.0",
        description: "TED Talks audio from the official RSS feed (via Acast)",
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

    log_info("fetching TED Talks RSS feed");

    let body = match http_get("https://www.ted.com/feeds/talks.rss") {
        Some(b) => b,
        None => {
            log_error("failed to fetch TED RSS feed");
            return return_json(&RefreshResponse { streams: vec![] });
        }
    };

    let xml = String::from_utf8_lossy(&body);
    let streams = parse_rss_items(&xml);

    log_info(&format!("parsed {} TED talks from RSS feed", streams.len()));

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
