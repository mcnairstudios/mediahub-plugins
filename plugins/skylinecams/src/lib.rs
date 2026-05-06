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
    let headers = b"{\"User-Agent\":\"Mozilla/5.0 (compatible; MediaHub/1.0)\"}";

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

#[derive(Serialize)]
struct RefreshResponse {
    streams: Vec<Stream>,
}

#[derive(Serialize, Deserialize, Clone)]
struct Stream {
    id: String,
    name: String,
    url: String,
    group: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    logo: Option<String>,
    vod_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    tags: Option<Vec<String>>,
}

// ============================================================
// Cached cam data
// ============================================================

#[derive(Serialize, Deserialize, Clone)]
struct CamInfo {
    id: String,
    name: String,
    url: String,
    group: String,
    thumbnail: String,
    tags: Vec<String>,
}

// ============================================================
// Config types
// ============================================================

#[derive(Deserialize)]
struct Config {
    #[serde(default = "default_mode")]
    mode: String,
    #[serde(default)]
    countries: Vec<String>,
    #[serde(default)]
    categories: Vec<String>,
}

fn default_mode() -> String {
    "top".to_string()
}

// ============================================================
// HTML parsing helpers (pure functions, testable without host)
// ============================================================

/// Base URL for SkylineWebcams.
const BASE_URL: &str = "https://www.skylinewebcams.com";

/// Extract webcam page links from an index page HTML.
/// Looks for href="/en/webcam/..." patterns and returns unique relative paths.
pub fn extract_cam_links(html: &str) -> Vec<String> {
    let mut links = Vec::new();
    let mut seen = std::collections::HashSet::new();
    let pattern = "href=\"/en/webcam/";

    let mut search_from = 0;
    while let Some(pos) = html[search_from..].find(pattern) {
        let abs_pos = search_from + pos;
        let href_start = abs_pos + 6; // skip `href="`
        if let Some(end_quote) = html[href_start..].find('"') {
            let path = &html[href_start..href_start + end_quote];
            // Only accept paths that end with .html and have multiple segments
            // (skip category/country index links that don't point to a specific cam)
            if path.ends_with(".html") && path.matches('/').count() >= 5 {
                if seen.insert(path.to_string()) {
                    links.push(path.to_string());
                }
            }
        }
        search_from = abs_pos + pattern.len();
    }

    links
}

/// Extract the HLS token from a cam page HTML.
/// Searches for `source:'livee.m3u8?a=TOKEN'` or `source: 'livee.m3u8?a=TOKEN'`
/// or `url: '...livee.m3u8?a=TOKEN'` patterns in the Clappr player init JS.
pub fn extract_hls_token(html: &str) -> Option<String> {
    // Try multiple patterns that SkylineWebcams uses for embedding the stream URL
    let patterns = [
        "livee.m3u8?a=",
        "livee.m3u8?a\\x3d",  // URL-encoded '=' in some JS contexts
    ];

    for pattern in &patterns {
        if let Some(pos) = html.find(pattern) {
            let token_start = pos + pattern.len();
            // Token ends at the next quote (single or double) or ampersand
            let remaining = &html[token_start..];
            let end = remaining.find(|c: char| c == '\'' || c == '"' || c == '&' || c == '\\' || c.is_whitespace());
            if let Some(end_pos) = end {
                let token = &remaining[..end_pos];
                if !token.is_empty() {
                    return Some(token.to_string());
                }
            } else if !remaining.is_empty() {
                // Token goes to end of string (unlikely but handle gracefully)
                let token = remaining.trim();
                if !token.is_empty() {
                    return Some(token.to_string());
                }
            }
        }
    }

    None
}

/// Extract the cam numeric ID (nkey) from cam page HTML.
/// Looks for `nkey:'123.jpg'` or `nkey: '123.jpg'` patterns.
pub fn extract_nkey(html: &str) -> Option<String> {
    let pattern = "nkey:";
    if let Some(pos) = html.find(pattern) {
        let after = &html[pos + pattern.len()..];
        // Skip optional whitespace and quote
        let after = after.trim_start();
        let after = if after.starts_with('\'') || after.starts_with('"') {
            &after[1..]
        } else {
            after
        };
        // Read until .jpg or end of token
        if let Some(dot_pos) = after.find(".jpg") {
            let id = &after[..dot_pos];
            if !id.is_empty() && id.chars().all(|c| c.is_ascii_digit()) {
                return Some(id.to_string());
            }
        }
    }
    None
}

/// Extract the page title from HTML.
/// Looks for <title>...</title> or <h1>...</h1> and strips common suffixes.
pub fn extract_title(html: &str) -> Option<String> {
    // Try <title> tag first
    if let Some(title) = extract_tag_content(html, "<title>", "</title>") {
        // Strip common suffixes like " | SkylineWebcams" or " - SkylineWebcams"
        let cleaned = title
            .split(" | ")
            .next()
            .unwrap_or(&title)
            .split(" - SkylineWebcams")
            .next()
            .unwrap_or(&title)
            .trim()
            .to_string();
        if !cleaned.is_empty() {
            return Some(cleaned);
        }
    }

    // Fallback: try <h1> tag
    if let Some(h1) = extract_tag_content(html, "<h1", "</h1>") {
        // Strip any attributes from the opening tag
        let content = if let Some(gt_pos) = h1.find('>') {
            &h1[gt_pos + 1..]
        } else {
            &h1
        };
        // Strip inner HTML tags
        let cleaned = strip_html_tags(content).trim().to_string();
        if !cleaned.is_empty() {
            return Some(cleaned);
        }
    }

    None
}

/// Extract content between two delimiters in HTML.
fn extract_tag_content<'a>(html: &'a str, open: &str, close: &str) -> Option<String> {
    let start = html.find(open)?;
    let content_start = start + open.len();
    let end = html[content_start..].find(close)?;
    Some(html[content_start..content_start + end].to_string())
}

/// Strip HTML tags from a string, keeping only text content.
fn strip_html_tags(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut in_tag = false;
    for c in s.chars() {
        if c == '<' {
            in_tag = true;
        } else if c == '>' {
            in_tag = false;
        } else if !in_tag {
            result.push(c);
        }
    }
    result
}

/// Extract the country (group) from a cam URL path like `/en/webcam/italy/lazio/roma/trevi.html`.
/// Returns a capitalized country name (e.g., "Italy").
pub fn extract_country_from_path(path: &str) -> String {
    // Path format: /en/webcam/<country>/<region>/<city>/<slug>.html
    let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    // segments: ["en", "webcam", "country", "region", "city", "slug.html"]
    if segments.len() >= 3 {
        let country_slug = segments[2];
        capitalize_slug(country_slug)
    } else {
        "Other".to_string()
    }
}

/// Convert a URL slug like "united-states" into "United States".
fn capitalize_slug(slug: &str) -> String {
    slug.split('-')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => {
                    let upper: String = c.to_uppercase().collect();
                    upper + &chars.collect::<String>()
                }
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Extract tags from URL path segments (region, city).
pub fn extract_tags_from_path(path: &str) -> Vec<String> {
    let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    let mut tags = Vec::new();
    // segments: ["en", "webcam", "country", "region", "city", "slug.html"]
    if segments.len() >= 5 {
        // region
        tags.push(capitalize_slug(segments[3]));
        // city
        tags.push(capitalize_slug(segments[4]));
    }
    tags
}

/// Build the full HLS stream URL from a token.
pub fn build_hls_url(token: &str) -> String {
    format!("{}/livee.m3u8?a={}", BASE_URL, token)
}

/// Build the thumbnail URL from a cam numeric ID.
pub fn build_thumbnail_url(nkey: &str) -> String {
    format!("https://cdn.skylinewebcams.com/live{}.jpg", nkey)
}

/// Build index page URL based on mode.
fn build_index_urls(config: &Config) -> Vec<(String, String)> {
    // Returns (url, group_hint) pairs
    match config.mode.as_str() {
        "country" => {
            let countries = if config.countries.is_empty() {
                vec!["italy".to_string()]
            } else {
                config.countries.clone()
            };
            countries
                .iter()
                .map(|c| {
                    (
                        format!("{}/en/webcam/{}.html", BASE_URL, c),
                        capitalize_slug(c),
                    )
                })
                .collect()
        }
        "category" => {
            let categories = if config.categories.is_empty() {
                vec!["beach-cams".to_string()]
            } else {
                config.categories.clone()
            };
            categories
                .iter()
                .map(|c| {
                    (
                        format!("{}/en/live-cams-category/{}.html", BASE_URL, c),
                        capitalize_slug(c),
                    )
                })
                .collect()
        }
        _ => {
            // Default: top live cams
            vec![(
                format!("{}/en/top-live-cams.html", BASE_URL),
                "Top Cams".to_string(),
            )]
        }
    }
}

/// Fetch and parse a single cam page, returning CamInfo if successful.
fn fetch_cam_info(cam_path: &str, group_hint: &str) -> Option<CamInfo> {
    let url = format!("{}{}", BASE_URL, cam_path);
    log_info(&format!("fetching cam page: {}", url));

    let body = match http_get(&url) {
        Some(b) => b,
        None => {
            log_error(&format!("failed to fetch cam page: {}", url));
            return None;
        }
    };

    let html = String::from_utf8_lossy(&body).to_string();

    let token = match extract_hls_token(&html) {
        Some(t) => t,
        None => {
            log_error(&format!("no HLS token found in: {}", url));
            return None;
        }
    };

    let nkey = extract_nkey(&html).unwrap_or_default();
    let name = extract_title(&html).unwrap_or_else(|| "Unknown Webcam".to_string());
    let group = extract_country_from_path(cam_path);
    let tags = extract_tags_from_path(cam_path);

    let id = if !nkey.is_empty() {
        nkey.clone()
    } else {
        // Fallback: use slug from URL path
        cam_path
            .rsplit('/')
            .next()
            .unwrap_or("unknown")
            .trim_end_matches(".html")
            .to_string()
    };

    let thumbnail = if !nkey.is_empty() {
        build_thumbnail_url(&nkey)
    } else {
        String::new()
    };

    // Use group_hint for category mode, otherwise derive from path
    let final_group = if group_hint != "Top Cams" && !group_hint.is_empty() && group_hint != group {
        // In category mode, use category as group; in country mode the path-derived group is fine
        group
    } else {
        group
    };

    Some(CamInfo {
        id,
        name,
        url: build_hls_url(&token),
        group: final_group,
        thumbnail,
        tags,
    })
}

/// Convert CamInfo to Stream for the refresh response.
fn cam_to_stream(cam: &CamInfo) -> Stream {
    Stream {
        id: cam.id.clone(),
        name: cam.name.clone(),
        url: cam.url.clone(),
        group: cam.group.clone(),
        logo: if cam.thumbnail.is_empty() { None } else { Some(cam.thumbnail.clone()) },
        vod_type: "live".to_string(),
        tags: if cam.tags.is_empty() { None } else { Some(cam.tags.clone()) },
    }
}

// ============================================================
// Plugin exports
// ============================================================

#[no_mangle]
pub extern "C" fn describe() -> u64 {
    let desc = Descriptor {
        r#type: "skylinecams",
        label: "SkylineWebcams",
        short_label: "SKY",
        color: "#0288d1",
        version: "1.0.0",
        description: "Live HD webcams from cities, beaches, and landmarks worldwide via SkylineWebcams",
        config_fields: vec![
            serde_json::json!({
                "key": "mode",
                "label": "Browse Mode",
                "type": "select",
                "options": [
                    {"value": "top", "label": "Top Live Cams"},
                    {"value": "country", "label": "By Country"},
                    {"value": "category", "label": "By Category"}
                ],
                "default": "top"
            }),
            serde_json::json!({
                "key": "countries",
                "label": "Countries",
                "type": "multi-select",
                "options": [
                    {"value": "italy", "label": "Italy"},
                    {"value": "spain", "label": "Spain"},
                    {"value": "greece", "label": "Greece"},
                    {"value": "united-states", "label": "United States"},
                    {"value": "croatia", "label": "Croatia"},
                    {"value": "france", "label": "France"},
                    {"value": "germany", "label": "Germany"},
                    {"value": "united-kingdom", "label": "United Kingdom"},
                    {"value": "japan", "label": "Japan"},
                    {"value": "mexico", "label": "Mexico"},
                    {"value": "brazil", "label": "Brazil"},
                    {"value": "thailand", "label": "Thailand"},
                    {"value": "turkey", "label": "Turkey"},
                    {"value": "portugal", "label": "Portugal"},
                    {"value": "netherlands", "label": "Netherlands"}
                ],
                "depends_on": {"mode": "country"}
            }),
            serde_json::json!({
                "key": "categories",
                "label": "Categories",
                "type": "multi-select",
                "options": [
                    {"value": "beach-cams", "label": "Beach Cams"},
                    {"value": "city-cams", "label": "City Cams"},
                    {"value": "ski-cams", "label": "Ski Cams"},
                    {"value": "landscape-cams", "label": "Landscape Cams"},
                    {"value": "volcano-cams", "label": "Volcano Cams"},
                    {"value": "airport-cams", "label": "Airport Cams"},
                    {"value": "port-cams", "label": "Port Cams"},
                    {"value": "zoo-cams", "label": "Zoo Cams"},
                    {"value": "underwater-cams", "label": "Underwater Cams"},
                    {"value": "construction-cams", "label": "Construction Cams"}
                ],
                "depends_on": {"mode": "category"}
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
                "id": "search_cams",
                "label": "Search Webcams",
                "type": "search",
                "target_field": "streams"
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
            Config {
                mode: "top".to_string(),
                countries: vec![],
                categories: vec![],
            }
        }
    };

    // Build a cache key based on the config mode and selections
    let cache_key = match config.mode.as_str() {
        "country" => format!("cams_country_{}", config.countries.join(",")),
        "category" => format!("cams_category_{}", config.categories.join(",")),
        _ => "cams_top".to_string(),
    };

    // Try loading from KV cache
    if let Some(cached) = kv_get(&cache_key) {
        if let Ok(cams) = serde_json::from_str::<Vec<CamInfo>>(&cached) {
            if !cams.is_empty() {
                log_info(&format!("loaded {} cams from cache", cams.len()));
                let streams: Vec<Stream> = cams.iter().map(cam_to_stream).collect();
                return return_json(&RefreshResponse { streams });
            }
        }
    }

    let index_urls = build_index_urls(&config);
    let mut all_cams: Vec<CamInfo> = Vec::new();
    let mut seen_ids = std::collections::HashSet::new();

    for (index_url, group_hint) in &index_urls {
        log_info(&format!("fetching index: {}", index_url));

        let body = match http_get(index_url) {
            Some(b) => b,
            None => {
                log_error(&format!("failed to fetch index: {}", index_url));
                continue;
            }
        };

        let html = String::from_utf8_lossy(&body).to_string();
        let cam_paths = extract_cam_links(&html);

        log_info(&format!(
            "found {} cam links on {}",
            cam_paths.len(),
            index_url
        ));

        // Limit to 50 cams per index page to avoid excessive fetching
        let limit = 50.min(cam_paths.len());
        for path in &cam_paths[..limit] {
            if let Some(cam) = fetch_cam_info(path, group_hint) {
                if seen_ids.insert(cam.id.clone()) {
                    all_cams.push(cam);
                }
            }
        }
    }

    log_info(&format!("total cams discovered: {}", all_cams.len()));

    // Cache the results
    if !all_cams.is_empty() {
        if let Ok(cache_data) = serde_json::to_string(&all_cams) {
            kv_set(&cache_key, &cache_data);
            log_info("cached cam data to KV store");
        }
    }

    let streams: Vec<Stream> = all_cams.iter().map(cam_to_stream).collect();
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

    if req.action != "search_cams" {
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

    // Try loading cached cams from all known cache keys
    let cache_keys = ["cams_top", "cams_country_", "cams_category_"];
    let mut all_cams: Vec<CamInfo> = Vec::new();

    // First try the top cams cache (most common)
    if let Some(cached) = kv_get("cams_top") {
        if let Ok(cams) = serde_json::from_str::<Vec<CamInfo>>(&cached) {
            all_cams.extend(cams);
        }
    }

    // Also check if there are any other cached results with known prefixes
    // Since we can't enumerate KV keys, we check the common ones
    let _ = cache_keys; // acknowledged above

    let query_lower = query.to_lowercase();
    let mut results: Vec<Value> = Vec::new();

    for cam in &all_cams {
        if results.len() >= 20 {
            break;
        }
        let name_lower = cam.name.to_lowercase();
        let group_lower = cam.group.to_lowercase();
        let tags_match = cam.tags.iter().any(|t| t.to_lowercase().contains(&query_lower));

        if name_lower.contains(&query_lower)
            || group_lower.contains(&query_lower)
            || tags_match
        {
            results.push(serde_json::json!({
                "id": cam.id,
                "title": cam.name,
                "subtitle": cam.group,
            }));
        }
    }

    return_json(&serde_json::json!({ "results": results }))
}
