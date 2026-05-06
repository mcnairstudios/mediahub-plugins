#![allow(dead_code)]

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

#[derive(Serialize, Clone)]
struct Stream {
    id: String,
    name: String,
    url: String,
    group: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    logo: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    vod_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    year: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tags: Option<Vec<String>>,
}

#[derive(Serialize)]
struct RefreshResponse {
    streams: Vec<Stream>,
}

// ============================================================
// Collection definitions
// ============================================================

struct CollectionInfo {
    id: &'static str,
    display_name: &'static str,
    media_type: MediaType,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MediaType {
    Video,
    Audio,
}

const KNOWN_COLLECTIONS: &[CollectionInfo] = &[
    CollectionInfo { id: "feature_films", display_name: "Feature Films", media_type: MediaType::Video },
    CollectionInfo { id: "prelinger", display_name: "Prelinger Archives", media_type: MediaType::Video },
    CollectionInfo { id: "oldtimeradio", display_name: "Old Time Radio", media_type: MediaType::Audio },
    CollectionInfo { id: "GratefulDead", display_name: "Grateful Dead", media_type: MediaType::Audio },
    CollectionInfo { id: "classic_tv", display_name: "Classic TV", media_type: MediaType::Video },
    CollectionInfo { id: "silent_films", display_name: "Silent Films", media_type: MediaType::Video },
    CollectionInfo { id: "film_noir", display_name: "Film Noir", media_type: MediaType::Video },
    CollectionInfo { id: "scifi", display_name: "Sci-Fi Films", media_type: MediaType::Video },
    CollectionInfo { id: "comedy_films", display_name: "Comedy Films", media_type: MediaType::Video },
    CollectionInfo { id: "horror_movies", display_name: "Horror Movies", media_type: MediaType::Video },
    CollectionInfo { id: "anime", display_name: "Anime", media_type: MediaType::Video },
    CollectionInfo { id: "classic_cartoons", display_name: "Classic Cartoons", media_type: MediaType::Video },
    CollectionInfo { id: "noir", display_name: "Noir Classics", media_type: MediaType::Video },
    CollectionInfo { id: "moviesandfilms", display_name: "Movies & Films", media_type: MediaType::Video },
];

fn lookup_collection(id: &str) -> Option<&'static CollectionInfo> {
    KNOWN_COLLECTIONS.iter().find(|c| c.id == id)
}

fn collection_display_name(id: &str) -> String {
    lookup_collection(id)
        .map(|c| c.display_name.to_string())
        .unwrap_or_else(|| id.to_string())
}

fn collection_media_type(id: &str) -> MediaType {
    lookup_collection(id)
        .map(|c| c.media_type)
        .unwrap_or(MediaType::Video)
}

// ============================================================
// Config parsing
// ============================================================

#[derive(Deserialize)]
struct Config {
    #[serde(default = "default_collections")]
    collections: String,
    #[serde(default = "default_items_per_collection")]
    items_per_collection: Value,
    #[serde(default = "default_sort")]
    sort: String,
}

fn default_collections() -> String {
    "feature_films,prelinger,oldtimeradio,GratefulDead,classic_tv,silent_films,film_noir,scifi,comedy_films,horror_movies,classic_cartoons".to_string()
}

fn default_items_per_collection() -> Value {
    Value::Number(serde_json::Number::from(40))
}

fn default_sort() -> String {
    "downloads desc".to_string()
}

fn parse_items_count(v: &Value) -> u32 {
    match v {
        Value::Number(n) => n.as_u64().unwrap_or(40) as u32,
        Value::String(s) => s.parse().unwrap_or(40),
        _ => 40,
    }
}

fn parse_collection_list(s: &str) -> Vec<String> {
    s.split(',')
        .map(|c| c.trim().to_string())
        .filter(|c| !c.is_empty())
        .collect()
}

// ============================================================
// Search response parsing (public for testing)
// ============================================================

#[derive(Deserialize, Debug)]
pub struct SearchResponse {
    pub response: SearchResponseInner,
}

#[derive(Deserialize, Debug)]
pub struct SearchResponseInner {
    #[serde(default)]
    pub docs: Vec<SearchDoc>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct SearchDoc {
    #[serde(default)]
    pub identifier: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub year: Option<Value>,
    #[serde(default)]
    pub creator: Option<Value>,
}

/// Parse search API JSON into docs.
pub fn parse_search_response(data: &[u8]) -> Option<Vec<SearchDoc>> {
    let resp: SearchResponse = serde_json::from_slice(data).ok()?;
    Some(resp.response.docs)
}

// ============================================================
// Year / creator extraction helpers (public for testing)
// ============================================================

/// Extract a year string from a Value that may be a string or number.
pub fn extract_year(v: &Option<Value>) -> Option<String> {
    match v {
        Some(Value::String(s)) => {
            let trimmed = s.trim();
            if trimmed.len() >= 4 {
                Some(trimmed[..4].to_string())
            } else if !trimmed.is_empty() {
                Some(trimmed.to_string())
            } else {
                None
            }
        }
        Some(Value::Number(n)) => Some(n.to_string()),
        _ => None,
    }
}

/// Extract creator from a Value that may be a string or array.
pub fn extract_creator(v: &Option<Value>) -> Option<String> {
    match v {
        Some(Value::String(s)) if !s.is_empty() => Some(s.clone()),
        Some(Value::Array(arr)) => arr.first().and_then(|v| v.as_str()).map(|s| s.to_string()),
        _ => None,
    }
}

// ============================================================
// Heuristic URL construction
// ============================================================

/// Build a heuristic download URL for a video item.
/// The Internet Archive commonly derives an .mp4 file named `{identifier}.mp4`.
pub fn heuristic_video_url(identifier: &str) -> String {
    format!("https://archive.org/download/{}/{}.mp4", identifier, identifier)
}

/// Build a heuristic download URL for an audio item.
/// Many audio items have a VBR MP3 derivative named `{identifier}.mp3` or
/// we can use the built-in playlist endpoint.
pub fn heuristic_audio_url(identifier: &str) -> String {
    format!("https://archive.org/download/{}/{}_vbr.mp3", identifier, identifier)
}

fn thumbnail_url(identifier: &str) -> String {
    format!("https://archive.org/services/img/{}", identifier)
}

fn search_url(collection: &str, sort: &str, rows: u32) -> String {
    let encoded_sort = sort.replace(' ', "+");
    format!(
        "https://archive.org/advancedsearch.php?q=collection:{}&fl[]=identifier&fl[]=title&fl[]=description&fl[]=year&fl[]=creator&sort={}&rows={}&output=json",
        collection, encoded_sort, rows
    )
}

fn search_query_url(query: &str, collections: &[String], sort: &str, rows: u32) -> String {
    let collection_filter = if collections.is_empty() {
        String::new()
    } else {
        let parts: Vec<String> = collections.iter().map(|c| format!("collection:{}", c)).collect();
        format!(" AND ({})", parts.join(" OR "))
    };
    let encoded_query = query.replace(' ', "+");
    let encoded_sort = sort.replace(' ', "+");
    format!(
        "https://archive.org/advancedsearch.php?q={}{}&fl[]=identifier&fl[]=title&fl[]=description&fl[]=year&fl[]=creator&sort={}&rows={}&output=json",
        encoded_query, collection_filter, encoded_sort, rows
    )
}

// ============================================================
// Build streams directly from search results (no per-item fetch)
// ============================================================

/// Convert a search doc to a stream using heuristic URL patterns.
/// No per-item metadata fetch required.
pub(crate) fn doc_to_stream(doc: &SearchDoc, group_name: &str, media_type: MediaType) -> Stream {
    let name = if !doc.title.is_empty() {
        doc.title.clone()
    } else {
        doc.identifier.clone()
    };

    let year = extract_year(&doc.year);

    let url = match media_type {
        MediaType::Video => heuristic_video_url(&doc.identifier),
        MediaType::Audio => heuristic_audio_url(&doc.identifier),
    };

    let vod_type = if media_type == MediaType::Video {
        Some("movie".to_string())
    } else {
        None
    };

    let tags = match media_type {
        MediaType::Video => Some(vec!["video".to_string()]),
        MediaType::Audio => Some(vec!["audio".to_string()]),
    };

    Stream {
        id: doc.identifier.clone(),
        name,
        url,
        group: group_name.to_string(),
        logo: Some(thumbnail_url(&doc.identifier)),
        vod_type,
        year,
        tags,
    }
}

fn fetch_collection_streams(collection_id: &str, sort: &str, rows: u32) -> Vec<Stream> {
    let display_name = collection_display_name(collection_id);
    let media_type = collection_media_type(collection_id);

    let url = search_url(collection_id, sort, rows);
    log_info(&format!("fetching collection {}: {}", collection_id, url));

    let body = match http_get(&url) {
        Some(b) => b,
        None => {
            log_error(&format!("failed to fetch search for {}", collection_id));
            return vec![];
        }
    };

    let docs = match parse_search_response(&body) {
        Some(d) => d,
        None => {
            log_error(&format!("failed to parse search for {}", collection_id));
            return vec![];
        }
    };

    log_info(&format!("found {} items in {}", docs.len(), collection_id));

    let streams: Vec<Stream> = docs
        .iter()
        .filter(|doc| !doc.identifier.is_empty())
        .map(|doc| doc_to_stream(doc, &display_name, media_type))
        .collect();

    log_info(&format!("built {} streams for {}", streams.len(), collection_id));
    streams
}

// ============================================================
// Plugin exports
// ============================================================

#[no_mangle]
pub extern "C" fn describe() -> u64 {
    let desc = Descriptor {
        r#type: "archive",
        label: "Internet Archive",
        short_label: "ARCHIVE",
        color: "#428bca",
        version: "2.0.0",
        description: "Public domain movies, classic TV, old-time radio, and live concert recordings from the Internet Archive",
        config_fields: vec![
            serde_json::json!({
                "key": "collections",
                "label": "Collections",
                "type": "select",
                "default": "feature_films,prelinger,oldtimeradio,GratefulDead,classic_tv,silent_films,film_noir,scifi,comedy_films,horror_movies,classic_cartoons",
                "options": [
                    {"value": "feature_films", "label": "Feature Films"},
                    {"value": "prelinger", "label": "Prelinger Archives"},
                    {"value": "oldtimeradio", "label": "Old Time Radio"},
                    {"value": "GratefulDead", "label": "Grateful Dead"},
                    {"value": "classic_tv", "label": "Classic TV"},
                    {"value": "silent_films", "label": "Silent Films"},
                    {"value": "film_noir", "label": "Film Noir"},
                    {"value": "scifi", "label": "Sci-Fi Films"},
                    {"value": "comedy_films", "label": "Comedy Films"},
                    {"value": "horror_movies", "label": "Horror Movies"},
                    {"value": "anime", "label": "Anime"},
                    {"value": "classic_cartoons", "label": "Classic Cartoons"},
                    {"value": "noir", "label": "Noir Classics"},
                    {"value": "moviesandfilms", "label": "Movies & Films"}
                ],
                "multi": true,
                "description": "Select which Internet Archive collections to browse"
            }),
            serde_json::json!({
                "key": "items_per_collection",
                "label": "Items per collection",
                "type": "number",
                "default": 40,
                "description": "Number of items to fetch per collection (max 200)"
            }),
            serde_json::json!({
                "key": "sort",
                "label": "Sort by",
                "type": "select",
                "default": "downloads desc",
                "options": [
                    {"value": "downloads desc", "label": "Most Downloaded"},
                    {"value": "date desc", "label": "Newest First"},
                    {"value": "date asc", "label": "Oldest First"},
                    {"value": "titleSorter asc", "label": "Title A-Z"}
                ]
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
                "label": "Search Archive",
                "type": "search"
            }),
        ],
    };
    return_json(&desc)
}

#[no_mangle]
pub extern "C" fn refresh(config_ptr: u32, config_len: u32) -> u64 {
    let input = read_input(config_ptr, config_len);

    let config: Config = match serde_json::from_slice(&input) {
        Ok(c) => c,
        Err(e) => {
            log_error(&format!("failed to parse config: {}", e));
            return return_json(&RefreshResponse { streams: vec![] });
        }
    };

    let collections = parse_collection_list(&config.collections);
    let rows = parse_items_count(&config.items_per_collection).min(200);
    let sort = &config.sort;

    let active_collections = if collections.is_empty() {
        parse_collection_list(&default_collections())
    } else {
        collections
    };

    let mut streams = Vec::new();
    for coll in &active_collections {
        streams.extend(fetch_collection_streams(coll, sort, rows));
    }

    log_info(&format!(
        "refresh complete: {} total streams from {} collections ({} HTTP requests)",
        streams.len(),
        active_collections.len(),
        active_collections.len()
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
        #[serde(default)]
        config: Option<Value>,
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
        return return_json(&serde_json::json!({ "streams": [] }));
    }

    let collections = if let Some(cfg) = &req.config {
        cfg.get("collections")
            .and_then(|v| v.as_str())
            .map(|s| parse_collection_list(s))
            .unwrap_or_else(|| parse_collection_list(&default_collections()))
    } else {
        parse_collection_list(&default_collections())
    };

    let sort = if let Some(cfg) = &req.config {
        cfg.get("sort")
            .and_then(|v| v.as_str())
            .unwrap_or("downloads desc")
            .to_string()
    } else {
        "downloads desc".to_string()
    };

    let url = search_query_url(query, &collections, &sort, 30);
    log_info(&format!("search query: {}", url));

    let body = match http_get(&url) {
        Some(b) => b,
        None => {
            log_error("search request failed");
            return return_json(&serde_json::json!({ "streams": [] }));
        }
    };

    let docs = match parse_search_response(&body) {
        Some(d) => d,
        None => {
            log_error("failed to parse search results");
            return return_json(&serde_json::json!({ "streams": [] }));
        }
    };

    log_info(&format!("search returned {} results", docs.len()));

    // Build streams directly from search results -- no per-item fetches
    let streams: Vec<Stream> = docs
        .iter()
        .filter(|doc| !doc.identifier.is_empty())
        .map(|doc| {
            // Default to video for search results
            doc_to_stream(doc, "Search Results", MediaType::Video)
        })
        .collect();

    return_json(&serde_json::json!({ "streams": streams }))
}
