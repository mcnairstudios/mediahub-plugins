# PeerTube Plugin Plan

## Plugin metadata

- **Name:** peertube
- **Label:** PeerTube
- **Short label:** PEER
- **Description:** Browse and play videos from the federated PeerTube network via Sepia Search
- **Language:** Go (TinyGo) -- consistent with most existing plugins
- **Version:** 1.0.0

## API endpoints used

1. **Sepia Search (cross-instance discovery)**
   `GET https://search.joinpeertube.org/api/v1/search/videos?search={query}&count={n}`
   - No authentication required
   - Returns video metadata including `url` (watch page on the hosting instance) and `uuid`
   - The `url` field encodes the hosting instance domain (e.g. `https://tube.xy-space.de/videos/watch/{uuid}`)

2. **Per-instance video detail**
   `GET https://{instance}/api/v1/videos/{uuid}`
   - No authentication required for public videos
   - Returns full video detail including:
     - `files[]` array with `fileUrl` (direct MP4 links at various resolutions: 1080p, 480p, 360p)
     - `streamingPlaylists[]` array with `playlistUrl` (HLS master.m3u8 playlist)
   - HLS playlists are preferred as they support adaptive bitrate

## How to get playable video URLs

1. Call Sepia Search with a search term (or use a curated instance list endpoint)
2. For each result, parse the instance hostname and video UUID from the `url` field
3. Call the per-instance detail endpoint: `https://{instance}/api/v1/videos/{uuid}`
4. Extract the HLS master playlist URL from `streamingPlaylists[0].playlistUrl` (preferred)
5. Fall back to the highest-resolution MP4 from `files[]` sorted by resolution descending

The two-step fetch (search then detail) is necessary because the search/list endpoints do not include file URLs.

## Grouping strategy

Group by **instance hostname** (e.g. "framatube.org", "tube.tchncs.de"). This naturally reflects the federated structure and keeps videos from the same community together.

Alternative: group by **category** if the search results include category metadata.

## Config fields

| Key | Label | Type | Required | Default |
|-----|-------|------|----------|---------|
| `search_terms` | Search terms (comma-separated) | text | no | "documentary,music,science,technology" |
| `max_results` | Max results per search term | text | no | "10" |

Minimal config -- works out of the box with sensible defaults. Users can customize search terms to curate their feed.

## Estimated stream count

With 4 default search terms at 10 results each: approximately 20-40 unique streams per refresh (some deduplication expected across search terms).

## Risks

1. **Two-step fetch latency:** Each video requires a second HTTP call to its hosting instance to get the playable URL. With 40 videos this means 40+ HTTP requests per refresh. Mitigation: use `host_kv_set/get` to cache video detail responses.
2. **Instance availability:** PeerTube instances are community-run and may go offline. Some detail requests may fail. Mitigation: skip failed fetches gracefully, log warnings.
3. **Rate limiting:** Individual PeerTube instances may rate-limit API requests. Mitigation: spread requests across instances (natural with federated search), respect reasonable concurrency.
4. **Sepia Search availability:** The central Sepia Search index could go down. Mitigation: could fall back to querying a hardcoded list of popular instances directly.
5. **Content moderation:** Federated search may surface NSFW content. PeerTube tags videos with `nsfw: true` -- the plugin should filter these out by default.

## API key requirement

**NO** -- all PeerTube APIs used are public and require no authentication.
