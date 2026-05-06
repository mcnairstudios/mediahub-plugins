# ccMixter Plugin Plan

## Feasibility Assessment: FEASIBLE

### Evidence

- **API Endpoint**: `http://ccmixter.org/api/query?f=json` -- returns well-structured JSON, no API key required.
- **Content volume**: 70,000+ uploads (upload IDs reach 70835 as of May 2026). Easily exceeds the 100-track minimum.
- **Direct audio URLs**: Each upload includes a `files` array with `download_url` fields pointing to MP3/FLAC files. URLs follow the pattern `https://ccmixter.org/content/{user}/{filename}.mp3`.
- **Hotlink protection**: Direct audio requests return 403 unless a `Referer: https://ccmixter.org/` header is sent. With that header, responses return 200 with `Content-Type: audio/mpeg`. The WASM host's `host_http_request` supports custom headers, so this is not a blocker.
- **Filtering/search**: The API supports `tags=`, `search=`, `sort=`, `limit=`, `offset=` query parameters.
- **Licensing**: All content is Creative Commons licensed. License name and URL are included in every API response.

### API Response Structure (key fields per upload)

```
upload_id          -- unique ID
upload_name        -- track title
user_name          -- artist username
user_real_name     -- artist display name
upload_tags        -- comma-separated tag string
upload_extra.usertags -- user-defined tags (genres, instruments)
upload_extra.bpm   -- BPM (integer)
license_name       -- e.g. "Attribution Noncommercial (4.0)"
license_url        -- CC license URL
file_page_url      -- web page for the upload
artist_page_url    -- artist profile URL
upload_date_format -- human-readable date
upload_description_plain -- text description
files[]            -- array of associated files:
  file_name        -- filename
  file_nicname     -- label (e.g. "mp3", "Synth", "Bass")
  file_format_info.mime_type   -- e.g. "audio/mpeg", "audio/x-flac"
  file_format_info.ps          -- duration string (e.g. "3:34")
  download_url     -- direct download URL
  file_rawsize     -- size in bytes
```

## Plugin Design

### Language
Rust (targeting `wasm32-unknown-unknown`), following the radiogarden plugin pattern.

### Exported Functions

1. **`describe() -> u64`** -- Returns plugin descriptor.
2. **`refresh(config_ptr, config_len) -> u64`** -- Fetches tracks from ccMixter API, returns stream list.
3. **`interact(action_ptr, action_len) -> u64`** -- Handles search interactions.

### Descriptor

| Field        | Value                                                   |
|--------------|---------------------------------------------------------|
| type         | `"ccmixter"`                                            |
| label        | `"ccMixter"`                                            |
| short_label  | `"CCMIX"`                                               |
| color        | `"#7b1fa2"` (purple, matching CC remix branding)        |
| description  | `"Creative Commons remixes, samples, and music"`        |
| layout       | `"grouped_list"`                                        |
| group_by     | `"group"` (group by tag category: remix / sample / a cappella) |
| searchable   | `true`                                                  |
| sortable     | `true`                                                  |

### Config Fields

```json
[
  {
    "key": "tags",
    "label": "Filter by tags",
    "type": "text",
    "default": "remix"
  },
  {
    "key": "limit",
    "label": "Number of tracks",
    "type": "number",
    "default": 50
  }
]
```

### Refresh Logic

1. Read `tags` and `limit` from config (defaults: `"remix"`, `50`).
2. Build URL: `http://ccmixter.org/api/query?f=json&tags={tags}&limit={limit}&sort=date`.
3. Call `http_get()` to fetch JSON.
4. Parse response into a `Vec` of upload objects.
5. For each upload, find the first file with `mime_type` = `"audio/mpeg"` (MP3 preferred).
6. Map to `Stream` struct:
   - `id`: `upload_id` as string
   - `name`: `upload_name`
   - `url`: `download_url` of the MP3 file
   - `group`: Derive from `upload_extra.ccud` field (e.g. "remix", "sample", "a_cappella")
   - `logo`: `license_logo_url`
   - `tags`: Parse from `upload_extra.usertags`
7. Return `RefreshResponse { streams }`.

### Interact Logic (search)

- Action `"search_tracks"`: Takes a `query` param.
- Calls `http://ccmixter.org/api/query?f=json&search={query}&limit=20&sort=rank`.
- Returns search results with `id`, `title` (track name), `subtitle` (artist + license).

### HTTP Headers

All requests to ccMixter audio URLs require:
```json
{
  "User-Agent": "Mozilla/5.0",
  "Referer": "https://ccmixter.org/"
}
```
API query requests work without special headers, but including them is good practice.

### Stream struct

```rust
struct Stream {
    id: String,       // upload_id
    name: String,     // upload_name (track title)
    url: String,      // download_url of primary MP3 file
    group: String,    // content type: "Remixes", "Samples", "A Cappellas"
    logo: String,     // license_logo_url
    vod_type: String, // empty (on-demand audio, not live)
    tags: Vec<String>, // from usertags
}
```

### Risks and Mitigations

| Risk | Mitigation |
|------|------------|
| SSL certificate issues (observed during testing) | Use `http://` for API queries; audio URLs work over HTTPS with proper headers |
| Hotlink protection changes | Referer header is standard; if it breaks, fall back to `file_page_url` |
| API rate limiting (unknown) | Respect reasonable limits; cache results via `host_kv_set` |
| Some uploads have no MP3 (FLAC only) | Fall back to first available audio file |

### File Structure

```
plugins/ccmixter/
  Cargo.toml
  src/lib.rs
  PLAN.md
```

### Dependencies (Cargo.toml)

- `serde` (with derive feature)
- `serde_json`

Same as the radiogarden plugin. Target: `wasm32-unknown-unknown`, crate-type: `["cdylib"]`.
