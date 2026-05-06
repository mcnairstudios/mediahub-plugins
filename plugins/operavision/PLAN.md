# OperaVision Plugin Plan

## Feasibility: CONFIRMED

OperaVision (operavision.eu) streams free opera, ballet, and concert performances from 44 European opera houses. All video content is hosted on YouTube and embedded dynamically on the OperaVision site via their Drupal 10 CMS. No API keys are required.

## Data Sources

### Primary: YouTube "Performances" Playlist
- **Playlist ID:** `PLYAQ82KNFI_cYU0EtHMphfNgZ8-h2N-S-`
- Contains all currently available full performances as unlisted YouTube videos
- Playlist is publicly accessible and scrapeable without authentication
- Each entry provides: video ID, title, duration
- Titles follow a consistent format: `TITLE Composer -- Opera House`

### Secondary: YouTube Channel RSS Feed
- **Channel ID:** `UCBTlXPAfOx300RZfWNw8-qg`
- **RSS URL:** `https://www.youtube.com/feeds/videos.xml?channel_id=UCBTlXPAfOx300RZfWNw8-qg`
- Returns the 15 most recent uploads (trailers, extracts, full performances mixed)
- Useful for detecting newly added content

### Supplementary: OperaVision Website
- `https://operavision.eu/performances` lists all performances with metadata
- URL pattern: `/performance/{slug}`
- Provides: title, opera house, genre context, thumbnail images
- Drupal backend; no public REST/JSON:API endpoints exposed
- Video player (YouTube embed) is loaded dynamically via JS, not in server-rendered HTML

## Architecture

### Language
Rust, compiled to WASM. Follow the pattern in `plugins/radiogarden/src/lib.rs`.

### Stream Model
Each stream maps to a YouTube video:

```
Stream {
    id: YouTube video ID (e.g., "LMn6rDScs_Y"),
    name: Performance title (e.g., "The Corsair - Estonian National Ballet"),
    url: "https://www.youtube.com/watch?v={video_id}",
    group: Opera house / company name (parsed from title after " -- "),
    logo: YouTube thumbnail URL "https://i.ytimg.com/vi/{video_id}/hqdefault.jpg",
    vod_type: "youtube",
    tags: [genre if detectable, e.g. "opera", "ballet", "concert"],
}
```

### Refresh Strategy

1. **Fetch the YouTube Performances playlist page:**
   ```
   GET https://www.youtube.com/playlist?list=PLYAQ82KNFI_cYU0EtHMphfNgZ8-h2N-S-
   Headers: Cookie: CONSENT=YES+1
   ```

2. **Parse `ytInitialData` JSON** from the HTML response to extract all playlist items (video ID, title, duration).

3. **Parse titles** to split into performance name, composer, and opera house:
   - Pattern: `TITLE Composer -- Opera House`
   - The delimiter ` \u2013 ` (en-dash) separates title from venue
   - Some titles have no composer (e.g., galas, showcases)

4. **Optionally scrape `operavision.eu/performances`** for additional metadata (streaming dates, genres, partner logos). This is HTML scraping of the Drupal-rendered listing page.

5. **Cache results** using `host_kv_set` to avoid re-fetching on every call.

### Exported Functions

- `describe()` -- Returns plugin descriptor with type "operavision", label "OperaVision", color "#1a237e" (deep blue), layout "grouped_list" grouped by opera house.
- `refresh(config)` -- Fetches playlist, parses entries, returns streams. Config can optionally filter by genre or opera house.
- `interact(action)` -- Optional: search/filter by opera house, composer, or genre.

## Key Technical Notes

- YouTube playlist HTML scraping requires the `CONSENT=YES+1` cookie header to bypass the EU consent wall.
- The playlist page returns up to ~100 items in the initial HTML; a `continuation` token in `ytInitialData` may be needed for pagination if the catalog grows beyond that.
- Full performance videos are typically 1-3.5 hours long.
- Video availability is time-limited (typically 6 months per performance), so the catalog rotates.
- Some performances may be "premiering" (scheduled but not yet watchable); these have a future start date in the YouTube metadata.
- The RSS feed (`/feeds/videos.xml`) mixes full performances, trailers, extracts, and shorts. It is useful as a lightweight check for new content but not as the primary data source.

## Risks and Mitigations

| Risk | Mitigation |
|------|-----------|
| YouTube changes playlist HTML structure | Parse `ytInitialData` JSON (stable for years); fall back to regex extraction of videoId + title pairs |
| Playlist ID changes | Can be discovered by scraping channel playlists page; could also be made a config field |
| Title format changes | Fuzzy parsing with fallback to using full title as name and "OperaVision" as group |
| Consent wall changes | Cookie-based bypass; if that fails, the RSS feed provides a partial fallback |
| Rate limiting | Cache playlist data for 1-6 hours; content only updates a few times per month |

## Not in Scope

- Extracts/clips (short arias and scenes) -- could be added later via additional playlists
- Behind-the-scenes content -- separate playlist, lower priority
- Podcast content (Opera Road Trip) -- hosted on Libsyn, different media type
- Subtitle/language selection -- handled by the YouTube player natively
