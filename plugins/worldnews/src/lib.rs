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

#[allow(dead_code)]
fn log_info(msg: &str) {
    let bytes = msg.as_bytes();
    unsafe { host_log(1, bytes.as_ptr() as u32, bytes.len() as u32) }
}

#[allow(dead_code)]
fn log_error(msg: &str) {
    let bytes = msg.as_bytes();
    unsafe { host_log(3, bytes.as_ptr() as u32, bytes.len() as u32) }
}

#[allow(dead_code)]
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

#[derive(Serialize, Clone, Debug)]
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
}

#[derive(Serialize)]
struct RefreshResponse {
    streams: Vec<Stream>,
}

// ============================================================
// Hardcoded stream catalog
// ============================================================

enum StreamUrl {
    /// Direct HLS stream URL
    Hls(&'static str),
    /// YouTube video/live ID (will be prefixed with watch URL)
    YouTube(&'static str),
}

struct NewsEntry {
    id: &'static str,
    name: &'static str,
    url: StreamUrl,
    group: &'static str,
    tags: &'static [&'static str],
}

const STREAMS: &[NewsEntry] = &[
    // ── International ──────────────────────────────────────
    NewsEntry {
        id: "dw-english",
        name: "Deutsche Welle (English)",
        url: StreamUrl::Hls("https://dwamdstream107.akamaized.net/hls/live/2017968/dwstream107/stream05/streamPlaylist.m3u8"),
        group: "International",
        tags: &["english", "german-broadcaster", "24/7"],
    },
    NewsEntry {
        id: "aljazeera-english",
        name: "Al Jazeera English",
        url: StreamUrl::Hls("https://live-hls-web-aje.getaj.net/AJE/index.m3u8"),
        group: "International",
        tags: &["english", "middle-east", "24/7"],
    },
    NewsEntry {
        id: "france24-english",
        name: "France 24 (English)",
        url: StreamUrl::Hls("https://uvotv-aniview.global.ssl.fastly.net/hls/live/2120684/france24english/playlist.m3u8"),
        group: "International",
        tags: &["english", "french-broadcaster", "24/7"],
    },
    NewsEntry {
        id: "cgtn-english",
        name: "CGTN News",
        url: StreamUrl::Hls("https://news.cgtn.com/resource/live/english/cgtn-news.m3u8"),
        group: "International",
        tags: &["english", "china", "24/7"],
    },
    NewsEntry {
        id: "trt-world",
        name: "TRT World",
        url: StreamUrl::Hls("https://tv-trtworld.live.trt.com.tr/master.m3u8"),
        group: "International",
        tags: &["english", "turkey", "24/7"],
    },
    NewsEntry {
        id: "nhk-world",
        name: "NHK World Japan",
        url: StreamUrl::Hls("https://cdn.nhkworld.jp/www11/nhkworld-tv/pre/hlscomp.m3u8"),
        group: "International",
        tags: &["english", "japan", "24/7"],
    },

    // ── Europe ─────────────────────────────────────────────
    NewsEntry {
        id: "dw-deutsch",
        name: "Deutsche Welle (Deutsch)",
        url: StreamUrl::Hls("https://dwamdstream111.akamaized.net/hls/live/2017972/dwstream111/stream05/streamPlaylist.m3u8"),
        group: "Europe",
        tags: &["german", "24/7"],
    },
    NewsEntry {
        id: "ard-daserste",
        name: "ARD Das Erste",
        url: StreamUrl::Hls("https://daserste-live.ard-mcdn.de/daserste/live/hls/de/master.m3u8"),
        group: "Europe",
        tags: &["german", "public-broadcaster", "24/7"],
    },
    NewsEntry {
        id: "euronews-english",
        name: "Euronews (English)",
        url: StreamUrl::YouTube("pBMpSLbNNHc"),
        group: "Europe",
        tags: &["english", "eu", "24/7"],
    },
    NewsEntry {
        id: "sky-news",
        name: "Sky News",
        url: StreamUrl::YouTube("9Auq9mYxFEE"),
        group: "Europe",
        tags: &["english", "uk", "24/7"],
    },
    NewsEntry {
        id: "gb-news",
        name: "GB News",
        url: StreamUrl::YouTube("Cch3-LRK1FU"),
        group: "Europe",
        tags: &["english", "uk", "24/7"],
    },
    NewsEntry {
        id: "france24-french",
        name: "France 24 (Francais)",
        url: StreamUrl::YouTube("l8PMl7tUDIE"),
        group: "Europe",
        tags: &["french", "24/7"],
    },

    // ── Asia ───────────────────────────────────────────────
    NewsEntry {
        id: "kbs-world",
        name: "KBS World 24",
        url: StreamUrl::Hls("https://kbsworld-ott.akamaized.net/hls/live/2002341/kbsworld/master.m3u8"),
        group: "Asia",
        tags: &["korean", "english", "24/7"],
    },
    NewsEntry {
        id: "arirang-korea",
        name: "Arirang TV (Korea)",
        url: StreamUrl::YouTube("rBAkIJBLmgs"),
        group: "Asia",
        tags: &["english", "korea", "24/7"],
    },
    NewsEntry {
        id: "cna-singapore",
        name: "CNA (Channel NewsAsia)",
        url: StreamUrl::YouTube("XWq5kBlakcQ"),
        group: "Asia",
        tags: &["english", "singapore", "24/7"],
    },
    NewsEntry {
        id: "wion-india",
        name: "WION News",
        url: StreamUrl::YouTube("MNwwMoFJwPI"),
        group: "Asia",
        tags: &["english", "india", "24/7"],
    },
    NewsEntry {
        id: "nhk-japan-yt",
        name: "NHK News (Japanese)",
        url: StreamUrl::YouTube("coYw-eVU0Ks"),
        group: "Asia",
        tags: &["japanese", "japan", "24/7"],
    },

    // ── Americas ───────────────────────────────────────────
    NewsEntry {
        id: "abc-news-live",
        name: "ABC News Live",
        url: StreamUrl::YouTube("GUyXFaR0yqM"),
        group: "Americas",
        tags: &["english", "usa", "24/7"],
    },
    NewsEntry {
        id: "cbs-news",
        name: "CBS News 24/7",
        url: StreamUrl::YouTube("plqnOSfqNyY"),
        group: "Americas",
        tags: &["english", "usa", "24/7"],
    },
    NewsEntry {
        id: "nbc-news-now",
        name: "NBC News NOW",
        url: StreamUrl::YouTube("msMHRuntuKw"),
        group: "Americas",
        tags: &["english", "usa", "24/7"],
    },
    NewsEntry {
        id: "fox-news-live",
        name: "Fox News Live",
        url: StreamUrl::YouTube("jL2sEaj1ops"),
        group: "Americas",
        tags: &["english", "usa", "24/7"],
    },
    NewsEntry {
        id: "cspan",
        name: "C-SPAN",
        url: StreamUrl::YouTube("NvqKZHpKs-g"),
        group: "Americas",
        tags: &["english", "usa", "government", "24/7"],
    },
    NewsEntry {
        id: "globo-news",
        name: "TV Globo News (Brazil)",
        url: StreamUrl::YouTube("gAHMzF0SGWQ"),
        group: "Americas",
        tags: &["portuguese", "brazil", "24/7"],
    },

    // ── Science & Space ────────────────────────────────────
    NewsEntry {
        id: "nasa-live",
        name: "NASA Live",
        url: StreamUrl::Hls("https://ntv1.akamaized.net/hls/live/2014075/NASA-NTV1-HLS/master.m3u8"),
        group: "Science & Space",
        tags: &["english", "space", "24/7"],
    },
    NewsEntry {
        id: "nasa-iss",
        name: "NASA ISS Earth Viewing",
        url: StreamUrl::YouTube("P9C25Un7xaM"),
        group: "Science & Space",
        tags: &["english", "space", "iss", "24/7"],
    },
    NewsEntry {
        id: "spacex-live",
        name: "SpaceX Launches & Events",
        url: StreamUrl::YouTube("bIZsnKGV8TE"),
        group: "Science & Space",
        tags: &["english", "space", "launches"],
    },
];

// ============================================================
// Stream builder (public for tests)
// ============================================================

pub fn build_streams() -> Vec<Stream> {
    STREAMS.iter()
        .map(|entry| {
            let url = match &entry.url {
                StreamUrl::Hls(u) => u.to_string(),
                StreamUrl::YouTube(vid) => format!("https://www.youtube.com/watch?v={}", vid),
            };
            Stream {
                id: entry.id.to_string(),
                name: entry.name.to_string(),
                url,
                group: entry.group.to_string(),
                logo: None,
                vod_type: "movie".to_string(),
                tags: Some(entry.tags.iter().map(|t| t.to_string()).collect()),
            }
        })
        .collect()
}

/// Return all unique group names in catalog order.
pub fn get_groups() -> Vec<String> {
    let mut groups: Vec<String> = Vec::new();
    for entry in STREAMS {
        let g = entry.group.to_string();
        if !groups.contains(&g) {
            groups.push(g);
        }
    }
    groups
}

/// Return total number of streams in the catalog.
pub fn stream_count() -> usize {
    STREAMS.len()
}

// ============================================================
// Plugin exports
// ============================================================

#[no_mangle]
pub extern "C" fn describe() -> u64 {
    let desc = Descriptor {
        r#type: "worldnews",
        label: "World News",
        short_label: "NEWS",
        color: "#1565c0",
        version: "0.1.0",
        description: "Curated 24/7 live news streams from major public broadcasters worldwide",
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

    let streams = build_streams();

    log_info(&format!("worldnews: returning {} streams", streams.len()));

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
