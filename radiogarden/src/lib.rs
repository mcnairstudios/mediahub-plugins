use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;
use std::slice;

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

#[derive(Serialize)]
struct Stream {
    id: String,
    name: String,
    url: String,
    group: String,
    logo: String,
    vod_type: String,
    tags: Vec<String>,
}

#[derive(Serialize)]
struct RefreshResponse {
    streams: Vec<Stream>,
}

#[derive(Deserialize)]
struct ConfigPlace {
    id: String,
    name: String,
}

#[derive(Deserialize)]
struct ChannelPage {
    url: String,
    title: String,
}

#[derive(Deserialize)]
struct ChannelItem {
    page: ChannelPage,
}

#[derive(Deserialize)]
struct ContentSection {
    items: Vec<ChannelItem>,
}

#[derive(Deserialize)]
struct ChannelsData {
    content: Vec<ContentSection>,
}

#[derive(Deserialize)]
struct ChannelsResponse {
    data: ChannelsData,
}

#[derive(Serialize, Deserialize, Clone)]
struct PlaceEntry {
    id: String,
    title: String,
    country: String,
    size: i64,
}

#[derive(Deserialize)]
struct PlacesListData {
    list: Vec<PlaceEntry>,
}

#[derive(Deserialize)]
struct PlacesResponse {
    data: PlacesListData,
}

#[derive(Serialize)]
struct SearchResult {
    id: String,
    title: String,
    subtitle: String,
}

// ============================================================
// Helpers
// ============================================================

/// Extract the last path segment from a URL like "/listen/bbc-radio-1/hYpXtjOZ".
fn extract_channel_id(url_path: &str) -> Option<&str> {
    let idx = url_path.rfind('/')?;
    let id = &url_path[idx + 1..];
    if id.is_empty() {
        None
    } else {
        Some(id)
    }
}

/// Load places list, using KV cache when available.
fn load_places() -> Option<Vec<PlaceEntry>> {
    // Try cache first
    if let Some(cached) = kv_get("places_cache") {
        if let Ok(places) = serde_json::from_str::<Vec<PlaceEntry>>(&cached) {
            if !places.is_empty() {
                return Some(places);
            }
        }
    }

    log_info("fetching places list from Radio Garden API");
    let body = match http_get("https://radio.garden/api/ara/content/places") {
        Some(b) => b,
        None => {
            log_error("failed to fetch places");
            return None;
        }
    };

    let resp: PlacesResponse = match serde_json::from_slice(&body) {
        Ok(r) => r,
        Err(_) => {
            log_error("failed to parse places");
            return None;
        }
    };

    log_info(&format!("fetched {} places, caching", resp.data.list.len()));

    if let Ok(cache_data) = serde_json::to_string(&resp.data.list) {
        kv_set("places_cache", &cache_data);
    }

    Some(resp.data.list)
}

// ============================================================
// Plugin exports
// ============================================================

#[no_mangle]
pub extern "C" fn describe() -> u64 {
    let desc = Descriptor {
        r#type: "radiogarden",
        label: "Radio Garden",
        short_label: "RADIO",
        color: "#43a047",
        version: "1.0.0",
        description: "Live radio streams from Radio Garden",
        config_fields: vec![
            serde_json::json!({
                "key": "places",
                "label": "Locations",
                "type": "custom",
                "component": "place-picker"
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
                "id": "search_places",
                "label": "Search Location",
                "type": "search",
                "target_field": "places"
            }),
        ],
    };
    return_json(&desc)
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

    let places: Vec<ConfigPlace> = if let Some(raw) = config.get("places") {
        // places may be a JSON string (stringified array) or a direct array.
        if let Some(s) = raw.as_str() {
            // It was a string — parse the inner JSON.
            serde_json::from_str(s).unwrap_or_default()
        } else {
            serde_json::from_value(raw.clone()).unwrap_or_default()
        }
    } else {
        vec![]
    };

    if places.is_empty() {
        log_info("no places configured");
        return return_json(&RefreshResponse { streams: vec![] });
    }

    let mut seen = HashSet::new();
    let mut streams: Vec<Stream> = Vec::new();

    for place in &places {
        let url = format!(
            "https://radio.garden/api/ara/content/page/{}/channels",
            place.id
        );
        let body = match http_get(&url) {
            Some(b) => b,
            None => {
                log_error(&format!("failed to fetch channels for {}", place.name));
                continue;
            }
        };

        let resp: ChannelsResponse = match serde_json::from_slice(&body) {
            Ok(r) => r,
            Err(e) => {
                log_error(&format!(
                    "failed to parse channels for {}: {}",
                    place.name, e
                ));
                continue;
            }
        };

        for section in &resp.data.content {
            for item in &section.items {
                let channel_id = match extract_channel_id(&item.page.url) {
                    Some(id) => id,
                    None => continue,
                };
                if !seen.insert(channel_id.to_string()) {
                    continue;
                }

                streams.push(Stream {
                    id: channel_id.to_string(),
                    name: item.page.title.clone(),
                    url: format!(
                        "https://radio.garden/api/ara/content/listen/{}/channel.mp3",
                        channel_id
                    ),
                    group: place.name.clone(),
                    logo: String::new(),
                    vod_type: String::new(),
                    tags: vec![],
                });
            }
        }

        log_info(&format!(
            "fetched {} channels for {}",
            resp.data.content.len(),
            place.name
        ));
    }

    log_info(&format!(
        "refresh complete: {} streams from {} places",
        streams.len(),
        places.len()
    ));
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

    if req.action != "search_places" {
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

    let places = match load_places() {
        Some(p) => p,
        None => {
            let empty: Vec<Value> = vec![];
            return return_json(&serde_json::json!({ "results": empty }));
        }
    };

    let query_lower = query.to_lowercase();
    let mut results: Vec<SearchResult> = Vec::new();

    for p in &places {
        if results.len() >= 20 {
            break;
        }
        let title_lower = p.title.to_lowercase();
        let country_lower = p.country.to_lowercase();
        if title_lower.contains(&query_lower) || country_lower.contains(&query_lower) {
            results.push(SearchResult {
                id: p.id.clone(),
                title: format!("{}, {}", p.title, p.country),
                subtitle: format!("{} stations", p.size),
            });
        }
    }

    return_json(&serde_json::json!({ "results": results }))
}
