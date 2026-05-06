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

#[cfg(test)]
fn http_get(_url: &str) -> Option<Vec<u8>> {
    // Stub for tests — returns None so refresh/interact return empty results.
    // Unit tests exercise parsing/mapping logic directly, not HTTP calls.
    None
}

#[cfg(not(test))]
fn http_get(url: &str) -> Option<Vec<u8>> {
    let url_bytes = url.as_bytes();
    let method = b"GET";
    let headers = b"{\"User-Agent\":\"MediaHub/1.0 radiobrowser-plugin\"}";

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

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct RadioStation {
    pub stationuuid: Option<String>,
    pub name: Option<String>,
    pub url_resolved: Option<String>,
    pub url: Option<String>,
    pub favicon: Option<String>,
    pub tags: Option<String>,
    pub country: Option<String>,
    pub codec: Option<String>,
    pub bitrate: Option<u32>,
}

// ============================================================
// Station mapping logic (shared with tests)
// ============================================================

/// Split a comma-separated tag string into a deduplicated, trimmed Vec of non-empty tags.
pub(crate) fn split_tags(raw: &str) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut result = Vec::new();
    for tag in raw.split(',') {
        let trimmed = tag.trim().to_lowercase();
        if !trimmed.is_empty() && seen.insert(trimmed.clone()) {
            result.push(trimmed);
        }
    }
    result
}

/// Convert a RadioStation into a Stream, using the given group name.
pub(crate) fn station_to_stream(station: &RadioStation, group: &str) -> Option<Stream> {
    let id = station.stationuuid.as_deref().unwrap_or("").to_string();
    if id.is_empty() {
        return None;
    }

    let name = station.name.as_deref().unwrap_or("Unknown Station").to_string();

    // Prefer url_resolved, fall back to url
    let url = station
        .url_resolved
        .as_deref()
        .filter(|u| !u.is_empty())
        .or(station.url.as_deref().filter(|u| !u.is_empty()))
        .unwrap_or("")
        .to_string();

    if url.is_empty() {
        return None;
    }

    let logo = station.favicon.as_deref().unwrap_or("").to_string();
    let tags = split_tags(station.tags.as_deref().unwrap_or(""));

    Some(Stream {
        id,
        name,
        url,
        group: group.to_string(),
        logo,
        vod_type: String::new(),
        tags,
    })
}

/// Parse a JSON response body into a Vec of RadioStation.
pub(crate) fn parse_stations(data: &[u8]) -> Vec<RadioStation> {
    serde_json::from_slice(data).unwrap_or_default()
}

/// Collect stations into streams, deduplicating by stationuuid.
pub(crate) fn collect_streams(
    stations: &[RadioStation],
    group: &str,
    seen: &mut HashSet<String>,
) -> Vec<Stream> {
    let mut result = Vec::new();
    for station in stations {
        if let Some(stream) = station_to_stream(station, group) {
            if seen.insert(stream.id.clone()) {
                result.push(stream);
            }
        }
    }
    result
}

// ============================================================
// Plugin exports
// ============================================================

#[no_mangle]
pub extern "C" fn describe() -> u64 {
    let desc = Descriptor {
        r#type: "radiobrowser",
        label: "Radio Browser",
        short_label: "RADIO",
        color: "#ff9800",
        version: "1.0.0",
        description: "Browse 90,000+ internet radio stations worldwide from the community-driven Radio Browser directory",
        config_fields: vec![
            serde_json::json!({
                "key": "mode",
                "label": "Browse by",
                "type": "select",
                "required": true,
                "default": "tag",
                "options": [
                    { "value": "tag", "label": "Genre / Tag" },
                    { "value": "country", "label": "Country" }
                ]
            }),
            serde_json::json!({
                "key": "tags",
                "label": "Genres",
                "type": "text",
                "required": false,
                "default": "jazz,rock,classical",
                "description": "Comma-separated list of genre tags (used when mode is 'tag')"
            }),
            serde_json::json!({
                "key": "countries",
                "label": "Countries",
                "type": "text",
                "required": false,
                "default": "",
                "description": "Comma-separated list of countries (used when mode is 'country')"
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
                "id": "search_stations",
                "label": "Search Stations",
                "type": "search",
                "params": [
                    {
                        "key": "query",
                        "label": "Station name",
                        "type": "text",
                        "required": true
                    }
                ]
            }),
        ],
    };
    return_json(&desc)
}

/// URL-encode a string for use in query parameters and path segments.
fn url_encode(input: &str) -> String {
    let mut encoded = String::new();
    for byte in input.as_bytes() {
        match *byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(*byte as char);
            }
            b' ' => encoded.push_str("%20"),
            _ => {
                encoded.push_str(&format!("%{:02X}", byte));
            }
        }
    }
    encoded
}

const BASE_URL: &str = "https://de1.api.radio-browser.info";

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

    let mode = config
        .get("mode")
        .and_then(|v| v.as_str())
        .unwrap_or("tag");

    let mut seen = HashSet::new();
    let mut streams: Vec<Stream> = Vec::new();

    match mode {
        "tag" => {
            let tags_raw = config
                .get("tags")
                .and_then(|v| v.as_str())
                .unwrap_or("jazz,rock,classical");

            for tag in tags_raw.split(',') {
                let tag = tag.trim();
                if tag.is_empty() {
                    continue;
                }

                let url = format!(
                    "{}/json/stations/bytag/{}?limit=100&order=votes&reverse=true&lastcheckok=1",
                    BASE_URL,
                    url_encode(tag)
                );

                log_info(&format!("fetching stations for tag: {}", tag));

                let body = match http_get(&url) {
                    Some(b) => b,
                    None => {
                        log_error(&format!("failed to fetch stations for tag: {}", tag));
                        continue;
                    }
                };

                let stations = parse_stations(&body);
                let new_streams = collect_streams(&stations, tag, &mut seen);
                log_info(&format!("got {} stations for tag '{}'", new_streams.len(), tag));
                streams.extend(new_streams);
            }
        }
        "country" => {
            let countries_raw = config
                .get("countries")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            if countries_raw.is_empty() {
                log_info("no countries configured");
                return return_json(&RefreshResponse { streams: vec![] });
            }

            for country in countries_raw.split(',') {
                let country = country.trim();
                if country.is_empty() {
                    continue;
                }

                let url = format!(
                    "{}/json/stations/bycountry/{}?limit=100&order=votes&reverse=true&lastcheckok=1",
                    BASE_URL,
                    url_encode(country)
                );

                log_info(&format!("fetching stations for country: {}", country));

                let body = match http_get(&url) {
                    Some(b) => b,
                    None => {
                        log_error(&format!("failed to fetch stations for country: {}", country));
                        continue;
                    }
                };

                let stations = parse_stations(&body);
                let new_streams = collect_streams(&stations, country, &mut seen);
                log_info(&format!("got {} stations for country '{}'", new_streams.len(), country));
                streams.extend(new_streams);
            }
        }
        _ => {
            log_error(&format!("unknown mode: {}", mode));
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

    if req.action != "search_stations" {
        log_error(&format!("unknown action: {}", req.action));
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

    let url = format!(
        "{}/json/stations/search?name={}&limit=50&lastcheckok=1",
        BASE_URL,
        url_encode(query)
    );

    log_info(&format!("searching stations: {}", query));

    let body = match http_get(&url) {
        Some(b) => b,
        None => {
            log_error("failed to search stations");
            let empty: Vec<Value> = vec![];
            return return_json(&serde_json::json!({ "results": empty }));
        }
    };

    let stations = parse_stations(&body);
    let mut seen = HashSet::new();
    let streams = collect_streams(&stations, "Search Results", &mut seen);

    return_json(&serde_json::json!({ "results": streams }))
}
