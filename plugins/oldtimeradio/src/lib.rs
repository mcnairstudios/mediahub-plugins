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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub episode_name: Option<String>,
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

/// Represents a file entry from IA item metadata.
#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct IAFile {
    pub name: String,
    #[serde(default)]
    pub format: String,
    #[serde(default)]
    pub title: String,
}

/// The metadata response for a single IA item.
#[derive(Deserialize)]
pub(crate) struct IAMetadataResponse {
    #[serde(default)]
    pub files: Vec<IAFile>,
}

// ============================================================
// Pure parsing functions (testable without host calls)
// ============================================================

/// Parse the advanced search JSON response into a list of docs.
pub(crate) fn parse_search_response(data: &[u8]) -> Option<Vec<IASearchDoc>> {
    let resp: IASearchResponse = serde_json::from_slice(data).ok()?;
    Some(resp.response.docs)
}

/// Parse item metadata JSON and return only VBR MP3 files.
pub(crate) fn parse_metadata_mp3s(data: &[u8]) -> Option<Vec<IAFile>> {
    let resp: IAMetadataResponse = serde_json::from_slice(data).ok()?;
    let mp3s: Vec<IAFile> = resp
        .files
        .into_iter()
        .filter(|f| f.format == "VBR MP3")
        .collect();
    Some(mp3s)
}

/// Derive a human-friendly episode name from an MP3 filename.
/// Strips the .mp3 extension and attempts to clean up common patterns.
pub(crate) fn episode_name_from_filename(filename: &str) -> String {
    let name = filename
        .strip_suffix(".mp3")
        .or_else(|| filename.strip_suffix(".MP3"))
        .unwrap_or(filename);

    // URL-decode percent-encoded characters
    let decoded = percent_decode(name);

    decoded.trim().to_string()
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

/// Derive a group name from the IA item. Prefers creator, falls back to title,
/// then "Miscellaneous".
pub(crate) fn derive_group(doc: &IASearchDoc) -> String {
    if !doc.creator.is_empty() {
        return doc.creator.clone();
    }
    if !doc.title.is_empty() {
        return doc.title.clone();
    }
    "Miscellaneous".to_string()
}

/// Build a stream ID from an identifier and a file index.
pub(crate) fn make_stream_id(identifier: &str, index: usize) -> String {
    format!("{}__{:04}", identifier, index)
}

/// Build the download URL for an IA file.
pub(crate) fn make_download_url(identifier: &str, filename: &str) -> String {
    // Encode the filename for URL safety (spaces -> %20, etc.)
    let encoded = url_encode_path(filename);
    format!("https://archive.org/download/{}/{}", identifier, encoded)
}

/// Minimal URL path encoding: encodes spaces and a few special characters
/// that commonly appear in IA filenames.
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

/// Convert a single IA doc + its MP3 files into Stream entries.
pub(crate) fn doc_to_streams(doc: &IASearchDoc, mp3_files: &[IAFile]) -> Vec<Stream> {
    let group = derive_group(doc);
    let logo = make_thumbnail_url(&doc.identifier);

    mp3_files
        .iter()
        .enumerate()
        .map(|(i, f)| {
            let ep_name = if !f.title.is_empty() {
                f.title.clone()
            } else {
                episode_name_from_filename(&f.name)
            };
            Stream {
                id: make_stream_id(&doc.identifier, i),
                name: ep_name.clone(),
                url: make_download_url(&doc.identifier, &f.name),
                group: group.clone(),
                logo: Some(logo.clone()),
                vod_type: "movie".to_string(),
                tags: Some(vec!["radio".to_string(), "classic".to_string()]),
                episode_name: Some(ep_name),
            }
        })
        .collect()
}

// ============================================================
// Fetch helpers (require host calls)
// ============================================================

/// Fetch search results from IA advanced search API.
fn fetch_search(max_shows: u32) -> Vec<IASearchDoc> {
    let url = format!(
        "https://archive.org/advancedsearch.php?q=collection:oldtimeradio+AND+format:VBR+MP3&fl=identifier,title,description,creator&rows={}&sort=downloads+desc&output=json",
        max_shows
    );

    log_info(&format!("fetching IA search: rows={}", max_shows));

    let body = match http_get(&url) {
        Some(b) => b,
        None => {
            log_error("failed to fetch IA search results");
            return vec![];
        }
    };

    match parse_search_response(&body) {
        Some(docs) => {
            log_info(&format!("parsed {} search results", docs.len()));
            docs
        }
        None => {
            log_error("failed to parse IA search response");
            vec![]
        }
    }
}

/// Fetch metadata for a single IA item, using KV cache.
fn fetch_item_metadata(identifier: &str) -> Option<Vec<IAFile>> {
    let cache_key = format!("ia_meta_{}", identifier);

    // Try KV cache first
    if let Some(cached) = kv_get(&cache_key) {
        if let Ok(files) = serde_json::from_str::<Vec<IAFile>>(&cached) {
            return Some(files);
        }
    }

    let url = format!("https://archive.org/metadata/{}", identifier);

    let body = match http_get(&url) {
        Some(b) => b,
        None => {
            log_error(&format!("failed to fetch metadata for {}", identifier));
            return None;
        }
    };

    let mp3s = parse_metadata_mp3s(&body)?;

    // Cache the parsed MP3 file list
    if let Ok(cache_data) = serde_json::to_string(&mp3s) {
        kv_set(&cache_key, &cache_data);
    }

    Some(mp3s)
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
        version: "1.0.0",
        description: "Classic radio shows from the 1930s-1950s golden age of radio, sourced from Internet Archive",
        config_fields: vec![
            serde_json::json!({
                "key": "max_shows",
                "label": "Max shows to load",
                "type": "number",
                "required": false,
                "default": 50
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
        .unwrap_or(50) as u32;

    // Try loading cached streams first
    if let Some(cached) = kv_get("otr_streams_cache") {
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

    let docs = fetch_search(max_shows);
    if docs.is_empty() {
        log_error("no search results returned");
        return return_json(&RefreshResponse { streams: vec![] });
    }

    let mut streams: Vec<Stream> = Vec::new();

    for doc in &docs {
        let mp3s = match fetch_item_metadata(&doc.identifier) {
            Some(files) if !files.is_empty() => files,
            _ => {
                log_info(&format!("no MP3s for {}, skipping", doc.identifier));
                continue;
            }
        };

        let item_streams = doc_to_streams(doc, &mp3s);
        log_info(&format!(
            "{}: {} episodes",
            doc.identifier,
            item_streams.len()
        ));
        streams.extend(item_streams);
    }

    log_info(&format!(
        "refresh complete: {} streams from {} items",
        streams.len(),
        docs.len()
    ));

    let resp = RefreshResponse { streams };

    // Cache the full result
    if let Ok(cache_data) = serde_json::to_string(&resp) {
        kv_set("otr_streams_cache", &cache_data);
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

    // Search directly against Internet Archive
    let encoded_query = url_encode_path(query);
    let url = format!(
        "https://archive.org/advancedsearch.php?q=collection:oldtimeradio+AND+format:VBR+MP3+AND+({})+&fl=identifier,title,description,creator&rows=20&sort=downloads+desc&output=json",
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
