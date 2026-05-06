use serde::Serialize;
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
pub struct RefreshResponse {
    pub streams: Vec<Stream>,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
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

// ============================================================
// Session data cached in KV
// ============================================================

/// Holds the data we need from the boot API response.
#[derive(Serialize, Debug, Clone)]
pub struct SessionData {
    pub session_token: String,
    pub stitcher_base: String,
    pub stitcher_params: String,
    pub active_region: String,
}

// ============================================================
// Parsing helpers (pub for testing)
// ============================================================

fn get_str(v: &Value, key: &str) -> String {
    v.get(key).and_then(|x| x.as_str()).unwrap_or("").to_string()
}

/// Parse the boot API response to extract session data.
pub fn parse_boot_response(body: &[u8]) -> Option<SessionData> {
    let root: Value = serde_json::from_slice(body).ok()?;

    let session_token = get_str(&root, "sessionToken");
    if session_token.is_empty() {
        log_error("boot response missing sessionToken");
        return None;
    }

    let stitcher_params = get_str(&root, "stitcherParams");

    let stitcher_base = root
        .get("servers")
        .and_then(|s| s.get("stitcher"))
        .and_then(|s| s.as_str())
        .unwrap_or("https://cfd-v4-service-channel-stitcher-use1-1.prd.pluto.tv")
        .to_string();

    let active_region = root
        .get("session")
        .and_then(|s| s.get("activeRegion"))
        .and_then(|s| s.as_str())
        .unwrap_or("unknown")
        .to_string();

    Some(SessionData {
        session_token,
        stitcher_base,
        stitcher_params,
        active_region,
    })
}

/// Parse the channel list API response into Stream objects.
/// Uses the session data to construct playable HLS URLs.
pub fn parse_channels(body: &[u8], session: &SessionData) -> Vec<Stream> {
    let channels: Vec<Value> = match serde_json::from_slice(body) {
        Ok(v) => v,
        Err(_) => {
            log_error("failed to parse channels JSON");
            return Vec::new();
        }
    };

    let mut streams: Vec<Stream> = Vec::new();
    let mut seen_ids: std::collections::HashSet<String> = std::collections::HashSet::new();

    for ch in &channels {
        let id = get_str(ch, "_id");
        let name = get_str(ch, "name");
        let category = get_str(ch, "category");
        let number = ch.get("number").and_then(|n| n.as_u64()).unwrap_or(0);

        if id.is_empty() || name.is_empty() {
            continue;
        }

        // Deduplicate by channel ID
        if !seen_ids.insert(id.clone()) {
            continue;
        }

        // Skip non-stitched channels (they don't have playable HLS streams)
        let is_stitched = ch.get("isStitched").and_then(|v| v.as_bool()).unwrap_or(false);
        if !is_stitched {
            continue;
        }

        // Skip internal/system channels
        let slug = get_str(ch, "slug");
        if slug.starts_with("announcement") || slug.starts_with("privacy-policy") {
            continue;
        }

        // Construct the playable HLS URL using stitcher base + channel ID + params + JWT
        let url = if session.stitcher_params.is_empty() {
            format!(
                "{}/v2/stitch/hls/channel/{}/master.m3u8?jwt={}&masterJWTPassthrough=true&includeExtendedEvents=true",
                session.stitcher_base, id, session.session_token
            )
        } else {
            format!(
                "{}/v2/stitch/hls/channel/{}/master.m3u8?{}&jwt={}&masterJWTPassthrough=true&includeExtendedEvents=true",
                session.stitcher_base, id, session.stitcher_params, session.session_token
            )
        };

        let logo = Some(format!(
            "https://images.pluto.tv/channels/{}/colorLogoPNG.png",
            id
        ));

        let group = if category.is_empty() {
            "Uncategorized".to_string()
        } else {
            category
        };

        let tags = if number > 0 {
            Some(vec![number.to_string()])
        } else {
            None
        };

        streams.push(Stream {
            id,
            name,
            url,
            group,
            logo,
            vod_type: String::new(), // live TV, not VOD
            tags,
        });
    }

    streams
}

/// Build the boot API URL with required query parameters.
/// Uses the provided device_id and client_id for session consistency.
fn build_boot_url(device_id: &str, client_id: &str) -> String {
    format!(
        "https://boot.pluto.tv/v4/start?\
         appName=web\
         &appVersion=8.0.0-111b2b9dc00bd0bea9030b30662159ed9e7c8bc6\
         &clientID={}\
         &clientModelNumber=1.0.0\
         &deviceDNT=0\
         &deviceId={}\
         &deviceMake=chrome\
         &deviceModel=web\
         &deviceType=web\
         &deviceVersion=122.0.0\
         &serverSideAds=false\
         &drmCapabilities=widevine%3AL3",
        client_id, device_id
    )
}

/// Get or generate a stable device ID from KV store.
#[cfg(not(test))]
fn get_or_create_device_id() -> String {
    if let Some(id) = kv_get("plutotv_device_id") {
        if !id.is_empty() {
            return id;
        }
    }
    // Generate a simple pseudo-unique ID based on a fixed seed.
    // In WASM we don't have access to random, so use a deterministic but
    // unique-looking string.
    let id = "plutotv-wasm-device-01a2b3c4d5e6".to_string();
    kv_set("plutotv_device_id", &id);
    id
}

/// Fetch session data: check KV cache first, then call boot API if needed.
#[cfg(not(test))]
fn get_session() -> Option<SessionData> {
    // Check KV cache for existing session
    if let Some(cached) = kv_get("plutotv_session") {
        if !cached.is_empty() {
            // Parse cached session JSON
            if let Ok(data) = serde_json::from_str::<Value>(&cached) {
                let session_token = get_str(&data, "session_token");
                let stitcher_base = get_str(&data, "stitcher_base");
                let stitcher_params = get_str(&data, "stitcher_params");
                let active_region = get_str(&data, "active_region");

                if !session_token.is_empty() && !stitcher_base.is_empty() {
                    log_info(&format!(
                        "using cached session (region: {})",
                        active_region
                    ));
                    return Some(SessionData {
                        session_token,
                        stitcher_base,
                        stitcher_params,
                        active_region,
                    });
                }
            }
        }
    }

    // No valid cache -- fetch fresh session from boot API
    let device_id = get_or_create_device_id();
    let client_id = "plutotv-wasm-client-f7e8d9c0b1a2";
    let boot_url = build_boot_url(&device_id, client_id);

    log_info(&format!("fetching boot session from: {}", boot_url));

    let body = match http_get(&boot_url) {
        Some(b) => b,
        None => {
            log_error("boot API request failed");
            return None;
        }
    };

    let session = parse_boot_response(&body)?;

    log_info(&format!(
        "boot session acquired (region: {}, stitcher: {})",
        session.active_region, session.stitcher_base
    ));

    // Cache session data as JSON in KV store
    if let Ok(json) = serde_json::to_string(&session) {
        kv_set("plutotv_session", &json);
    }

    Some(session)
}

// ============================================================
// Plugin exports
// ============================================================

#[no_mangle]
pub extern "C" fn describe() -> u64 {
    let desc = Descriptor {
        r#type: "plutotv",
        label: "Pluto TV",
        short_label: "PLUTO",
        color: "#00b4ff",
        version: "1.0.0",
        description: "Free live TV channels from Pluto TV",
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

#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn refresh(config_ptr: u32, config_len: u32) -> u64 {
    let _ = read_input(config_ptr, config_len);

    // Step 1: Get session (from cache or boot API)
    let session = match get_session() {
        Some(s) => s,
        None => {
            log_error("failed to get Pluto TV session -- returning empty list");
            return return_json(&RefreshResponse {
                streams: Vec::new(),
            });
        }
    };

    // Step 2: Fetch channel list
    log_info("fetching channel list from api.pluto.tv");
    let body = match http_get("https://api.pluto.tv/v2/channels") {
        Some(b) => b,
        None => {
            log_error("channel list request failed -- returning empty list");
            return return_json(&RefreshResponse {
                streams: Vec::new(),
            });
        }
    };

    // Step 3: Parse channels and build stream list
    let streams = parse_channels(&body, &session);
    log_info(&format!(
        "parsed {} channels (region: {})",
        streams.len(),
        session.active_region
    ));

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
