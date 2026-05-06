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

const YOUTUBE_RSS_URL: &str =
    "https://www.youtube.com/feeds/videos.xml?channel_id=UCBTlXPAfOx300RZfWNw8-qg";

const PLAYLIST_URL: &str =
    "https://www.youtube.com/playlist?list=PLYAQ82KNFI_cYU0EtHMphfNgZ8-h2N-S-";

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

/// Extract video IDs from YouTube RSS XML feed.
///
/// Looks for `<yt:videoId>XXXX</yt:videoId>` tags and corresponding
/// `<title>XXXX</title>` tags within `<entry>` elements.
/// Returns a list of (video_id, title) pairs.
fn parse_rss_xml(xml: &str) -> Vec<(String, String)> {
    let mut results = Vec::new();
    let mut search_from = 0;

    loop {
        // Find the next <entry> block
        let entry_start = match xml[search_from..].find("<entry>") {
            Some(pos) => search_from + pos,
            None => break,
        };
        let entry_end = match xml[entry_start..].find("</entry>") {
            Some(pos) => entry_start + pos + 8,
            None => break,
        };
        let entry = &xml[entry_start..entry_end];

        // Extract video ID
        let video_id = extract_tag_content(entry, "<yt:videoId>", "</yt:videoId>");
        // Extract title (within entry, the <title> after <yt:videoId>)
        let title = extract_tag_content(entry, "<title>", "</title>");

        if let Some(vid) = video_id {
            let t = title.unwrap_or_default();
            if !vid.is_empty() {
                results.push((vid, t));
            }
        }

        search_from = entry_end;
    }

    results
}

/// Extract text content between an opening and closing XML tag.
fn extract_tag_content(text: &str, open_tag: &str, close_tag: &str) -> Option<String> {
    let start = text.find(open_tag)?;
    let content_start = start + open_tag.len();
    let end = text[content_start..].find(close_tag)?;
    let content = &text[content_start..content_start + end];
    Some(content.trim().to_string())
}

/// Extract video IDs from YouTube playlist HTML page.
///
/// The playlist page contains video IDs in `"videoId":"XXXXXXXXXXX"` patterns
/// within the ytInitialData JSON blob embedded in the HTML.
/// Returns a list of (video_id, title) pairs.
fn parse_playlist_html(html: &str) -> Vec<(String, String)> {
    let mut results = Vec::new();
    let mut seen_ids: Vec<String> = Vec::new();

    // Strategy: find ytInitialData JSON and extract videoId + title pairs
    // The pattern in playlist HTML is:
    //   "videoId":"XXXXXXXXXXX" near "title":{"runs":[{"text":"TITLE"}]}
    // We look for playlistVideoRenderer objects.

    let marker = "\"playlistVideoRenderer\"";
    let mut search_from = 0;

    loop {
        let pos = match html[search_from..].find(marker) {
            Some(p) => search_from + p,
            None => break,
        };

        // Grab a reasonable chunk after the marker to find videoId and title
        let chunk_end = (pos + 2000).min(html.len());
        let chunk = &html[pos..chunk_end];

        // Extract videoId
        let video_id = extract_json_string_value(chunk, "\"videoId\":\"");

        // Extract title text from "title":{"runs":[{"text":"..."}]}
        // or "title":{"simpleText":"..."}
        let title = extract_playlist_item_title(chunk);

        if let Some(vid) = video_id {
            if !vid.is_empty() && !seen_ids.contains(&vid) {
                seen_ids.push(vid.clone());
                let t = title.unwrap_or_default();
                results.push((vid, t));
            }
        }

        search_from = pos + marker.len();
    }

    results
}

/// Extract a JSON string value that follows the given prefix pattern.
/// e.g., for prefix `"videoId":"`, extracts the string until the next `"`.
fn extract_json_string_value(text: &str, prefix: &str) -> Option<String> {
    let start = text.find(prefix)?;
    let value_start = start + prefix.len();
    let end = text[value_start..].find('"')?;
    let value = &text[value_start..value_start + end];
    // Sanity check: YouTube video IDs are 11 chars, alphanumeric + dash + underscore
    if value.len() > 20 || value.is_empty() {
        return None;
    }
    Some(value.to_string())
}

/// Extract the title of a playlist item from a chunk of JSON-like HTML.
fn extract_playlist_item_title(chunk: &str) -> Option<String> {
    // Try "title":{"runs":[{"text":"..."}]}
    if let Some(pos) = chunk.find("\"title\":{\"runs\":[{\"text\":\"") {
        let prefix = "\"title\":{\"runs\":[{\"text\":\"";
        let value_start = pos + prefix.len();
        if let Some(end) = chunk[value_start..].find('"') {
            let title = &chunk[value_start..value_start + end];
            if !title.is_empty() {
                return Some(unescape_json_string(title));
            }
        }
    }

    // Try "title":{"simpleText":"..."}
    if let Some(pos) = chunk.find("\"title\":{\"simpleText\":\"") {
        let prefix = "\"title\":{\"simpleText\":\"";
        let value_start = pos + prefix.len();
        if let Some(end) = chunk[value_start..].find('"') {
            let title = &chunk[value_start..value_start + end];
            if !title.is_empty() {
                return Some(unescape_json_string(title));
            }
        }
    }

    None
}

/// Basic unescape for JSON string content (handles common escapes).
fn unescape_json_string(s: &str) -> String {
    s.replace("\\\"", "\"")
        .replace("\\\\", "\\")
        .replace("\\n", "\n")
        .replace("\\u0026", "&")
        .replace("\\u003c", "<")
        .replace("\\u003e", ">")
        .replace("\\u0027", "'")
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

/// Merge two lists of streams, deduplicating by video ID.
/// The first list takes priority (its entries are kept if duplicated).
fn merge_streams(primary: Vec<Stream>, secondary: Vec<Stream>) -> Vec<Stream> {
    let mut seen: Vec<String> = Vec::new();
    let mut merged = Vec::new();

    for stream in primary.into_iter().chain(secondary.into_iter()) {
        if !seen.contains(&stream.id) {
            seen.push(stream.id.clone());
            merged.push(stream);
        }
    }

    merged
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

    // Fetch playlist page (primary source)
    let consent_headers = r#"{"Cookie":"CONSENT=YES+1"}"#;
    let playlist_streams = match http_get_with_headers(PLAYLIST_URL, consent_headers) {
        Some(body) => {
            let html = String::from_utf8_lossy(&body).to_string();
            let items = parse_playlist_html(&html);
            log_info(&format!("parsed {} items from playlist", items.len()));
            build_streams(&items)
        }
        None => {
            log_error("failed to fetch playlist page");
            Vec::new()
        }
    };

    // Fetch RSS feed (secondary source for recent uploads)
    let rss_streams = match http_get_with_headers(YOUTUBE_RSS_URL, "{}") {
        Some(body) => {
            let xml = String::from_utf8_lossy(&body).to_string();
            let items = parse_rss_xml(&xml);
            log_info(&format!("parsed {} items from RSS feed", items.len()));
            build_streams(&items)
        }
        None => {
            log_error("failed to fetch RSS feed");
            Vec::new()
        }
    };

    // Merge: playlist is primary, RSS is secondary
    let streams = merge_streams(playlist_streams, rss_streams);
    log_info(&format!("total unique streams: {}", streams.len()));

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
