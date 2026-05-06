use serde::{Deserialize, Serialize};
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

#[derive(Serialize, Deserialize, Clone, Debug)]
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
// Caltrans district definitions
// ============================================================

pub struct DistrictInfo {
    pub number: u8,
    pub label: &'static str,
}

pub const DISTRICTS: &[DistrictInfo] = &[
    DistrictInfo { number: 1, label: "D1 - Northwest" },
    DistrictInfo { number: 2, label: "D2 - Northeast" },
    DistrictInfo { number: 3, label: "D3 - Sacramento" },
    DistrictInfo { number: 4, label: "D4 - Bay Area" },
    DistrictInfo { number: 5, label: "D5 - Central Coast" },
    DistrictInfo { number: 6, label: "D6 - Fresno" },
    DistrictInfo { number: 7, label: "D7 - Los Angeles" },
    DistrictInfo { number: 8, label: "D8 - San Bernardino" },
    DistrictInfo { number: 9, label: "D9 - Bishop" },
    DistrictInfo { number: 10, label: "D10 - Stockton" },
    DistrictInfo { number: 11, label: "D11 - San Diego" },
    DistrictInfo { number: 12, label: "D12 - Orange County" },
];

/// Build the Caltrans CCTV API URL for a given district number.
/// District numbers < 10 use single digit in the path but zero-padded in the filename.
/// e.g. district 3 -> /data/d3/cctv/cctvStatusD03.json
///      district 11 -> /data/d11/cctv/cctvStatusD11.json
pub fn district_api_url(district_num: u8) -> String {
    format!(
        "https://cwwp2.dot.ca.gov/data/d{}/cctv/cctvStatusD{:02}.json",
        district_num, district_num
    )
}

/// Find the district label for a given district number.
pub fn district_label(district_num: u8) -> &'static str {
    for d in DISTRICTS {
        if d.number == district_num {
            return d.label;
        }
    }
    "Unknown District"
}

// ============================================================
// Camera JSON parsing
// ============================================================

/// Parse a single camera entry from the Caltrans CCTV JSON.
/// Returns None if the camera has no streaming video URL or is not in service.
pub fn parse_camera(cam: &Value, district_num: u8) -> Option<Stream> {
    let in_service = cam.get("inService")
        .and_then(|v| v.as_str())
        .unwrap_or("FALSE");
    if in_service != "TRUE" {
        return None;
    }

    let stream_url = cam.get("streamingVideoURL")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    if stream_url.is_empty() {
        return None;
    }

    let cctv_id = cam.get("index")
        .and_then(|v| v.as_str())
        .or_else(|| cam.get("cctv-id").and_then(|v| v.as_str()))
        .unwrap_or("");

    let location_name = cam.get("location")
        .and_then(|v| v.get("locationName"))
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let nearby_place = cam.get("location")
        .and_then(|v| v.get("nearbyPlace"))
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let county = cam.get("location")
        .and_then(|v| v.get("county"))
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let route = cam.get("location")
        .and_then(|v| v.get("route"))
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let direction = cam.get("location")
        .and_then(|v| v.get("direction"))
        .and_then(|v| v.as_str())
        .unwrap_or("");

    // Build a descriptive name
    let name = build_camera_name(route, location_name, nearby_place, direction);

    // Build the stream ID
    let id = format!("caltrans-d{}-{}", district_num, cctv_id);

    // Group by district label
    let group = district_label(district_num).to_string();

    // Build tags for search
    let mut tags: Vec<String> = vec!["california".to_string()];
    if !route.is_empty() {
        tags.push(route.to_lowercase());
    }
    if !county.is_empty() {
        tags.push(county.to_lowercase());
    }
    if !nearby_place.is_empty() {
        tags.push(nearby_place.to_lowercase());
    }
    if !direction.is_empty() {
        tags.push(direction.to_lowercase());
    }

    Some(Stream {
        id,
        name,
        url: stream_url.to_string(),
        group,
        logo: None,
        vod_type: "live".to_string(),
        tags: Some(tags),
    })
}

/// Build a human-readable camera name from route, location, nearby place, and direction.
pub fn build_camera_name(route: &str, location_name: &str, nearby_place: &str, direction: &str) -> String {
    let mut parts: Vec<String> = Vec::new();

    if !route.is_empty() {
        parts.push(format!("{}", route));
    }

    if !location_name.is_empty() {
        parts.push(format!("at {}", location_name));
    } else if !nearby_place.is_empty() {
        parts.push(format!("near {}", nearby_place));
    }

    if !direction.is_empty() {
        parts.push(format!("({})", direction));
    }

    if parts.is_empty() {
        "Unknown Camera".to_string()
    } else {
        parts.join(" ")
    }
}

/// Parse the Caltrans CCTV JSON response for a district.
/// The JSON structure has a top-level "data" key containing an array of camera objects,
/// or may directly be an array. We handle both.
pub fn parse_district_cameras(body: &[u8], district_num: u8) -> Vec<Stream> {
    let json: Value = match serde_json::from_slice(body) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };

    let cameras = if let Some(data) = json.get("data") {
        data
    } else {
        &json
    };

    let cam_array = match cameras.as_array() {
        Some(arr) => arr,
        None => return Vec::new(),
    };

    let mut streams = Vec::new();
    for cam_wrapper in cam_array {
        // Each entry may have a "cctv" sub-object or be the camera directly
        let cam = if let Some(cctv) = cam_wrapper.get("cctv") {
            cctv
        } else {
            cam_wrapper
        };

        if let Some(stream) = parse_camera(cam, district_num) {
            streams.push(stream);
        }
    }

    streams
}

/// Get the list of selected district numbers from config, or return all districts.
fn get_selected_districts(config: &serde_json::Map<String, Value>) -> Vec<u8> {
    if let Some(raw) = config.get("districts") {
        let district_strs: Vec<String> = if let Some(s) = raw.as_str() {
            // Could be a JSON-encoded string array
            serde_json::from_str(s).unwrap_or_default()
        } else if let Some(arr) = raw.as_array() {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        } else {
            Vec::new()
        };

        if !district_strs.is_empty() {
            return district_strs
                .iter()
                .filter_map(|s| s.parse::<u8>().ok())
                .filter(|n| DISTRICTS.iter().any(|d| d.number == *n))
                .collect();
        }
    }

    // Default: all districts
    DISTRICTS.iter().map(|d| d.number).collect()
}

// ============================================================
// Interact: search cameras
// ============================================================

#[derive(Serialize)]
struct SearchResult {
    id: String,
    title: String,
    subtitle: String,
}

/// Search cached camera data by query string (matches route, location, county, tags).
fn search_cameras(query: &str) -> Vec<SearchResult> {
    let cached = match kv_get("cameras_cache") {
        Some(c) => c,
        None => return Vec::new(),
    };

    let streams: Vec<Stream> = match serde_json::from_str(&cached) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    let query_lower = query.to_lowercase();
    let mut results: Vec<SearchResult> = Vec::new();

    for stream in &streams {
        if results.len() >= 30 {
            break;
        }

        let name_lower = stream.name.to_lowercase();
        let group_lower = stream.group.to_lowercase();
        let tags_match = stream.tags.as_ref().map_or(false, |tags| tags.iter().any(|t| t.contains(&query_lower)));

        if name_lower.contains(&query_lower) || group_lower.contains(&query_lower) || tags_match {
            results.push(SearchResult {
                id: stream.id.clone(),
                title: stream.name.clone(),
                subtitle: stream.group.clone(),
            });
        }
    }

    results
}

// ============================================================
// Plugin exports
// ============================================================

#[no_mangle]
pub extern "C" fn describe() -> u64 {
    let district_options: Vec<Value> = DISTRICTS
        .iter()
        .map(|d| {
            serde_json::json!({
                "value": d.number.to_string(),
                "label": d.label
            })
        })
        .collect();

    let desc = Descriptor {
        r#type: "trafficcams",
        label: "Traffic Cameras",
        short_label: "TRAFFIC",
        color: "#ef6c00",
        version: "1.0.0",
        description: "Live California highway traffic cameras from Caltrans CCTV feeds",
        config_fields: vec![
            serde_json::json!({
                "key": "districts",
                "label": "Caltrans Districts",
                "type": "multi-select",
                "options": district_options,
                "default": DISTRICTS.iter().map(|d| d.number.to_string()).collect::<Vec<_>>()
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
                "id": "search_cameras",
                "label": "Search Cameras",
                "type": "search",
                "target_field": "cameras"
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

    let selected_districts = get_selected_districts(&config);
    log_info(&format!("refreshing {} districts: {:?}", selected_districts.len(), selected_districts));

    // Check KV cache first
    let cache_key = format!("cameras_d{}", {
        let mut sorted = selected_districts.clone();
        sorted.sort();
        sorted.iter().map(|n| n.to_string()).collect::<Vec<_>>().join("_")
    });

    if let Some(cached) = kv_get(&cache_key) {
        if let Ok(streams) = serde_json::from_str::<Vec<Stream>>(&cached) {
            if !streams.is_empty() {
                log_info(&format!("returning {} cached streams", streams.len()));
                // Also update the search cache
                kv_set("cameras_cache", &cached);
                return return_json(&RefreshResponse { streams });
            }
        }
    }

    let mut all_streams: Vec<Stream> = Vec::new();

    for district_num in &selected_districts {
        let url = district_api_url(*district_num);
        log_info(&format!("fetching district {}: {}", district_num, url));

        let body = match http_get(&url) {
            Some(b) => b,
            None => {
                log_error(&format!("failed to fetch district {}", district_num));
                continue;
            }
        };

        let streams = parse_district_cameras(&body, *district_num);
        log_info(&format!("district {}: {} cameras with video", district_num, streams.len()));
        all_streams.extend(streams);
    }

    log_info(&format!("total cameras: {}", all_streams.len()));

    // Cache the results
    if let Ok(cache_data) = serde_json::to_string(&all_streams) {
        kv_set(&cache_key, &cache_data);
        kv_set("cameras_cache", &cache_data);
    }

    return_json(&RefreshResponse { streams: all_streams })
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

    if req.action != "search_cameras" {
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

    let results = search_cameras(query);
    return_json(&serde_json::json!({ "results": results }))
}
