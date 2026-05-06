# SkylineWebcams Plugin Plan

## Feasibility: CONFIRMED

SkylineWebcams serves **real HLS video streams** (not JPEG refreshes). Each webcam page
embeds a Clappr player with a `.m3u8` source URL. No API key is required.

## Technical Findings

### Stream delivery
- **Player**: Clappr (open-source HTML5 player)
- **Format**: HLS via `livee.m3u8?a=<token>` (relative URL on skylinewebcams.com)
- **Full URL pattern**: `https://www.skylinewebcams.com/livee.m3u8?a=<token>`
- **Token**: The `?a=` parameter appears to be a static per-camera session token,
  embedded directly in the HTML (not fetched via a secondary API call).
- **nkey**: Each cam has a numeric ID (e.g., `286` for Trevi Fountain) visible as
  `nkey:'286.jpg'` in the player init.

### Discovery (no public API)
- **No documented API**. Webcam catalog must be scraped from HTML pages.
- **Country index**: `/en/webcam/<country>.html` lists all cams for a country.
- **Category index**: `/en/live-cams-category/<category>.html` (city-cams, beach-cams,
  ski-cams, etc.)
- **Top cams**: `/en/top-live-cams.html` (~100 featured cams)
- **robots.txt**: Fully permissive for general user agents (only blocks two specific bots).
- **URL pattern**: `/en/webcam/<country>/<region>/<city>/<slug>.html`
- **Stats endpoint**: `https://cdn.skylinewebcams.com/<id>.json` returns
  `{"t":"<total_views>","n":"<online_now>"}` (lightweight, no auth).

### Metadata available per cam page
- Title, description, location hierarchy (country/region/city)
- View count, current viewer count, rating
- Thumbnail: `https://cdn.skylinewebcams.com/live<id>.jpg`

## Plugin Design (Rust WASM)

### Language & pattern
Rust, following the radiogarden plugin pattern: `describe()`, `refresh()`, `interact()` exports
with `host_http_request`, `host_log`, `host_kv_get/set` host imports.

### describe()
```
type: "skylinecams"
label: "SkylineWebcams"
short_label: "SKYLINE"
color: "#0288d1"
layout: "grouped_list"
group_by: "group" (country or category)
searchable: true
```

Config fields:
- `mode`: enum ["top", "country", "category"] -- which index to scrape
- `countries`: multi-select of country slugs (when mode=country)
- `categories`: multi-select of category slugs (when mode=category)

### refresh(config)
1. Based on `mode`, fetch the appropriate index page(s):
   - `top` -> fetch `/en/top-live-cams.html`
   - `country` -> fetch `/en/webcam/<country>.html` for each selected country
   - `category` -> fetch `/en/live-cams-category/<cat>.html` for each selected category
2. Parse HTML to extract webcam links (regex or simple string scanning for
   `/en/webcam/...html` href patterns -- no full DOM parser needed in WASM).
3. For each discovered cam URL, fetch the cam page HTML.
4. Extract from the Clappr init JS:
   - `source:'livee.m3u8?a=<token>'` via regex
   - `nkey:'<id>.jpg'` via regex
   - Title from `<title>` or `<h1>` tag
5. Construct full stream URL: `https://www.skylinewebcams.com/livee.m3u8?a=<token>`
6. Group by country (extracted from URL path segment).
7. Cache cam list in KV store to avoid re-scraping on every refresh.
8. Return streams with `vod_type: "live"`.

### Stream struct
```rust
Stream {
    id: "<numeric_id>",           // from nkey
    name: "Trevi Fountain",       // from page title
    url: "https://www.skylinewebcams.com/livee.m3u8?a=<token>",
    group: "Italy",               // country from URL path
    logo: "https://cdn.skylinewebcams.com/live<id>.jpg",
    vod_type: "live",
    tags: ["city", "rome"],       // optional: from URL segments
}
```

### interact() -- optional search
- `search_cams`: text search across cached cam titles/locations.

### Rate limiting / caching
- Index pages change infrequently. Cache cam list in KV for 24 hours.
- Individual cam pages only need fetching once to extract the m3u8 token
  (tokens appear stable). Cache token per cam ID.
- Limit concurrent fetches: process cam pages sequentially in the WASM
  (host controls actual HTTP concurrency).

## Risks and mitigations

| Risk | Mitigation |
|------|-----------|
| m3u8 tokens may rotate over time | Re-scrape cam pages periodically (e.g., daily). If a stream 404s, trigger a re-fetch of that cam page. |
| HTML structure changes break parsing | Use resilient regex patterns; log parse failures for debugging. |
| Large number of cams per country (Italy has 500+) | Limit initial fetch to top-cams or allow user to select specific countries. Paginate fetches. |
| No structured API means slower discovery | Aggressive KV caching of the full cam catalog after first scrape. |
| Geo-blocking on some streams | Nothing we can do; document as known limitation. |

## File structure
```
plugins/skylinecams/
  Cargo.toml
  src/lib.rs
  README.md
```

## Estimated complexity
Medium. The main challenge is HTML parsing without a full DOM parser in WASM.
Simple regex extraction of href patterns and Clappr source values should suffice
since the HTML structure is consistent across pages.
