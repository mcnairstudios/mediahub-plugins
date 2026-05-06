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
struct RefreshResponse {
    streams: Vec<Stream>,
}

#[derive(Serialize, Clone, Debug, PartialEq)]
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
// Curated Slow TV video catalog
// ============================================================

struct VideoEntry {
    id: &'static str,
    title: &'static str,
    group: &'static str,
    duration: &'static str,
}

const CURATED_VIDEOS: &[VideoEntry] = &[
    // --- Train Journeys ---
    VideoEntry {
        id: "3rDjPLvOShM",
        title: "Nordlandsbanen: Train to the Arctic Circle (Winter)",
        group: "Train Journeys",
        duration: "10h",
    },
    VideoEntry {
        id: "cNiN7gOcNI4",
        title: "Nordlandsbanen: Train to the Arctic Circle (Spring)",
        group: "Train Journeys",
        duration: "10h",
    },
    VideoEntry {
        id: "yCtt26c_AOg",
        title: "Nordlandsbanen: Train to the Arctic Circle (Summer)",
        group: "Train Journeys",
        duration: "10h",
    },
    VideoEntry {
        id: "hXMtC8Kj-sQ",
        title: "Norwegian Slow TV Train Compilation",
        group: "Train Journeys",
        duration: "7h",
    },
    VideoEntry {
        id: "xisVS_DKpJg",
        title: "Bergen Railway: Oslo to Bergen in Winter",
        group: "Train Journeys",
        duration: "7h",
    },
    VideoEntry {
        id: "z7VYVjR_nwE",
        title: "Flamsbana: Mountain Railway Cab Ride",
        group: "Train Journeys",
        duration: "1h",
    },
    VideoEntry {
        id: "DjKJiDMhAPo",
        title: "Dovre Railway: Oslo to Trondheim",
        group: "Train Journeys",
        duration: "7h",
    },
    VideoEntry {
        id: "2e8TFCluACk",
        title: "Swiss Alps Train Journey: Bernina Express",
        group: "Train Journeys",
        duration: "4h",
    },
    // --- Boat & Ship ---
    VideoEntry {
        id: "2uEe0Y1pBSs",
        title: "Hurtigruten: Norwegian Coastal Voyage (Full)",
        group: "Boat & Ship",
        duration: "5h",
    },
    VideoEntry {
        id: "GQm_HA1CJfA",
        title: "Hurtigruten Minute by Minute",
        group: "Boat & Ship",
        duration: "5h",
    },
    VideoEntry {
        id: "tkOaxSsWJBk",
        title: "Sailing Through Norwegian Fjords",
        group: "Boat & Ship",
        duration: "3h",
    },
    VideoEntry {
        id: "ShZTkKFflGo",
        title: "Canal Boat Journey Through England",
        group: "Boat & Ship",
        duration: "2h",
    },
    // --- Fireplace & Cabin ---
    VideoEntry {
        id: "L_LUpnjgPso",
        title: "Crackling Fireplace with Christmas Music",
        group: "Fireplace & Cabin",
        duration: "10h",
    },
    VideoEntry {
        id: "UgHKb_7884o",
        title: "Birchwood Fireplace Burning (No Music)",
        group: "Fireplace & Cabin",
        duration: "8h",
    },
    VideoEntry {
        id: "eyU3bRy2x44",
        title: "Cozy Cabin Fireplace with Snow Outside",
        group: "Fireplace & Cabin",
        duration: "10h",
    },
    VideoEntry {
        id: "RDfjXj5EGqI",
        title: "Crackling Fireplace with Rain Sounds",
        group: "Fireplace & Cabin",
        duration: "8h",
    },
    // --- Nature ---
    VideoEntry {
        id: "qsOUv9EzKsg",
        title: "Northern Lights in Real Time (Norway)",
        group: "Nature",
        duration: "5h",
    },
    VideoEntry {
        id: "xNN7iTA57jM",
        title: "Autumn Forest Walk with Nature Sounds",
        group: "Nature",
        duration: "3h",
    },
    VideoEntry {
        id: "CGyEd0aKWZE",
        title: "Spring in the Norwegian Mountains",
        group: "Nature",
        duration: "6h",
    },
    VideoEntry {
        id: "V-_O7nl0Ii0",
        title: "Rain on a Tent in the Forest",
        group: "Nature",
        duration: "8h",
    },
    VideoEntry {
        id: "Nep1qytq9JM",
        title: "Snowfall in a Winter Forest",
        group: "Nature",
        duration: "6h",
    },
    // --- Ocean ---
    VideoEntry {
        id: "WHPEKLQID4U",
        title: "Ocean Waves Crashing on Rocky Shore",
        group: "Ocean",
        duration: "8h",
    },
    VideoEntry {
        id: "bn9F19Hi1Lk",
        title: "Calm Ocean Waves at Sunset Beach",
        group: "Ocean",
        duration: "10h",
    },
    VideoEntry {
        id: "f77SKdyn-1Y",
        title: "Tropical Beach Waves with Sea Birds",
        group: "Ocean",
        duration: "6h",
    },
    // --- City Walks ---
    VideoEntry {
        id: "rx6w3j7cWa0",
        title: "Walking Tokyo at Night in the Rain",
        group: "City Walks",
        duration: "1h",
    },
    VideoEntry {
        id: "n3Dru5y3ROc",
        title: "New York City Walk: Manhattan to Brooklyn",
        group: "City Walks",
        duration: "4h",
    },
    VideoEntry {
        id: "5BIylSbIBCo",
        title: "Paris Walking Tour: Eiffel Tower to Montmartre",
        group: "City Walks",
        duration: "2h",
    },
    VideoEntry {
        id: "HDhJbJJTJ5E",
        title: "London Walk: Covent Garden to Tower Bridge",
        group: "City Walks",
        duration: "2h",
    },
];

fn youtube_url(video_id: &str) -> String {
    format!("https://www.youtube.com/watch?v={}", video_id)
}

fn youtube_thumbnail(video_id: &str) -> String {
    format!("https://img.youtube.com/vi/{}/hqdefault.jpg", video_id)
}

fn build_streams() -> Vec<Stream> {
    CURATED_VIDEOS
        .iter()
        .map(|v| Stream {
            id: format!("slowtv-{}", v.id),
            name: format!("{} ({})", v.title, v.duration),
            url: youtube_url(v.id),
            group: v.group.to_string(),
            logo: Some(youtube_thumbnail(v.id)),
            vod_type: "movie".to_string(),
            year: None,
            tags: Some(vec!["slow-tv".to_string()]),
            episode_name: None,
        })
        .collect()
}

// ============================================================
// Plugin exports
// ============================================================

#[no_mangle]
pub extern "C" fn describe() -> u64 {
    let desc = Descriptor {
        r#type: "slowtv",
        label: "Slow TV",
        short_label: "SLOW",
        color: "#2e7d32",
        version: "1.0.0",
        description: "Curated long-form ambient videos: train rides, fireplaces, nature walks, ocean waves, and city strolls",
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

    log_info(&format!("slowtv: returning {} streams", streams.len()));

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
