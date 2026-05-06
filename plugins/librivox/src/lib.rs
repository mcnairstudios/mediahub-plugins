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
pub struct Stream {
    pub id: String,
    pub name: String,
    pub url: String,
    pub group: String,
    pub logo: String,
    pub vod_type: String,
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub episode_name: Option<String>,
}

#[derive(Serialize)]
struct RefreshResponse {
    streams: Vec<Stream>,
}

// ============================================================
// API response types
// ============================================================

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Author {
    #[serde(default)]
    pub first_name: String,
    #[serde(default)]
    pub last_name: String,
}

impl Author {
    pub fn full_name(&self) -> String {
        let name = format!("{} {}", self.first_name, self.last_name);
        let trimmed = name.trim();
        if trimmed.is_empty() {
            "Unknown Author".to_string()
        } else {
            trimmed.to_string()
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Audiobook {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub language: String,
    #[serde(default)]
    pub num_sections: String,
    #[serde(default)]
    pub authors: Vec<Author>,
    #[serde(default)]
    pub totaltime: String,
    #[serde(default)]
    pub url_librivox: String,
}

impl Audiobook {
    pub fn author_display(&self) -> String {
        if self.authors.is_empty() {
            return "Unknown Author".to_string();
        }
        self.authors
            .iter()
            .map(|a| a.full_name())
            .collect::<Vec<_>>()
            .join(", ")
    }

    pub fn group_label(&self) -> String {
        format!("{} - {}", self.title, self.author_display())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AudioTrack {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub section_number: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub listen_url: String,
    #[serde(default)]
    pub language: String,
    #[serde(default)]
    pub playtime: String,
}

#[derive(Deserialize, Debug)]
pub struct AudiobooksApiResponse {
    pub books: Vec<Audiobook>,
}

#[derive(Deserialize, Debug)]
pub struct AudiotracksApiResponse {
    pub sections: Vec<AudioTrack>,
}

// ============================================================
// Parsing helpers (public for tests)
// ============================================================

pub fn parse_audiobooks(data: &[u8]) -> Option<Vec<Audiobook>> {
    let resp: AudiobooksApiResponse = serde_json::from_slice(data).ok()?;
    Some(resp.books)
}

pub fn parse_audiotracks(data: &[u8]) -> Option<Vec<AudioTrack>> {
    let resp: AudiotracksApiResponse = serde_json::from_slice(data).ok()?;
    Some(resp.sections)
}

pub fn audiobooks_url(limit: u32, language: &str) -> String {
    if language.is_empty() || language == "all" {
        format!(
            "https://librivox.org/api/feed/audiobooks?format=json&limit={}",
            limit
        )
    } else {
        format!(
            "https://librivox.org/api/feed/audiobooks?format=json&limit={}&language={}",
            limit, language
        )
    }
}

pub fn audiotracks_url(book_id: &str) -> String {
    format!(
        "https://librivox.org/api/feed/audiotracks?project_id={}&format=json",
        book_id
    )
}

pub fn search_url(query: &str, limit: u32) -> String {
    let encoded = query.replace(' ', "+");
    format!(
        "https://librivox.org/api/feed/audiobooks?format=json&limit={}&title=^{}",
        limit, encoded
    )
}

pub fn track_to_stream(book: &Audiobook, track: &AudioTrack) -> Stream {
    let section_num = &track.section_number;
    let stream_id = format!("lbv-{}-{}", book.id, section_num);
    let episode_label = format!("Ch. {}: {}", section_num, track.title);
    let group = book.group_label();
    let mut tags = vec!["audiobook".to_string()];
    if !track.language.is_empty() {
        tags.push(track.language.clone());
    } else if !book.language.is_empty() {
        tags.push(book.language.clone());
    }

    Stream {
        id: stream_id,
        name: track.title.clone(),
        url: track.listen_url.clone(),
        group,
        logo: String::new(),
        vod_type: "episode".to_string(),
        tags,
        episode_name: Some(episode_label),
    }
}

// ============================================================
// Cached book list fetcher
// ============================================================

fn fetch_books_cached(limit: u32, language: &str) -> Vec<Audiobook> {
    let cache_key = format!("books_{}_{}", limit, language);

    // Try cache first
    if let Some(cached) = kv_get(&cache_key) {
        if let Ok(books) = serde_json::from_str::<Vec<Audiobook>>(&cached) {
            if !books.is_empty() {
                log_info(&format!("using cached book list ({} books)", books.len()));
                return books;
            }
        }
    }

    let url = audiobooks_url(limit, language);
    log_info(&format!("fetching audiobooks: {}", url));

    let body = match http_get(&url) {
        Some(b) => b,
        None => {
            log_error("failed to fetch audiobooks");
            return vec![];
        }
    };

    let books = match parse_audiobooks(&body) {
        Some(b) => b,
        None => {
            log_error("failed to parse audiobooks response");
            return vec![];
        }
    };

    log_info(&format!("fetched {} audiobooks, caching", books.len()));

    if let Ok(cache_data) = serde_json::to_string(&books) {
        kv_set(&cache_key, &cache_data);
    }

    books
}

/// Fetch tracks for a single book, with KV caching.
fn fetch_tracks_cached(book_id: &str) -> Vec<AudioTrack> {
    let cache_key = format!("tracks_{}", book_id);

    if let Some(cached) = kv_get(&cache_key) {
        if let Ok(tracks) = serde_json::from_str::<Vec<AudioTrack>>(&cached) {
            if !tracks.is_empty() {
                return tracks;
            }
        }
    }

    let url = audiotracks_url(book_id);
    log_info(&format!("fetching tracks for book {}: {}", book_id, url));

    let body = match http_get(&url) {
        Some(b) => b,
        None => {
            log_error(&format!("failed to fetch tracks for book {}", book_id));
            return vec![];
        }
    };

    let tracks = match parse_audiotracks(&body) {
        Some(t) => t,
        None => {
            log_error(&format!("failed to parse tracks for book {}", book_id));
            return vec![];
        }
    };

    if let Ok(cache_data) = serde_json::to_string(&tracks) {
        kv_set(&cache_key, &cache_data);
    }

    tracks
}

// ============================================================
// Plugin exports
// ============================================================

#[no_mangle]
pub extern "C" fn describe() -> u64 {
    let desc = Descriptor {
        r#type: "librivox",
        label: "LibriVox Audiobooks",
        short_label: "BOOKS",
        color: "#8d6e63",
        version: "1.0.0",
        description: "Free public domain audiobooks from LibriVox, with direct MP3 playback via Internet Archive",
        config_fields: vec![
            serde_json::json!({
                "key": "language",
                "label": "Language",
                "type": "select",
                "required": false,
                "default": "English",
                "options": [
                    {"label": "All Languages", "value": "all"},
                    {"label": "English", "value": "English"},
                    {"label": "German", "value": "German"},
                    {"label": "French", "value": "French"},
                    {"label": "Spanish", "value": "Spanish"},
                    {"label": "Chinese", "value": "Chinese"},
                    {"label": "Russian", "value": "Russian"},
                    {"label": "Italian", "value": "Italian"},
                    {"label": "Portuguese", "value": "Portuguese"},
                    {"label": "Dutch", "value": "Dutch"},
                    {"label": "Japanese", "value": "Japanese"}
                ]
            }),
            serde_json::json!({
                "key": "limit",
                "label": "Max Books",
                "type": "select",
                "required": false,
                "default": "25",
                "options": [
                    {"label": "10", "value": "10"},
                    {"label": "25", "value": "25"},
                    {"label": "50", "value": "50"},
                    {"label": "100", "value": "100"}
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
                "id": "search_books",
                "label": "Search Books",
                "type": "search",
                "target_field": "title"
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

    let language = config
        .get("language")
        .and_then(|v| v.as_str())
        .unwrap_or("English");

    let limit: u32 = config
        .get("limit")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse().ok())
        .unwrap_or(25);

    let books = fetch_books_cached(limit, language);

    if books.is_empty() {
        log_info("no books found");
        return return_json(&RefreshResponse { streams: vec![] });
    }

    let mut streams: Vec<Stream> = Vec::new();

    for book in &books {
        let tracks = fetch_tracks_cached(&book.id);

        for track in &tracks {
            if track.listen_url.is_empty() {
                continue;
            }
            streams.push(track_to_stream(book, track));
        }
    }

    log_info(&format!(
        "refresh complete: {} streams from {} books",
        streams.len(),
        books.len()
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

    if req.action != "search_books" {
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

    let url = search_url(query, 10);
    log_info(&format!("searching books: {}", url));

    let body = match http_get(&url) {
        Some(b) => b,
        None => {
            log_error("search request failed");
            let empty: Vec<Value> = vec![];
            return return_json(&serde_json::json!({ "results": empty }));
        }
    };

    let books = match parse_audiobooks(&body) {
        Some(b) => b,
        None => {
            log_error("failed to parse search results");
            let empty: Vec<Value> = vec![];
            return return_json(&serde_json::json!({ "results": empty }));
        }
    };

    let mut streams: Vec<Stream> = Vec::new();

    for book in &books {
        let tracks = fetch_tracks_cached(&book.id);
        for track in &tracks {
            if track.listen_url.is_empty() {
                continue;
            }
            streams.push(track_to_stream(book, track));
        }
    }

    return_json(&RefreshResponse { streams })
}
