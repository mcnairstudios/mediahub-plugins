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

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct Stream {
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

#[derive(Serialize, Deserialize)]
struct RefreshResponse {
    streams: Vec<Stream>,
}

// ============================================================
// Search result type for interact
// ============================================================

#[derive(Serialize)]
struct SearchResult {
    id: String,
    title: String,
    subtitle: String,
}

// ============================================================
// Internet Archive response parsing
// ============================================================

/// Represents one item from the IA advanced search results.
#[derive(Deserialize, Clone)]
pub(crate) struct IASearchDoc {
    pub identifier: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub creator: String,
}

/// The response envelope from the IA advanced search API.
#[derive(Deserialize)]
pub(crate) struct IASearchResponse {
    pub response: IASearchResponseInner,
}

#[derive(Deserialize)]
pub(crate) struct IASearchResponseInner {
    pub docs: Vec<IASearchDoc>,
}

// ============================================================
// Show name categories for grouping
// ============================================================

/// Well-known old-time radio show names for grouping.
const KNOWN_SHOWS: &[(&str, &str)] = &[
    ("gunsmoke", "Gunsmoke"),
    ("dragnet", "Dragnet"),
    ("suspense", "Suspense"),
    ("x_minus", "X Minus One"),
    ("xminusone", "X Minus One"),
    ("fibber_mcgee", "Fibber McGee and Molly"),
    ("fibbermcgee", "Fibber McGee and Molly"),
    ("jack_benny", "The Jack Benny Program"),
    ("jackbenny", "The Jack Benny Program"),
    ("burns_allen", "Burns and Allen"),
    ("burnsallen", "Burns and Allen"),
    ("lone_ranger", "The Lone Ranger"),
    ("loneranger", "The Lone Ranger"),
    ("shadow", "The Shadow"),
    ("inner_sanctum", "Inner Sanctum"),
    ("innersanctum", "Inner Sanctum"),
    ("mercury_theater", "Mercury Theater"),
    ("mercury_theatre", "Mercury Theatre"),
    ("lights_out", "Lights Out"),
    ("lightsout", "Lights Out"),
    ("dimension_x", "Dimension X"),
    ("dimensionx", "Dimension X"),
    ("escapepod", "Escape"),
    ("escape_", "Escape"),
    ("amos_andy", "Amos 'n' Andy"),
    ("amosandy", "Amos 'n' Andy"),
    ("green_hornet", "The Green Hornet"),
    ("greenhornet", "The Green Hornet"),
    ("have_gun", "Have Gun Will Travel"),
    ("philip_marlowe", "Philip Marlowe"),
    ("philipmarlow", "Philip Marlowe"),
    ("our_miss_brooks", "Our Miss Brooks"),
    ("ourmissbrooks", "Our Miss Brooks"),
    ("cbsradio", "CBS Radio Mystery Theater"),
    ("cbs_radio_mystery", "CBS Radio Mystery Theater"),
    ("bold_venture", "Bold Venture"),
    ("boldventure", "Bold Venture"),
    ("yours_truly", "Yours Truly, Johnny Dollar"),
    ("johnnydollar", "Yours Truly, Johnny Dollar"),
    ("richard_diamond", "Richard Diamond"),
    ("richarddiamond", "Richard Diamond"),
    ("box_thirteen", "Box Thirteen"),
    ("boxthirteen", "Box Thirteen"),
    ("tales_of_texas", "Tales of the Texas Rangers"),
    ("gangbusters", "Gangbusters"),
    ("fort_laramie", "Fort Laramie"),
    ("fortlaramie", "Fort Laramie"),
    ("pat_novak", "Pat Novak For Hire"),
    ("patnovak", "Pat Novak For Hire"),
];

// ============================================================
// Pure parsing functions (testable without host calls)
// ============================================================

/// Parse the advanced search JSON response into a list of docs.
pub(crate) fn parse_search_response(data: &[u8]) -> Option<Vec<IASearchDoc>> {
    let resp: IASearchResponse = serde_json::from_slice(data).ok()?;
    Some(resp.response.docs)
}

/// Derive a group name from the IA item. Uses show-name matching on the
/// identifier and title, falls back to creator, then "Miscellaneous".
pub(crate) fn derive_group(doc: &IASearchDoc) -> String {
    let id_lower = doc.identifier.to_lowercase();
    let title_lower = doc.title.to_lowercase();

    // Check known show patterns against identifier and title
    for &(pattern, show_name) in KNOWN_SHOWS {
        if id_lower.contains(pattern) || title_lower.contains(pattern) {
            return show_name.to_string();
        }
    }

    // Fall back to creator
    if !doc.creator.is_empty() {
        return doc.creator.clone();
    }

    // Fall back to title (for collection-type items)
    if !doc.title.is_empty() {
        return doc.title.clone();
    }

    "Miscellaneous".to_string()
}

/// Simple percent-decoding for URL-encoded filenames.
pub(crate) fn percent_decode(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let bytes = input.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(byte_val) = u8::from_str_radix(
                &input[i + 1..i + 3],
                16,
            ) {
                result.push(byte_val as char);
                i += 3;
                continue;
            }
        }
        result.push(bytes[i] as char);
        i += 1;
    }
    result
}

/// Minimal URL path encoding for IA filenames.
pub(crate) fn url_encode_path(input: &str) -> String {
    let mut result = String::with_capacity(input.len() * 2);
    for b in input.bytes() {
        match b {
            b' ' => result.push_str("%20"),
            b'#' => result.push_str("%23"),
            b'?' => result.push_str("%3F"),
            b'[' => result.push_str("%5B"),
            b']' => result.push_str("%5D"),
            _ => result.push(b as char),
        }
    }
    result
}

/// Build the thumbnail URL for an IA item.
fn make_thumbnail_url(identifier: &str) -> String {
    format!("https://archive.org/services/img/{}", identifier)
}

/// Build a heuristic audio stream URL for an IA item.
/// Uses the VBR MP3 aggregate endpoint which plays all tracks in an item.
pub(crate) fn make_stream_url(identifier: &str) -> String {
    format!(
        "https://archive.org/download/{}/{}_vbr.mp3",
        identifier, identifier
    )
}

/// Convert a single IA search doc into a Stream using heuristic URLs.
/// No per-item metadata fetch needed.
pub(crate) fn doc_to_stream(doc: &IASearchDoc) -> Stream {
    let group = derive_group(doc);
    let logo = make_thumbnail_url(&doc.identifier);
    let url = make_stream_url(&doc.identifier);

    let name = if !doc.title.is_empty() {
        doc.title.clone()
    } else {
        doc.identifier.clone()
    };

    Stream {
        id: doc.identifier.clone(),
        name,
        url,
        group,
        logo: Some(logo),
        vod_type: "movie".to_string(),
        tags: Some(vec!["radio".to_string(), "classic".to_string()]),
    }
}

// ============================================================
// Search queries for different show categories
// ============================================================

/// Search queries to fetch diverse shows across multiple categories.
/// Each query focuses on popular shows from the collection and returns
/// a batch of results, keeping total HTTP requests low.
const SEARCH_QUERIES: &[(&str, u32)] = &[
    // Main collection sorted by popularity -- catches the big shows
    ("collection:oldtimeradio", 80),
    // Drama and mystery
    ("collection:oldtimeradio AND (suspense OR mystery OR thriller)", 50),
    // Comedy
    ("collection:oldtimeradio AND (comedy OR funny OR humor)", 40),
    // Western
    ("collection:oldtimeradio AND (western OR gunsmoke OR ranger)", 40),
    // Science fiction
    ("collection:oldtimeradio AND (science fiction OR dimension OR x minus)", 40),
    // Detective / crime
    ("collection:oldtimeradio AND (detective OR crime OR dragnet OR marlowe)", 40),
];

fn build_search_url(query: &str, rows: u32, sort: &str) -> String {
    let encoded_sort = sort.replace(' ', "+");
    let encoded_query = query.replace(' ', "+");
    format!(
        "https://archive.org/advancedsearch.php?q={}&fl[]=identifier&fl[]=title&fl[]=description&fl[]=creator&sort={}&rows={}&output=json",
        encoded_query, encoded_sort, rows
    )
}

// ============================================================
// Plugin exports
// ============================================================

#[no_mangle]
pub extern "C" fn describe() -> u64 {
    let desc = Descriptor {
        r#type: "oldtimeradio",
        label: "Old Time Radio",
        short_label: "OTR",
        color: "#8d6e63",
        version: "2.0.0",
        description: "Classic radio shows from the 1930s-1950s golden age of radio, sourced from Internet Archive",
        config_fields: vec![
            serde_json::json!({
                "key": "max_shows",
                "label": "Max shows to load",
                "type": "number",
                "required": false,
                "default": 300
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
                "id": "search",
                "label": "Search Shows",
                "type": "search",
                "target_field": "query"
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

    let max_shows = config
        .get("max_shows")
        .and_then(|v| v.as_u64())
        .unwrap_or(300) as u32;

    // Try loading cached streams first
    if let Some(cached) = kv_get("otr_streams_cache_v2") {
        if let Ok(resp) = serde_json::from_str::<RefreshResponse>(&cached) {
            if !resp.streams.is_empty() {
                log_info(&format!(
                    "returning {} cached streams",
                    resp.streams.len()
                ));
                return return_json(&resp);
            }
        }
    }

    // Track seen identifiers to deduplicate across queries
    let mut seen = std::collections::HashSet::new();
    let mut streams: Vec<Stream> = Vec::new();

    for &(query, rows) in SEARCH_QUERIES {
        if streams.len() >= max_shows as usize {
            break;
        }

        let url = build_search_url(query, rows, "downloads+desc");
        log_info(&format!("fetching: {}", url));

        let body = match http_get(&url) {
            Some(b) => b,
            None => {
                log_error(&format!("failed to fetch search: {}", query));
                continue;
            }
        };

        let docs = match parse_search_response(&body) {
            Some(d) => d,
            None => {
                log_error(&format!("failed to parse search: {}", query));
                continue;
            }
        };

        log_info(&format!("query returned {} results", docs.len()));

        for doc in &docs {
            if seen.contains(&doc.identifier) {
                continue;
            }
            seen.insert(doc.identifier.clone());
            streams.push(doc_to_stream(doc));

            if streams.len() >= max_shows as usize {
                break;
            }
        }
    }

    log_info(&format!(
        "refresh complete: {} streams from {} HTTP requests",
        streams.len(),
        SEARCH_QUERIES.len()
    ));

    let resp = RefreshResponse { streams };

    // Cache the full result
    if let Ok(cache_data) = serde_json::to_string(&resp) {
        kv_set("otr_streams_cache_v2", &cache_data);
    }

    return_json(&resp)
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

    if req.action != "search" {
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

    // Search directly against Internet Archive -- single HTTP request
    let encoded_query = url_encode_path(query);
    let url = format!(
        "https://archive.org/advancedsearch.php?q=collection:oldtimeradio+AND+({})+&fl[]=identifier&fl[]=title&fl[]=description&fl[]=creator&rows=20&sort=downloads+desc&output=json",
        encoded_query
    );

    let body = match http_get(&url) {
        Some(b) => b,
        None => {
            log_error("search request failed");
            let empty: Vec<Value> = vec![];
            return return_json(&serde_json::json!({ "results": empty }));
        }
    };

    let docs = match parse_search_response(&body) {
        Some(d) => d,
        None => {
            log_error("failed to parse search response");
            let empty: Vec<Value> = vec![];
            return return_json(&serde_json::json!({ "results": empty }));
        }
    };

    let results: Vec<SearchResult> = docs
        .iter()
        .take(20)
        .map(|doc| {
            let subtitle = if !doc.creator.is_empty() {
                doc.creator.clone()
            } else if !doc.description.is_empty() {
                let desc = &doc.description;
                if desc.len() > 100 {
                    format!("{}...", &desc[..100])
                } else {
                    desc.clone()
                }
            } else {
                doc.identifier.clone()
            };
            SearchResult {
                id: doc.identifier.clone(),
                title: doc.title.clone(),
                subtitle,
            }
        })
        .collect();

    return_json(&serde_json::json!({ "results": results }))
}
