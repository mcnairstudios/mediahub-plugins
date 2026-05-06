use serde::{Deserialize, Serialize};
use serde_json::Value;
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

#[cfg(not(test))]
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
// Data types -- API response
// ============================================================

/// Represents a single entry from the iptv-org streams.json API.
#[derive(Deserialize, Clone, Debug, PartialEq)]
pub(crate) struct IptvStream {
    #[serde(default)]
    pub channel: Option<String>,
    #[serde(default)]
    pub feed: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub quality: Option<String>,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub user_agent: Option<String>,
    #[serde(default)]
    pub referrer: Option<String>,
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

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub(crate) struct Stream {
    pub id: String,
    pub name: String,
    pub url: String,
    pub group: String,
    pub logo: String,
    pub vod_type: String,
    pub tags: Vec<String>,
}

#[derive(Serialize)]
struct RefreshResponse {
    streams: Vec<Stream>,
}

// ============================================================
// Core logic (testable without host functions)
// ============================================================

/// Returns true if the URL points to an HLS (.m3u8) stream.
/// Strips query strings and fragments before checking the extension.
pub(crate) fn is_hls_url(url: &str) -> bool {
    let path = url.split('?').next().unwrap_or(url);
    let path = path.split('#').next().unwrap_or(path);
    path.ends_with(".m3u8")
}

/// Build a deterministic stream ID from the index.
pub(crate) fn make_stream_id(index: usize) -> String {
    format!("iptv-{}", index)
}

/// Build tags from quality and label fields.
pub(crate) fn build_tags(quality: &Option<String>, label: &Option<String>) -> Vec<String> {
    let mut tags = Vec::new();
    if let Some(q) = quality {
        let q = q.trim();
        if !q.is_empty() {
            tags.push(q.to_lowercase());
        }
    }
    if let Some(l) = label {
        let l = l.trim();
        if !l.is_empty() {
            tags.push(l.to_lowercase());
        }
    }
    tags
}

/// Categorize a stream title into a meaningful group using keyword matching.
/// Returns a category string like "News", "Sports", "Movies", etc.
pub(crate) fn categorize_title(title: &str) -> String {
    let lower = title.to_lowercase();

    // Order matters: check more specific patterns first
    static RULES: &[(&[&str], &str)] = &[
        (&["news", "cnn", "bbc news", "fox news", "msnbc", "cnbc", "al jazeera", "reuters",
            "euronews", "france 24", "dw ", "rt ", "breaking news", "headline", "journal",
            "noticias", "nachrichten", "akhbar"], "News"),
        (&["sport", "espn", "football", "soccer", "cricket", "nba", "nfl", "mlb", "nhl",
            "tennis", "golf", "racing", "f1 ", "formula", "boxing", "wrestling", "ufc",
            "futbol", "calcio", "deportes"], "Sports"),
        (&["movie", "cinema", "film", "hollywood", "bollywood", "action movie",
            "thriller", "horror movie", "comedy movie", "classic movie"], "Movies"),
        (&["music", "mtv", "vh1", "radio", "hits", "songs", "concert", "jazz",
            "rock ", "pop ", "hip hop", "reggae", "country music", "musica",
            "musik", "musique"], "Music"),
        (&["kids", "cartoon", "nick", "disney", "junior", "child", "baby tv",
            "boomerang", "toon", "animat", "pokemon", "sesame"], "Kids"),
        (&["document", "discovery", "national geographic", "nat geo", "history",
            "nature", "science", "animal planet", "planet earth", "wildlife",
            "education"], "Documentary"),
        (&["cook", "food", "kitchen", "recipe", "chef", "bake", "cuisine",
            "travel", "adventure", "explore", "tourism", "destination",
            "lonely planet"], "Lifestyle"),
        (&["religion", "church", "gospel", "christian", "islamic", "quran",
            "bible", "faith", "prayer", "worship", "god ", "jesus", "allah",
            "hindu", "buddhist"], "Religious"),
        (&["shop", "qvc", "hsn", "home shopping", "teleshopping",
            "infomercial"], "Shopping"),
        (&["weather", "forecast", "climate", "meteo"], "Weather"),
        (&["gaming", "twitch", "esport", "game show", "gamer"], "Gaming"),
        (&["comedy", "funny", "laugh", "humor", "stand-up", "standup",
            "sitcom"], "Entertainment"),
        (&["drama", "series", "soap", "telenovela", "serial"], "Entertainment"),
    ];

    for (keywords, category) in RULES {
        for kw in *keywords {
            if lower.contains(kw) {
                return category.to_string();
            }
        }
    }

    "General".to_string()
}

/// Filter and map raw IPTV streams into plugin Stream objects.
/// Only keeps entries that have a title and a .m3u8 URL.
pub(crate) fn process_streams(raw: &[IptvStream]) -> Vec<Stream> {
    let mut streams = Vec::new();

    for (i, entry) in raw.iter().enumerate() {
        // Skip entries without a URL
        let url = match &entry.url {
            Some(u) if !u.is_empty() => u.clone(),
            _ => continue,
        };

        // Only keep HLS (.m3u8) streams
        if !is_hls_url(&url) {
            continue;
        }

        // Skip entries without a title
        let name = match &entry.title {
            Some(t) if !t.is_empty() => t.clone(),
            _ => continue,
        };

        let tags = build_tags(&entry.quality, &entry.label);
        let id = make_stream_id(i);
        let group = categorize_title(&name);

        streams.push(Stream {
            id,
            name,
            url,
            group,
            logo: String::new(),
            vod_type: "live".to_string(),
            tags,
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
        r#type: "iptvorg",
        label: "Live TV (iptv-org)",
        short_label: "IPTV",
        color: "#e53935",
        version: "1.0.0",
        description: "Free live TV channels from the iptv-org community directory",
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

const STREAMS_URL: &str = "https://iptv-org.github.io/api/streams.json";
const CACHE_KEY: &str = "iptvorg_streams_json";

#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn refresh(config_ptr: u32, config_len: u32) -> u64 {
    let _ = read_input(config_ptr, config_len);

    // Try to load raw JSON from KV cache first
    let json_bytes: Vec<u8> = if let Some(cached) = kv_get(CACHE_KEY) {
        log_info("using cached streams.json from KV store");
        cached.into_bytes()
    } else {
        log_info("fetching streams.json from iptv-org API");
        match http_get(STREAMS_URL) {
            Some(body) => {
                // Cache the raw JSON for future calls
                if let Ok(s) = std::str::from_utf8(&body) {
                    kv_set(CACHE_KEY, s);
                    log_info("cached streams.json to KV store");
                }
                body
            }
            None => {
                log_error("failed to fetch streams.json");
                return return_json(&RefreshResponse { streams: vec![] });
            }
        }
    };

    // Parse the JSON array of stream entries
    let raw_streams: Vec<IptvStream> = match serde_json::from_slice(&json_bytes) {
        Ok(s) => s,
        Err(e) => {
            log_error(&format!("failed to parse streams.json: {}", e));
            return return_json(&RefreshResponse { streams: vec![] });
        }
    };

    log_info(&format!("parsed {} raw stream entries", raw_streams.len()));

    let streams = process_streams(&raw_streams);

    log_info(&format!("filtered to {} HLS streams", streams.len()));

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
