# NASA Video Library Plugin - Plan

**Language:** Rust (WASM)
**Source:** NASA Image and Video Library API (images-api.nasa.gov)
**Feasibility:** CONFIRMED

## Feasibility Assessment

### Sources Evaluated

| Source | Type | Verdict |
|--------|------|---------|
| NASA APOD | ~95% images, ~5% video (YouTube embeds) | NOT FEASIBLE - too few videos |
| NASA EPIC | Images only (Earth photos) | NOT FEASIBLE |
| NASA Live | HLS stream, already in demo plugin | SKIP |
| NASA Video Library | ~7,000+ videos with direct MP4 URLs | FEASIBLE |

### NASA Video Library - Key Findings

- **No API key required.** The API at `images-api.nasa.gov` is fully public with no authentication.
- **Large video catalog:** 6,961 results for "space", 3,310 for "launch", 2,515 for "earth", 2,149 for "moon", 1,568 for "mars", 1,091 for "ISS".
- **Direct MP4 playback:** Each video has multiple quality variants (orig, medium, mobile, small, preview) served from `images-assets.nasa.gov` (S3-backed, `Content-Type: video/mp4`).
- **No rate limiting documented** for the public search/asset endpoints (unlike APOD's DEMO_KEY throttling).
- **Two-step URL resolution:** Search returns `nasa_id` values. A second call to `/asset/{nasa_id}` returns direct MP4 links.
- **Thumbnail images** available at multiple sizes (thumb, small, medium, large) directly from search results.
- **Captions** available as `.srt`/`.vtt` files for many videos.

## Plugin Design

### API Flow

1. **Search endpoint:** `GET https://images-api.nasa.gov/search?q={query}&media_type=video&page={n}`
   - Returns 100 items per page with `nasa_id`, title, description, date, center, keywords.
   - Paginated via `links[rel=next]`.

2. **Asset endpoint:** `GET https://images-api.nasa.gov/asset/{nasa_id}`
   - Returns list of file URLs including multiple MP4 quality variants.
   - Plugin should prefer `~mobile.mp4` (small, fast) or `~medium.mp4` for playback.

### Descriptor

```
type: "nasa"
label: "NASA Videos"
short_label: "NASA"
color: "#0B3D91"  (NASA blue)
layout: "grouped_list"
group_by: "group"  (NASA center or keyword category)
searchable: true
sortable: true
```

### Config Fields

- **categories** (multi-select): Predefined search terms the user picks from:
  - "Launch", "ISS", "Mars", "Moon", "Earth", "Hubble", "Webb", "Artemis", "Apollo", "Perseverance"
- **max_per_category** (number, default 50): How many videos per category to fetch.

### Refresh Logic

For each configured category:
1. Call search endpoint with `q={category}&media_type=video`, fetch up to `max_per_category / 100` pages.
2. For each result, construct a predictable MP4 URL using the pattern:
   `https://images-assets.nasa.gov/video/{nasa_id}/{nasa_id}~mobile.mp4`
   (This avoids calling `/asset/` for every single video -- the URL pattern is consistent.)
3. Build stream entries:
   - **id:** `nasa_id`
   - **name:** title (truncated if needed)
   - **url:** direct MP4 URL (`~mobile.mp4` variant)
   - **group:** category name (e.g., "Launch", "Mars")
   - **logo:** thumbnail from `https://images-assets.nasa.gov/video/{nasa_id}/{nasa_id}~thumb.jpg`
   - **vod_type:** `"vod"` (these are on-demand videos, not live)
   - **tags:** keywords from API metadata

### Interactions

- **search_videos** (type: "search"): Free-text search against the NASA API, returns matching video titles for the user to browse or add.

### Optimization Notes

- The MP4 URL pattern (`{nasa_id}~mobile.mp4`) is deterministic, so we can skip the `/asset/` call entirely during refresh. This cuts API calls from `N + N` to just `N` (one search call per page).
- Use KV cache for search results to avoid re-fetching on every refresh.
- Pagination: fetch only 1-2 pages per category to keep refresh fast (100-200 videos per category, up to ~2000 total across 10 categories).

### File Structure

```
plugins/nasa/
  Cargo.toml
  src/
    lib.rs
  README.md
```

### Dependencies (Cargo.toml)

- `serde` + `serde_json` (JSON parsing, same as radiogarden)
- Target: `wasm32-wasip1`

## Risks and Mitigations

| Risk | Mitigation |
|------|-----------|
| URL pattern assumption breaks for some videos | Fall back to `/asset/{id}` call if direct URL returns 404 |
| Large number of API calls during refresh | Limit pages fetched; cache results in KV store |
| Some MP4s may be very large (orig can be GBs) | Always use `~mobile.mp4` variant (typically 2-10 MB) |
| NASA API could add rate limiting | Respect any `Retry-After` headers; add backoff |
