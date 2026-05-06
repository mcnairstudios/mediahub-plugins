# Podcasts Plugin — PLAN

## Plugin metadata
- **Name:** Podcasts
- **Type:** `podcasts`
- **Label:** Podcasts
- **Short label:** POD
- **Language:** Go (TinyGo, consistent with demo plugin)
- **Description:** Search and browse podcast episodes with direct audio playback via the Apple iTunes Search API

## API approach

### Primary API: Apple iTunes Search API

The iTunes Search API is the clear winner for this use case:

- **No API key required** — completely open, no authentication
- **Returns direct MP3 URLs** via the `episodeUrl` field (e.g. `https://traffic.megaphone.fm/FSI5518908653.mp3`)
- **Two useful endpoints:**
  1. **Episode search:** `https://itunes.apple.com/search?term={query}&media=podcast&entity=podcastEpisode&limit=50`
     - Returns episodes matching a search term, each with `episodeUrl` (playable MP3)
  2. **Podcast lookup + episodes:** `https://itunes.apple.com/lookup?id={collectionId}&media=podcast&entity=podcastEpisode&limit=200`
     - Returns up to 200 recent episodes for a specific podcast, each with `episodeUrl`
- **No RSS/XML parsing needed** — JSON responses with direct audio URLs
- **Rich metadata:** artwork (multiple sizes), descriptions, durations, genres, release dates

### Why NOT Podcast Index API

- **Requires API key** — confirmed by 403 response when calling without credentials
- Keys are free but require registration at podcastindex.org
- Even though it likely returns `enclosureUrl` fields, the key requirement disqualifies it under our "no API keys" constraint
- Would be a good secondary option if the key policy is acceptable in the future

### Why NOT RSS parsing

- Searching for podcasts (entity=podcast) on iTunes returns `feedUrl` (RSS) but no episode audio URLs
- Parsing RSS/XML in WASM is possible but adds significant complexity and binary size
- Completely unnecessary since the episode search endpoint returns MP3 URLs directly

## Stream mapping

Each podcast **episode** maps to one stream:

| Stream field   | Source                                                        |
|----------------|---------------------------------------------------------------|
| `id`           | `trackId` (unique episode ID from iTunes)                     |
| `name`         | `trackName` (episode title)                                   |
| `url`          | `episodeUrl` (direct MP3 link)                                |
| `group`        | `collectionName` (podcast name — groups episodes by show)     |
| `logo`         | `artworkUrl600` (600x600 podcast artwork)                     |
| `tags`         | `genres` array from response                                  |
| `vod_type`     | `"podcast"`                                                   |
| `episode_name` | `trackName` (same as name, or could use `shortDescription`)   |

## Config fields

| Key          | Label            | Type     | Required | Default       | Notes                                          |
|--------------|------------------|----------|----------|---------------|-------------------------------------------------|
| `searches`   | Search Terms     | `text`   | No       | `""`          | Comma-separated search terms to fetch episodes  |
| `podcast_ids`| Podcast IDs      | `text`   | No       | `""`          | Comma-separated iTunes collection IDs to follow |
| `limit`      | Episodes per source | `select` | No    | `"25"`        | 10, 25, 50, 100, 200                           |

### User workflow
1. User enters search terms (e.g. "true crime, tech news, comedy")
2. Plugin calls episode search for each term, collects results
3. Optionally, user provides specific podcast collection IDs for "subscribed" shows
4. Plugin calls lookup endpoint for each ID to get latest episodes
5. All episodes are returned as streams, grouped by podcast name

## Estimated stream count

- Each search term returns up to `limit` episodes (default 25)
- Each subscribed podcast ID returns up to `limit` episodes
- With 4 search terms and 5 subscribed podcasts at limit=25: ~225 streams
- Maximum practical limit: ~1000-2000 streams before response gets unwieldy

## View config

```json
{
  "layout": "grouped_list",
  "group_by": "group",
  "searchable": true,
  "sortable": true
}
```

## Implementation notes

- Use `host_http_request` to call iTunes API endpoints
- Parse JSON responses (straightforward in Go, no external dependencies)
- Deduplicate episodes that appear in multiple search results (by `trackId`)
- Use `host_kv_set/get` to cache results and reduce API calls on repeated refreshes
- The `interact` export can be minimal initially (no special actions needed)

## Risks

| Risk | Severity | Mitigation |
|------|----------|------------|
| iTunes API rate limiting | Medium | Apple does not document rate limits, but aggressive use could be throttled. Use KV caching to reduce calls. |
| `episodeUrl` availability | Low | Some episodes may have missing or expired URLs. Filter these out during refresh. |
| Apple may change/deprecate the API | Low | The iTunes Search API has been stable for 10+ years with no announced deprecation. |
| Limited to Apple-indexed podcasts | Low | Apple Podcasts indexes millions of podcasts; coverage is excellent. |
| No real-time/live podcast streams | N/A | This plugin is for on-demand episode playback, not live audio. |
| WASM binary size | Low | Only needs JSON parsing (stdlib), no XML/RSS dependencies. Should be small. |

## Verdict

**Feasible.** The Apple iTunes Search API meets all three feasibility criteria:

1. **No API keys** — completely open, unauthenticated access
2. **Direct playable MP3 URLs** — `episodeUrl` field in every episode result
3. **WASM-compatible** — JSON-only responses, no complex dependencies needed

This is one of the cleanest API fits for the plugin pattern. The JSON responses map almost 1:1 to the stream model, and the two endpoints (search + lookup) provide both discovery and subscription-style browsing.
