use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;
use std::slice;

#[cfg(test)]
mod tests;

// ============================================================
// Host function imports
// ============================================================

#[cfg(not(test))]
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

#[allow(dead_code)]
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

#[cfg(not(test))]
fn log_info(msg: &str) {
    let bytes = msg.as_bytes();
    unsafe { host_log(1, bytes.as_ptr() as u32, bytes.len() as u32) }
}

#[cfg(not(test))]
fn log_error(msg: &str) {
    let bytes = msg.as_bytes();
    unsafe { host_log(3, bytes.as_ptr() as u32, bytes.len() as u32) }
}

#[cfg(test)]
fn log_info(_msg: &str) {}

#[cfg(test)]
fn log_error(_msg: &str) {}

#[cfg(test)]
fn http_get(_url: &str) -> Option<Vec<u8>> {
    None
}

#[cfg(test)]
#[allow(dead_code)]
fn kv_get(_key: &str) -> Option<String> {
    None
}

#[cfg(test)]
#[allow(dead_code)]
fn kv_set(_key: &str, _value: &str) {}

#[cfg(not(test))]
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

#[cfg(not(test))]
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

#[cfg(not(test))]
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
// URL encoding helper
// ============================================================

fn url_encode(s: &str) -> String {
    let mut result = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                result.push(b as char);
            }
            b' ' => result.push_str("%20"),
            _ => {
                result.push('%');
                result.push_str(&format!("{:02X}", b));
            }
        }
    }
    result
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
    tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    duration: Option<String>,
}

#[derive(Serialize)]
struct RefreshResponse {
    streams: Vec<Stream>,
}

// ============================================================
// Sepia Search response types
// ============================================================

#[derive(Deserialize, Debug, Clone)]
struct SepiaCategory {
    #[serde(default)]
    label: String,
}

#[derive(Deserialize, Debug, Clone)]
#[allow(non_snake_case)]
struct SepiaChannel {
    #[serde(default)]
    displayName: String,
}

#[derive(Deserialize, Debug, Clone)]
#[allow(non_snake_case)]
struct SepiaVideo {
    #[serde(default)]
    uuid: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    url: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    duration: i64,
    #[serde(default)]
    thumbnailUrl: String,
    #[serde(default)]
    nsfw: bool,
    #[serde(default)]
    category: Option<SepiaCategory>,
    #[serde(default)]
    channel: Option<SepiaChannel>,
}

#[derive(Deserialize, Debug)]
struct SepiaSearchResponse {
    #[serde(default)]
    data: Vec<SepiaVideo>,
    #[serde(default)]
    total: i64,
}

// ============================================================
// Video detail response types
// ============================================================

#[derive(Deserialize, Debug, Clone)]
#[allow(non_snake_case)]
struct VideoFile {
    #[serde(default)]
    fileUrl: String,
    #[serde(default)]
    fileDownloadUrl: String,
    #[serde(default)]
    resolution: Option<VideoResolution>,
}

#[derive(Deserialize, Debug, Clone)]
#[allow(dead_code)]
struct VideoResolution {
    #[serde(default)]
    id: i64,
    #[serde(default)]
    label: String,
}

#[derive(Deserialize, Debug, Clone)]
#[allow(non_snake_case, dead_code)]
struct StreamingPlaylist {
    #[serde(default)]
    playlistUrl: String,
    #[serde(default)]
    files: Vec<VideoFile>,
}

#[derive(Deserialize, Debug)]
#[allow(non_snake_case, dead_code)]
struct VideoDetail {
    #[serde(default)]
    uuid: String,
    #[serde(default)]
    files: Vec<VideoFile>,
    #[serde(default)]
    streamingPlaylists: Vec<StreamingPlaylist>,
}

// ============================================================
// Cached video URL entry
// ============================================================

#[derive(Serialize, Deserialize, Debug, Clone)]
#[allow(dead_code)]
struct CachedVideoUrl {
    url: String,
}

// ============================================================
// Core logic (testable functions)
// ============================================================

/// Parse the instance hostname from a PeerTube video watch URL.
/// e.g. "https://tube.example.org/videos/watch/some-uuid" -> "tube.example.org"
pub(crate) fn parse_instance_from_url(url: &str) -> Option<String> {
    let stripped = url.strip_prefix("https://").or_else(|| url.strip_prefix("http://"))?;
    let host = stripped.split('/').next()?;
    if host.is_empty() {
        return None;
    }
    Some(host.to_string())
}

/// Extract the best playable URL from a video detail response.
/// Prefers HLS streaming playlists, falls back to highest-resolution MP4.
pub(crate) fn extract_playable_url(detail: &VideoDetail) -> Option<String> {
    // Prefer HLS from streaming playlists
    for playlist in &detail.streamingPlaylists {
        if !playlist.playlistUrl.is_empty() {
            return Some(playlist.playlistUrl.clone());
        }
    }

    // Fall back to direct file URLs, preferring highest resolution
    let mut best_url: Option<String> = None;
    let mut best_resolution: i64 = -1;

    for file in &detail.files {
        let res = file.resolution.as_ref().map(|r| r.id).unwrap_or(0);
        let file_url = if !file.fileUrl.is_empty() {
            &file.fileUrl
        } else if !file.fileDownloadUrl.is_empty() {
            &file.fileDownloadUrl
        } else {
            continue;
        };

        if res > best_resolution {
            best_resolution = res;
            best_url = Some(file_url.clone());
        }
    }

    best_url
}

/// Parse a Sepia Search API response from raw JSON bytes.
pub(crate) fn parse_search_response(data: &[u8]) -> Option<SepiaSearchResponse> {
    serde_json::from_slice(data).ok()
}

/// Parse a video detail API response from raw JSON bytes.
pub(crate) fn parse_video_detail(data: &[u8]) -> Option<VideoDetail> {
    serde_json::from_slice(data).ok()
}

/// Format duration in seconds to a human-readable string like "1h 23m" or "5m 30s".
pub(crate) fn format_duration(seconds: i64) -> String {
    if seconds <= 0 {
        return String::new();
    }
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;

    if hours > 0 {
        format!("{}h {:02}m", hours, minutes)
    } else if minutes > 0 {
        format!("{}m {:02}s", minutes, secs)
    } else {
        format!("{}s", secs)
    }
}

/// Convert a Sepia Search result into a Stream, given the playable URL.
fn sepia_video_to_stream(video: &SepiaVideo, playable_url: &str) -> Stream {
    let instance = parse_instance_from_url(&video.url).unwrap_or_else(|| "unknown".to_string());

    let group = video
        .category
        .as_ref()
        .map(|c| c.label.clone())
        .filter(|l| !l.is_empty())
        .unwrap_or(instance);

    let duration_str = format_duration(video.duration);

    let logo = if video.thumbnailUrl.is_empty() {
        None
    } else {
        // thumbnailUrl may be relative (e.g. "/static/thumbnails/...") or absolute
        if video.thumbnailUrl.starts_with("http") {
            Some(video.thumbnailUrl.clone())
        } else {
            let inst = parse_instance_from_url(&video.url).unwrap_or_default();
            if inst.is_empty() {
                None
            } else {
                Some(format!("https://{}{}", inst, video.thumbnailUrl))
            }
        }
    };

    let channel_name = video
        .channel
        .as_ref()
        .map(|c| c.displayName.clone())
        .filter(|n| !n.is_empty());

    let tags = channel_name.map(|name| vec![name]);

    let description = if video.description.is_empty() {
        None
    } else {
        // Truncate long descriptions
        let desc = if video.description.len() > 200 {
            format!("{}...", &video.description[..197])
        } else {
            video.description.clone()
        };
        Some(desc)
    };

    Stream {
        id: video.uuid.clone(),
        name: video.name.clone(),
        url: playable_url.to_string(),
        group,
        logo,
        vod_type: "movie".to_string(),
        tags,
        description,
        duration: if duration_str.is_empty() {
            None
        } else {
            Some(duration_str)
        },
    }
}

// ============================================================
// Plugin exports
// ============================================================

#[no_mangle]
pub extern "C" fn describe() -> u64 {
    let desc = Descriptor {
        r#type: "peertube",
        label: "PeerTube",
        short_label: "PEER",
        color: "#f57d00",
        version: "1.0.0",
        description: "Browse and play videos from the federated PeerTube network via Sepia Search",
        config_fields: vec![
            serde_json::json!({
                "key": "search_terms",
                "label": "Search terms (comma-separated)",
                "type": "text",
                "default": "documentary,music,science,technology"
            }),
            serde_json::json!({
                "key": "max_results",
                "label": "Max results per search term",
                "type": "text",
                "default": "10"
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
                "type": "search",
                "target_field": "search_terms"
            }),
        ],
    };
    return_json(&desc)
}

/// Fetch the playable URL for a video, using KV cache to avoid redundant per-instance calls.
#[cfg(not(test))]
fn fetch_playable_url(video: &SepiaVideo) -> Option<String> {
    let cache_key = format!("pt_url_{}", video.uuid);

    // Check cache first
    if let Some(cached) = kv_get(&cache_key) {
        if let Ok(entry) = serde_json::from_str::<CachedVideoUrl>(&cached) {
            if !entry.url.is_empty() {
                log_info(&format!("cache hit for video {}", video.uuid));
                return Some(entry.url);
            }
        }
    }

    // Parse instance from the video watch URL
    let instance = match parse_instance_from_url(&video.url) {
        Some(h) => h,
        None => {
            log_error(&format!("cannot parse instance from URL: {}", video.url));
            return None;
        }
    };

    // Fetch video detail from the hosting instance
    let detail_url = format!("https://{}/api/v1/videos/{}", instance, video.uuid);
    log_info(&format!("fetching video detail: {}", detail_url));

    let body = match http_get(&detail_url) {
        Some(b) => b,
        None => {
            log_error(&format!("failed to fetch detail for video {} from {}", video.uuid, instance));
            return None;
        }
    };

    let detail: VideoDetail = match parse_video_detail(&body) {
        Some(d) => d,
        None => {
            log_error(&format!("failed to parse video detail for {}", video.uuid));
            return None;
        }
    };

    let playable = extract_playable_url(&detail);

    // Cache the result
    if let Some(ref url) = playable {
        let entry = CachedVideoUrl {
            url: url.clone(),
        };
        if let Ok(json) = serde_json::to_string(&entry) {
            kv_set(&cache_key, &json);
        }
    }

    playable
}

/// Test stub: returns None (no HTTP calls in tests).
#[cfg(test)]
fn fetch_playable_url(_video: &SepiaVideo) -> Option<String> {
    None
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

    // Read config values
    let search_terms_raw = config
        .get("search_terms")
        .and_then(|v| v.as_str())
        .unwrap_or("documentary,music,science,technology");

    let max_results: u32 = config
        .get("max_results")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse().ok())
        .unwrap_or(10);

    let terms: Vec<&str> = search_terms_raw
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    if terms.is_empty() {
        log_info("no search terms configured");
        return return_json(&RefreshResponse { streams: vec![] });
    }

    let mut streams: Vec<Stream> = Vec::new();
    let mut seen_uuids: HashSet<String> = HashSet::new();

    for term in &terms {
        let encoded_term = url_encode(term);
        let search_url = format!(
            "https://search.joinpeertube.org/api/v1/search/videos?search={}&count={}&nsfw=false",
            encoded_term, max_results
        );
        log_info(&format!("searching: {}", search_url));

        let body = match http_get(&search_url) {
            Some(b) => b,
            None => {
                log_error(&format!("failed to search for term: {}", term));
                continue;
            }
        };

        let search_resp: SepiaSearchResponse = match parse_search_response(&body) {
            Some(r) => r,
            None => {
                log_error(&format!("failed to parse search response for: {}", term));
                continue;
            }
        };

        log_info(&format!(
            "found {} results for '{}' (total: {})",
            search_resp.data.len(),
            term,
            search_resp.total
        ));

        for video in &search_resp.data {
            // Skip NSFW content
            if video.nsfw {
                continue;
            }

            // Deduplicate by UUID
            if video.uuid.is_empty() || !seen_uuids.insert(video.uuid.clone()) {
                continue;
            }

            // Fetch playable URL (with caching)
            let playable_url = match fetch_playable_url(video) {
                Some(url) => url,
                None => {
                    // Fall back to the watch page URL if we cannot get a direct stream
                    log_info(&format!("using watch URL as fallback for {}", video.uuid));
                    video.url.clone()
                }
            };

            streams.push(sepia_video_to_stream(video, &playable_url));
        }
    }

    log_info(&format!("refresh complete: {} streams", streams.len()));
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

    // Perform a search via Sepia Search and return results
    let encoded = url_encode(query);
    let search_url = format!(
        "https://search.joinpeertube.org/api/v1/search/videos?search={}&count=20&nsfw=false",
        encoded
    );

    let body = match http_get(&search_url) {
        Some(b) => b,
        None => {
            log_error("failed to search in interact");
            let empty: Vec<Value> = vec![];
            return return_json(&serde_json::json!({ "results": empty }));
        }
    };

    let search_resp: SepiaSearchResponse = match parse_search_response(&body) {
        Some(r) => r,
        None => {
            log_error("failed to parse search in interact");
            let empty: Vec<Value> = vec![];
            return return_json(&serde_json::json!({ "results": empty }));
        }
    };

    #[derive(Serialize)]
    struct SearchResult {
        id: String,
        title: String,
        subtitle: String,
    }

    let results: Vec<SearchResult> = search_resp
        .data
        .iter()
        .filter(|v| !v.nsfw)
        .take(20)
        .map(|v| {
            let instance = parse_instance_from_url(&v.url).unwrap_or_default();
            let dur = format_duration(v.duration);
            let subtitle = if dur.is_empty() {
                instance
            } else {
                format!("{} - {}", dur, instance)
            };
            SearchResult {
                id: v.uuid.clone(),
                title: v.name.clone(),
                subtitle,
            }
        })
        .collect();

    return_json(&serde_json::json!({ "results": results }))
}
