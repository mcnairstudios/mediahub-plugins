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
    // No-op in WASM — memory reclaimed on module close.
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

struct CamEntry {
    id: &'static str,
    name: &'static str,
    video_id: &'static str,
    group: &'static str,
    tags: &'static [&'static str],
}

const CAMS: &[CamEntry] = &[
    // ── Volcanoes ───────────────────────────────────────────
    CamEntry {
        id: "volcano-kilauea-multicam",
        name: "Kilauea Volcano Multi-Cam",
        video_id: "FVdmnpJ2kM0",
        group: "Volcanoes",
        tags: &["volcano", "hawaii", "24/7"],
    },
    CamEntry {
        id: "volcano-kilauea-cam-a",
        name: "Kilauea Volcano Livestream Cam A",
        video_id: "iws3rh5vLAQ",
        group: "Volcanoes",
        tags: &["volcano", "hawaii", "24/7"],
    },
    CamEntry {
        id: "volcano-etna-afartv",
        name: "Mount Etna, Sicily (HD)",
        video_id: "zy4u-v0AjOg",
        group: "Volcanoes",
        tags: &["volcano", "italy", "hd", "24/7"],
    },
    CamEntry {
        id: "volcano-mayon-afartv",
        name: "Mayon Volcano, Philippines (4K)",
        video_id: "X3u0DNtaSNY",
        group: "Volcanoes",
        tags: &["volcano", "philippines", "4k", "24/7"],
    },
    CamEntry {
        id: "volcano-popocatepetl-afartv",
        name: "Popocatepetl, Mexico (4K)",
        video_id: "FjqEaGzRMKc",
        group: "Volcanoes",
        tags: &["volcano", "mexico", "4k", "24/7"],
    },
    CamEntry {
        id: "volcano-iceland-afartv",
        name: "Iceland Volcanoes & Auroras (4K)",
        video_id: "e1sA7WeFckU",
        group: "Volcanoes",
        tags: &["volcano", "iceland", "4k", "24/7"],
    },
    CamEntry {
        id: "volcano-semeru-afartv",
        name: "Semeru Volcano, Java (4K)",
        video_id: "5L-pCGOAa_4",
        group: "Volcanoes",
        tags: &["volcano", "indonesia", "4k", "24/7"],
    },
    CamEntry {
        id: "volcano-merapi-afartv",
        name: "Merapi Volcano, Indonesia (4K)",
        video_id: "CAPq0pLfMaI",
        group: "Volcanoes",
        tags: &["volcano", "indonesia", "4k", "24/7"],
    },
    CamEntry {
        id: "volcano-santa-maria-afartv",
        name: "Santa Maria Volcano, Guatemala (4K)",
        video_id: "4MkmM-JY1JA",
        group: "Volcanoes",
        tags: &["volcano", "guatemala", "4k", "24/7"],
    },
    CamEntry {
        id: "volcano-fuego-afartv",
        name: "Fuego Volcano, Guatemala (4K)",
        video_id: "LMP58dcp1Wg",
        group: "Volcanoes",
        tags: &["volcano", "guatemala", "4k", "24/7"],
    },
    // ── Beach & Surf ────────────────────────────────────────
    CamEntry {
        id: "surf-pipeline-explore",
        name: "Pipeline Cam, North Shore, Oahu",
        video_id: "DY5RYp4sxYc",
        group: "Beach & Surf",
        tags: &["surf", "hawaii", "24/7"],
    },
    CamEntry {
        id: "surf-waimea-explore",
        name: "Waimea Bay Cam, Oahu",
        video_id: "wnNrd-VjLsQ",
        group: "Beach & Surf",
        tags: &["surf", "hawaii", "24/7"],
    },
    CamEntry {
        id: "surf-whale-sanctuary",
        name: "Humpback Whale Sanctuary, Maui",
        video_id: "6i0yI_pfg7k",
        group: "Beach & Surf",
        tags: &["ocean", "hawaii", "24/7"],
    },
    CamEntry {
        id: "surf-soggy-dollar",
        name: "Soggy Dollar Bar, BVI",
        video_id: "LXWVYoBluT4",
        group: "Beach & Surf",
        tags: &["beach", "caribbean", "24/7"],
    },
    CamEntry {
        id: "surf-santa-monica",
        name: "Santa Monica Beach Cam",
        video_id: "OWbI6WtlI-k",
        group: "Beach & Surf",
        tags: &["beach", "california", "24/7"],
    },
    CamEntry {
        id: "surf-pacifica-pier",
        name: "Pacifica Pier and Beach (4K)",
        video_id: "zYi_5AF6B2A",
        group: "Beach & Surf",
        tags: &["beach", "california", "4k", "24/7"],
    },
    CamEntry {
        id: "surf-30a-florida",
        name: "30A Beach Cam, Santa Rosa Beach, FL",
        video_id: "ftGfQqCA184",
        group: "Beach & Surf",
        tags: &["beach", "florida", "24/7"],
    },
    CamEntry {
        id: "surf-hamptons",
        name: "Hamptons Main Beach, East Hampton, NY",
        video_id: "Ba2cLC3xUpU",
        group: "Beach & Surf",
        tags: &["beach", "new-york", "24/7"],
    },
    CamEntry {
        id: "surf-glass-beach",
        name: "Glass Beach, Fort Bragg, CA",
        video_id: "rxBBRLWF0mM",
        group: "Beach & Surf",
        tags: &["beach", "california", "24/7"],
    },
    CamEntry {
        id: "surf-ambergris-belize",
        name: "Ambergris Caye, Belize",
        video_id: "r_XFhbOQ-Jo",
        group: "Beach & Surf",
        tags: &["beach", "belize", "24/7"],
    },
    CamEntry {
        id: "surf-calichi-usvi",
        name: "Picture Point, St. John USVI",
        video_id: "m7c12NY6xok",
        group: "Beach & Surf",
        tags: &["beach", "caribbean", "24/7"],
    },
    // ── Ski Resorts ─────────────────────────────────────────
    CamEntry {
        id: "ski-panorama",
        name: "Ski Panorama (200 webcams, 8 countries)",
        video_id: "HVt5n0CDRF8",
        group: "Ski Resorts",
        tags: &["ski", "europe", "24/7"],
    },
    CamEntry {
        id: "ski-grouse-mountain",
        name: "Grouse Mountain (4K), Vancouver",
        video_id: "-XM7S9nm9js",
        group: "Ski Resorts",
        tags: &["ski", "canada", "4k"],
    },
    CamEntry {
        id: "ski-new-england",
        name: "Ski Resort Webcams, New England",
        video_id: "oRyJBAIOto0",
        group: "Ski Resorts",
        tags: &["ski", "new-england"],
    },
    CamEntry {
        id: "ski-palisades-tahoe",
        name: "Palisades Tahoe Live",
        video_id: "8xEgLRLR7u0",
        group: "Ski Resorts",
        tags: &["ski", "california"],
    },
    CamEntry {
        id: "ski-mount-washington",
        name: "Mount Washington Alpine Resort",
        video_id: "k7loUrt8HkM",
        group: "Ski Resorts",
        tags: &["ski", "canada"],
    },
    CamEntry {
        id: "ski-sundown",
        name: "Ski Sundown Live Webcam",
        video_id: "2zVEuh_7rKk",
        group: "Ski Resorts",
        tags: &["ski", "connecticut"],
    },
    CamEntry {
        id: "ski-transalpina",
        name: "Transalpina Ski Resort, Romania",
        video_id: "1t9RkU0khvo",
        group: "Ski Resorts",
        tags: &["ski", "romania", "europe"],
    },
    CamEntry {
        id: "ski-pomerelle",
        name: "Pomerelle Mountain Ski Resort",
        video_id: "uM-oftYVFGA",
        group: "Ski Resorts",
        tags: &["ski", "idaho"],
    },
];

// ============================================================
// Stream builder (public for tests)
// ============================================================

pub fn build_streams() -> Vec<Stream> {
    CAMS.iter()
        .map(|cam| Stream {
            id: cam.id.to_string(),
            name: cam.name.to_string(),
            url: format!("https://www.youtube.com/watch?v={}", cam.video_id),
            group: cam.group.to_string(),
            logo: None,
            vod_type: "movie".to_string(),
            tags: Some(cam.tags.iter().map(|t| t.to_string()).collect()),
        })
        .collect()
}

/// Return all unique group names in catalog order.
pub fn get_groups() -> Vec<String> {
    let mut groups: Vec<String> = Vec::new();
    for cam in CAMS {
        let g = cam.group.to_string();
        if !groups.contains(&g) {
            groups.push(g);
        }
    }
    groups
}

/// Return total number of cams in the catalog.
pub fn cam_count() -> usize {
    CAMS.len()
}

// ============================================================
// Plugin exports
// ============================================================

#[no_mangle]
pub extern "C" fn describe() -> u64 {
    let desc = Descriptor {
        r#type: "outdoorcams",
        label: "Outdoor Cams",
        short_label: "CAMS",
        color: "#43a047",
        version: "0.1.0",
        description: "Live 24/7 video streams of volcanoes, surf/beach cams, and ski resort webcams from YouTube",
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

    log_info(&format!("outdoorcams: returning {} streams", streams.len()));

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
