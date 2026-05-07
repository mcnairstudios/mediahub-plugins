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
    let headers = br#"{"User-Agent":"Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125.0.0.0 Safari/537.36","Origin":"https://openmhz.com","Accept":"application/json","Referer":"https://openmhz.com/"}"#;

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
// Constants -- default systems to scan
// ============================================================

/// Popular trunked radio systems. The plugin fetches up to MAX_SYSTEMS
/// from this list, then up to MAX_CALLS_PER_SYSTEM recent calls each.
const DEFAULT_SYSTEMS: &[&str] = &[
    "chi_cpd",      // Chicago Police
    "chi_cfd",      // Chicago Fire
    "chi_oemc",     // Chicago OEMC
    "sdrtrunk",     // San Diego
    "hennmn",       // Hennepin County MN
    "crptrunk1",    // Corpus Christi TX
    "kcmo",         // Kansas City MO
    "daltrunk",     // Dallas TX
    "lasvegas",     // Las Vegas NV
    "denverpd",     // Denver CO
];

const MAX_SYSTEMS: usize = 8;
const MAX_CALLS_PER_SYSTEM: usize = 50;

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

#[derive(Serialize)]
struct RefreshResponse {
    streams: Vec<Stream>,
}

#[derive(Serialize, Clone, Debug, PartialEq)]
struct Stream {
    id: String,
    name: String,
    url: String,
    group: String,
    logo: String,
    vod_type: String,
    tags: Vec<String>,
}

// ============================================================
// API response types
// ============================================================

#[derive(Deserialize, Debug)]
struct SystemInfo {
    #[serde(rename = "shortName")]
    short_name: String,
    name: String,
}

#[derive(Deserialize, Debug)]
struct SystemsResponse {
    systems: Vec<SystemInfo>,
}

#[derive(Deserialize, Debug, Clone)]
#[allow(dead_code)]
struct SrcEntry {
    src: Option<Value>,
    #[serde(default)]
    tag: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
struct Call {
    #[serde(rename = "_id")]
    id: Option<String>,
    url: Option<String>,
    #[serde(rename = "talkgroupNum")]
    talkgroup_num: Option<i64>,
    #[serde(rename = "srcList")]
    #[allow(dead_code)]
    src_list: Option<Vec<SrcEntry>>,
    time: Option<String>,
    len: Option<f64>,
    #[allow(dead_code)]
    freq: Option<i64>,
    emergency: Option<bool>,
}

#[derive(Deserialize, Debug)]
struct CallsResponse {
    calls: Option<Vec<Call>>,
}

#[derive(Deserialize, Debug, Clone)]
struct Talkgroup {
    num: i64,
    description: Option<String>,
}

#[derive(Deserialize, Debug)]
struct TalkgroupsResponse {
    talkgroups: Option<Vec<Talkgroup>>,
}

// ============================================================
// Config type
// ============================================================

#[derive(Deserialize, Debug)]
struct ConfigSystem {
    #[serde(rename = "shortName")]
    short_name: String,
}

// ============================================================
// Parsing helpers (testable, no host calls)
// ============================================================

/// Parse systems list JSON into a vec of (short_name, display_name) pairs.
fn parse_systems(data: &[u8]) -> Vec<(String, String)> {
    let resp: SystemsResponse = match serde_json::from_slice(data) {
        Ok(r) => r,
        Err(_) => return vec![],
    };
    resp.systems
        .into_iter()
        .map(|s| (s.short_name, s.name))
        .collect()
}

/// Parse calls JSON into a vec of Call structs.
fn parse_calls(data: &[u8]) -> Vec<Call> {
    let resp: CallsResponse = match serde_json::from_slice(data) {
        Ok(r) => r,
        Err(_) => return vec![],
    };
    resp.calls.unwrap_or_default()
}

/// Convert a single call into a Stream, using the system display name as group.
/// `tg_map` maps talkgroup numbers to their descriptions.
fn call_to_stream(
    call: &Call,
    system_name: &str,
    tg_map: &std::collections::HashMap<i64, String>,
) -> Option<Stream> {
    let url = call.url.as_deref().unwrap_or("");
    if url.is_empty() {
        return None;
    }

    let tg_num = call.talkgroup_num.unwrap_or(0);

    let tg_desc = tg_map
        .get(&tg_num)
        .map(|s| s.as_str())
        .unwrap_or("Unknown Talkgroup");

    let duration = call.len.unwrap_or(0.0);

    // Build a human-readable name: "TG Description (Xs)"
    let name = if duration > 0.0 {
        format!("{} ({:.0}s)", tg_desc, duration)
    } else {
        tg_desc.to_string()
    };

    // Use call _id or synthesize from talkgroup + time
    let id = match &call.id {
        Some(id) if !id.is_empty() => id.clone(),
        _ => format!("{}-{}", tg_num, call.time.as_deref().unwrap_or("0")),
    };

    // Build tags from talkgroup number and emergency status
    let mut tags: Vec<String> = Vec::new();
    tags.push(format!("tg{}", tg_num));
    if call.emergency.unwrap_or(false) {
        tags.push("emergency".to_string());
    }

    Some(Stream {
        id,
        name,
        url: url.to_string(),
        group: system_name.to_string(),
        logo: String::new(),
        vod_type: String::new(),
        tags,
    })
}

/// Parse talkgroups JSON into a map from talkgroup number to description.
fn parse_talkgroups(data: &[u8]) -> std::collections::HashMap<i64, String> {
    let resp: TalkgroupsResponse = match serde_json::from_slice(data) {
        Ok(r) => r,
        Err(_) => return std::collections::HashMap::new(),
    };
    resp.talkgroups
        .unwrap_or_default()
        .into_iter()
        .filter_map(|tg| tg.description.map(|desc| (tg.num, desc)))
        .collect()
}

/// Convert a full calls response into streams for a given system.
fn calls_to_streams(
    data: &[u8],
    system_name: &str,
    limit: usize,
    tg_map: &std::collections::HashMap<i64, String>,
) -> Vec<Stream> {
    let calls = parse_calls(data);
    calls
        .iter()
        .take(limit)
        .filter_map(|c| call_to_stream(c, system_name, tg_map))
        .collect()
}

/// Select which systems to fetch from config or defaults.
fn select_systems(config: &serde_json::Map<String, Value>) -> Vec<String> {
    // Check for "systems" config field (array of {shortName: ...} or plain strings)
    if let Some(raw) = config.get("systems") {
        let systems_list: Vec<String> = if let Some(s) = raw.as_str() {
            // Stringified JSON array
            if let Ok(parsed) = serde_json::from_str::<Vec<ConfigSystem>>(s) {
                parsed.into_iter().map(|s| s.short_name).collect()
            } else if let Ok(parsed) = serde_json::from_str::<Vec<String>>(s) {
                parsed
            } else {
                vec![]
            }
        } else if let Ok(parsed) = serde_json::from_value::<Vec<ConfigSystem>>(raw.clone()) {
            parsed.into_iter().map(|s| s.short_name).collect()
        } else if let Ok(parsed) = serde_json::from_value::<Vec<String>>(raw.clone()) {
            parsed
        } else {
            vec![]
        };

        if !systems_list.is_empty() {
            return systems_list.into_iter().take(MAX_SYSTEMS).collect();
        }
    }

    // Fall back to defaults
    DEFAULT_SYSTEMS
        .iter()
        .take(MAX_SYSTEMS)
        .map(|s| s.to_string())
        .collect()
}

/// Build a system name lookup from the systems API response.
fn build_system_names(data: &[u8]) -> std::collections::HashMap<String, String> {
    let pairs = parse_systems(data);
    pairs.into_iter().collect()
}

// ============================================================
// Plugin exports
// ============================================================

#[no_mangle]
pub extern "C" fn describe() -> u64 {
    let desc = Descriptor {
        r#type: "openmhz",
        label: "Scanner Radio",
        short_label: "SCAN",
        color: "#d32f2f",
        version: "1.0.0",
        description: "Police, fire, and EMS radio transmissions from OpenMHz trunked radio systems",
        config_fields: vec![
            serde_json::json!({
                "key": "systems",
                "label": "Radio Systems",
                "type": "text",
                "description": "Comma-separated system short names (e.g. chi_cpd,chi_cfd). Leave empty for defaults.",
                "required": false
            }),
        ],
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
    let input = read_input(config_ptr, config_len);

    let config: serde_json::Map<String, Value> = match serde_json::from_slice(&input) {
        Ok(c) => c,
        Err(e) => {
            log_error(&format!("failed to parse config: {}", e));
            return return_json(&RefreshResponse { streams: vec![] });
        }
    };

    let selected = select_systems(&config);
    log_info(&format!("selected {} systems: {:?}", selected.len(), selected));

    // Fetch the systems list to resolve short names to display names
    let system_names = match http_get("https://api.openmhz.com/systems") {
        Some(data) => build_system_names(&data),
        None => {
            log_error("failed to fetch systems list");
            std::collections::HashMap::new()
        }
    };

    let mut streams: Vec<Stream> = Vec::new();

    for short_name in &selected {
        let display_name = system_names
            .get(short_name.as_str())
            .cloned()
            .unwrap_or_else(|| short_name.clone());

        // Fetch talkgroups for this system to resolve descriptions
        let tg_url = format!("https://api.openmhz.com/{}/talkgroups", short_name);
        let tg_map = match http_get(&tg_url) {
            Some(data) => parse_talkgroups(&data),
            None => {
                log_error(&format!("failed to fetch talkgroups for {}", short_name));
                std::collections::HashMap::new()
            }
        };

        let url = format!("https://api.openmhz.com/{}/calls", short_name);

        let data = match http_get(&url) {
            Some(d) => d,
            None => {
                log_error(&format!("failed to fetch calls for {}", short_name));
                continue;
            }
        };

        let system_streams =
            calls_to_streams(&data, &display_name, MAX_CALLS_PER_SYSTEM, &tg_map);
        log_info(&format!(
            "fetched {} calls for {} ({})",
            system_streams.len(),
            short_name,
            display_name
        ));
        streams.extend(system_streams);
    }

    log_info(&format!("refresh complete: {} total streams", streams.len()));
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
