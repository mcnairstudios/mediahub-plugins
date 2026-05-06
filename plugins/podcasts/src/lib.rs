use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;
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
// Data types -- Refresh / stream output
// ============================================================

#[derive(Serialize, Clone, Debug, PartialEq)]
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub episode_name: Option<String>,
}

#[derive(Serialize)]
struct RefreshResponse {
    streams: Vec<Stream>,
}

// ============================================================
// Interact types
// ============================================================

#[derive(Deserialize)]
struct InteractRequest {
    action: String,
    #[serde(default)]
    params: serde_json::Map<String, Value>,
}

#[derive(Serialize)]
struct SearchResult {
    id: String,
    title: String,
    subtitle: String,
}

// ============================================================
// iTunes API parsing
// ============================================================

/// Parse an iTunes API JSON response (search or lookup) into streams.
/// Filters out entries that are not podcast episodes or lack an episodeUrl.
pub fn parse_episodes(data: &[u8]) -> Vec<Stream> {
    let root: Value = match serde_json::from_slice(data) {
        Ok(v) => v,
        Err(_) => return vec![],
    };

    let results = match root.get("results").and_then(|r| r.as_array()) {
        Some(arr) => arr,
        None => return vec![],
    };

    let mut streams = Vec::new();

    for item in results {
        // Only process podcast episodes (the lookup endpoint also returns
        // a "collection" wrapper as the first result -- skip it).
        let wrapper_type = item
            .get("wrapperType")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        if wrapper_type != "podcastEpisode" {
            // Also accept items with kind == "podcast-episode"
            let kind = item.get("kind").and_then(|v| v.as_str()).unwrap_or("");
            if kind != "podcast-episode" {
                continue;
            }
        }

        let episode_url = item
            .get("episodeUrl")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        if episode_url.is_empty() {
            continue;
        }

        let track_id = item
            .get("trackId")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        let track_name = item
            .get("trackName")
            .and_then(|v| v.as_str())
            .unwrap_or("Untitled Episode");
        let collection_name = item
            .get("collectionName")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown Podcast");
        let artwork = item
            .get("artworkUrl600")
            .and_then(|v| v.as_str())
            .or_else(|| item.get("artworkUrl160").and_then(|v| v.as_str()))
            .or_else(|| item.get("artworkUrl100").and_then(|v| v.as_str()));

        let genres: Option<Vec<String>> = item
            .get("genres")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|g| g.as_str().map(|s| s.to_string()))
                    .collect()
            });

        let short_desc = item
            .get("shortDescription")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());

        streams.push(Stream {
            id: track_id.to_string(),
            name: track_name.to_string(),
            url: episode_url.to_string(),
            group: collection_name.to_string(),
            logo: artwork.map(|s| s.to_string()),
            vod_type: "podcast".to_string(),
            tags: genres,
            episode_name: short_desc,
        });
    }

    streams
}

/// Deduplicate streams by id, keeping the first occurrence.
pub fn dedup_streams(streams: Vec<Stream>) -> Vec<Stream> {
    let mut seen = HashSet::new();
    let mut result = Vec::new();
    for s in streams {
        if seen.insert(s.id.clone()) {
            result.push(s);
        }
    }
    result
}

/// URL-encode a query string (minimal: spaces, &, +, =, #).
fn url_encode(input: &str) -> String {
    let mut result = String::with_capacity(input.len() * 2);
    for b in input.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                result.push(b as char);
            }
            _ => {
                result.push_str(&format!("%{:02X}", b));
            }
        }
    }
    result
}

/// Fetch episodes from the iTunes search endpoint for a given search term.
fn fetch_search_episodes(term: &str, limit: u32) -> Vec<Stream> {
    let encoded = url_encode(term);
    let url = format!(
        "https://itunes.apple.com/search?term={}&media=podcast&entity=podcastEpisode&limit={}",
        encoded, limit
    );
    log_info(&format!("searching episodes: {}", url));

    match http_get(&url) {
        Some(data) => parse_episodes(&data),
        None => {
            log_error(&format!("failed to fetch search results for '{}'", term));
            vec![]
        }
    }
}

/// Fetch episodes for a specific podcast by its iTunes collection ID.
fn fetch_podcast_episodes(collection_id: &str, limit: u32) -> Vec<Stream> {
    let url = format!(
        "https://itunes.apple.com/lookup?id={}&entity=podcastEpisode&limit={}",
        collection_id, limit
    );
    log_info(&format!("fetching podcast episodes: {}", url));

    match http_get(&url) {
        Some(data) => parse_episodes(&data),
        None => {
            log_error(&format!(
                "failed to fetch episodes for podcast {}",
                collection_id
            ));
            vec![]
        }
    }
}

/// Search for podcasts (collections, not episodes) by term. Used by the
/// interact search action to help users discover podcast IDs.
fn search_podcasts(term: &str) -> Vec<SearchResult> {
    let encoded = url_encode(term);
    let url = format!(
        "https://itunes.apple.com/search?term={}&media=podcast&entity=podcast&limit=20",
        encoded
    );
    log_info(&format!("searching podcasts: {}", url));

    let data = match http_get(&url) {
        Some(d) => d,
        None => return vec![],
    };

    let root: Value = match serde_json::from_slice(&data) {
        Ok(v) => v,
        Err(_) => return vec![],
    };

    let results = match root.get("results").and_then(|r| r.as_array()) {
        Some(arr) => arr,
        None => return vec![],
    };

    results
        .iter()
        .filter_map(|item| {
            let collection_id = item.get("collectionId").and_then(|v| v.as_i64())?;
            let name = item
                .get("collectionName")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown");
            let artist = item
                .get("artistName")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let genre = item
                .get("primaryGenreName")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            let subtitle = if !artist.is_empty() && !genre.is_empty() {
                format!("{} - {}", artist, genre)
            } else if !artist.is_empty() {
                artist.to_string()
            } else {
                genre.to_string()
            };

            Some(SearchResult {
                id: collection_id.to_string(),
                title: name.to_string(),
                subtitle,
            })
        })
        .collect()
}

/// Parse a config value as a comma-separated list of trimmed, non-empty strings.
fn parse_comma_list(config: &serde_json::Map<String, Value>, key: &str) -> Vec<String> {
    config
        .get(key)
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Parse the episode limit from config, defaulting to 25.
fn parse_limit(config: &serde_json::Map<String, Value>) -> u32 {
    config
        .get("limit")
        .and_then(|v| {
            // Could be a string like "50" or a number
            v.as_str()
                .and_then(|s| s.parse::<u32>().ok())
                .or_else(|| v.as_u64().map(|n| n as u32))
        })
        .unwrap_or(25)
}

// ============================================================
// Plugin exports
// ============================================================

#[no_mangle]
pub extern "C" fn describe() -> u64 {
    let desc = Descriptor {
        r#type: "podcasts",
        label: "Podcasts",
        short_label: "POD",
        color: "#9c27b0",
        version: "1.0.0",
        description: "Search and browse podcast episodes with direct audio playback via Apple iTunes Search API",
        config_fields: vec![
            serde_json::json!({
                "key": "searches",
                "label": "Search Terms",
                "type": "text",
                "required": false,
                "default": "",
                "description": "Comma-separated search terms to fetch episodes (e.g. true crime, tech news)"
            }),
            serde_json::json!({
                "key": "podcast_ids",
                "label": "Podcast IDs",
                "type": "text",
                "required": false,
                "default": "",
                "description": "Comma-separated iTunes collection IDs for subscribed podcasts"
            }),
            serde_json::json!({
                "key": "limit",
                "label": "Episodes per source",
                "type": "select",
                "required": false,
                "default": "25",
                "options": [
                    {"label": "10", "value": "10"},
                    {"label": "25", "value": "25"},
                    {"label": "50", "value": "50"},
                    {"label": "100", "value": "100"},
                    {"label": "200", "value": "200"}
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
                "id": "search_podcasts",
                "label": "Search Podcasts",
                "type": "search",
                "target_field": "podcast_ids"
            }),
            serde_json::json!({
                "id": "subscribe",
                "label": "Subscribe to Podcast",
                "type": "action",
                "description": "Add a podcast by its iTunes collection ID"
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

    let searches = parse_comma_list(&config, "searches");
    let podcast_ids = parse_comma_list(&config, "podcast_ids");
    let limit = parse_limit(&config);

    if searches.is_empty() && podcast_ids.is_empty() {
        log_info("no search terms or podcast IDs configured");
        return return_json(&RefreshResponse { streams: vec![] });
    }

    let mut all_streams: Vec<Stream> = Vec::new();

    // Fetch episodes for each search term
    for term in &searches {
        let episodes = fetch_search_episodes(term, limit);
        log_info(&format!(
            "search '{}': found {} episodes",
            term,
            episodes.len()
        ));
        all_streams.extend(episodes);
    }

    // Fetch episodes for each subscribed podcast ID
    for id in &podcast_ids {
        let episodes = fetch_podcast_episodes(id, limit);
        log_info(&format!(
            "podcast {}: found {} episodes",
            id,
            episodes.len()
        ));
        all_streams.extend(episodes);
    }

    // Deduplicate by track ID
    let streams = dedup_streams(all_streams);

    log_info(&format!(
        "refresh complete: {} streams from {} searches + {} subscriptions",
        streams.len(),
        searches.len(),
        podcast_ids.len()
    ));

    return_json(&RefreshResponse { streams })
}

#[no_mangle]
pub extern "C" fn interact(action_ptr: u32, action_len: u32) -> u64 {
    let input = read_input(action_ptr, action_len);

    let req: InteractRequest = match serde_json::from_slice(&input) {
        Ok(r) => r,
        Err(e) => {
            log_error(&format!("failed to parse interact request: {}", e));
            return return_json(&serde_json::json!({}));
        }
    };

    match req.action.as_str() {
        "search_podcasts" => {
            let query = req
                .params
                .get("query")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if query.is_empty() {
                let empty: Vec<Value> = vec![];
                return return_json(&serde_json::json!({ "results": empty }));
            }

            let results = search_podcasts(query);
            return_json(&serde_json::json!({ "results": results }))
        }
        "subscribe" => {
            // Subscribe adds a podcast_id to the config. The host handles
            // persisting config; we just need to look up the podcast to
            // confirm it exists and return its info.
            let podcast_id = req
                .params
                .get("podcast_id")
                .or_else(|| req.params.get("id"))
                .and_then(|v| {
                    v.as_str()
                        .map(|s| s.to_string())
                        .or_else(|| v.as_i64().map(|n| n.to_string()))
                })
                .unwrap_or_default();

            if podcast_id.is_empty() {
                return return_json(
                    &serde_json::json!({"error": "podcast_id is required"}),
                );
            }

            let episodes = fetch_podcast_episodes(&podcast_id, 5);
            if episodes.is_empty() {
                return return_json(
                    &serde_json::json!({"error": "no episodes found for this podcast ID"}),
                );
            }

            let podcast_name = &episodes[0].group;
            return_json(&serde_json::json!({
                "success": true,
                "podcast_id": podcast_id,
                "podcast_name": podcast_name,
                "episode_count": episodes.len(),
                "message": format!("Subscribed to '{}'. Add {} to your Podcast IDs config field.", podcast_name, podcast_id)
            }))
        }
        _ => return_json(&serde_json::json!({})),
    }
}
