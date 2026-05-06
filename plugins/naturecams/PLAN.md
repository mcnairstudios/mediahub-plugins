# Nature Cams Plugin

## Overview

- **Plugin name:** naturecams
- **Label:** Nature Cams
- **Short label:** NATURE
- **Language:** Go (TinyGo, matching existing plugins)
- **Description:** Live wildlife, aquarium, and nature cameras from around the world

## Feasibility: CONFIRMED

Multiple sources provide real, playable video streams (YouTube live streams and direct HLS).
This plugin is feasible with no API keys required.

## Confirmed Video Stream Sources

### 1. Monterey Bay Aquarium (YouTube Live Streams)

**Status: CONFIRMED - real video streams on YouTube**

The Monterey Bay Aquarium runs 5+ simultaneous YouTube live streams daily (7am-7pm Pacific).
Channel ID: `UCnM5iMGiKsZg-iOlIO2ZkdQ`

Known cams (YouTube video IDs change per broadcast):
- Live Sea Otter Cam
- Live Shark Cam
- Live Monterey Bay Cam
- Live Aviary Cam
- Live Open Sea Cam
- Live Kelp Forest Cam

**Discovery method:** The plugin can use the YouTube channel's `/streams` page or the
`youtube.com/embed/live_stream?channel=UCnM5iMGiKsZg-iOlIO2ZkdQ` embed format. However,
for direct playback, the plugin needs individual video IDs. These can be discovered by
scraping the channel page or using the explore.org API approach (see below).

### 2. Explore.org (Wowza HLS + YouTube)

**Status: CONFIRMED - real video streams**

Explore.org operates 130+ cameras worldwide. They have:

- **Primary streaming:** Wowza Streaming Engine servers (e.g., `wowza1-us-central-1-prod.explore.org`).
  The Wowza HLS URL pattern is typically `https://{wowza_fqdn}/live/{stream_name}.smil/playlist.m3u8`,
  but these appear to require specific stream names not exposed in the public API.
- **YouTube channels:** Explore.org runs multiple YouTube sub-channels for live streaming:
  - Explore Live Nature Cams: `UC-2KSeUU5SMCX6XLRD-AEvw`
  - Explore Bears: `UC2Sk0aXLq3ADkH_USGPKT_Q`
  - Explore Oceans: `UCSyg9cb3Iq-NtlbxqNB9wGw`
  - Explore Africa: `UCiGOIXjFqy5_mUNxQNOMfHw`
  - Explore Eagle Nest Cams: `UC8NnosPOvXnm0O1u5YnLQiw`
- **Public API:** `https://explore.org/api/livecams` returns JSON with all cameras, including
  `wowza_fqdn`, `slug`, `title`, `is_offline`, `active` status, thumbnails, and descriptions.
  The API does NOT return YouTube video IDs or direct stream URLs.

**Best approach:** Use the explore.org API to get the camera catalog and metadata, then
construct YouTube watch URLs by scraping the individual cam pages or using the YouTube
Data API. Alternatively, link to `https://explore.org/livecams/{category}/{slug}` as web
URLs (but these aren't directly playable video).

### 3. NASA ISS Live Feed (Direct HLS)

**Status: CONFIRMED - direct HLS m3u8 stream**

- URL: `https://ntv1.akamaized.net/hls/live/2014075/NASA-NTV1-HLS/master.m3u8`
- Valid HLS manifest with 3 quality variants (500k, 700k, 2000k) at 1280x720
- Already used in the demo plugin; always available, no auth required

### 4. Cornell Lab Bird Cams (YouTube Live Streams)

**Status: CONFIRMED - real video streams on YouTube**

Channel ID: `UCZXZQxS3d6NpR-eH_gdDwYA`

The Cornell Lab of Ornithology operates 5-12 bird cameras year-round, streaming live on
YouTube. Includes feeder cams, nest cams, and various locations.

### 5. San Diego Zoo (YouTube Live Streams)

**Status: LIKELY - needs further verification**

San Diego Zoo has live cameras on their website. Need to confirm whether they use YouTube
embeds (likely) or a proprietary player.

### 6. National Park Webcams

**Status: MOSTLY STATIC IMAGES - NOT suitable**

NPS webcams are overwhelmingly static JPEG images that refresh every 30-60 seconds. These
are NOT video streams. A few exceptions exist (e.g., Katmai bear cam is actually an
explore.org camera). National park webcams should be excluded from this plugin.

## Architecture

### Stream Discovery Strategy

The key challenge is that YouTube live stream video IDs change with each new broadcast.
Two approaches:

**Approach A: Channel-ID-based URLs (Recommended)**
- Use YouTube watch URLs: `https://www.youtube.com/watch?v={VIDEO_ID}`
- Discover current video IDs at refresh time by fetching YouTube channel pages
- Cache video IDs with short TTL (1 hour) using `host_kv_get/set`
- Fallback: hardcode known long-running stream IDs as defaults

**Approach B: Curated static list with periodic updates**
- Maintain a hardcoded list of known YouTube video IDs and HLS URLs
- The explore.org API at `https://explore.org/api/livecams` provides the camera catalog
- Supplement with known stable streams (NASA ISS, etc.)
- Risk: YouTube IDs go stale; mitigate with refresh-time validation

**Recommended: Hybrid of A + B.** Use the explore.org API for the camera catalog (names,
descriptions, thumbnails, online/offline status). For playable URLs, maintain a curated
mapping of camera slugs to YouTube video IDs, and attempt to validate/refresh these at
`refresh()` time by fetching the explore.org cam page for each active camera.

### Grouping Strategy

Streams grouped by type:
- **Wildlife** - bears, eagles, wolves, big cats (explore.org)
- **Aquarium** - Monterey Bay Aquarium cams
- **Birds** - Cornell Lab bird cams, explore.org bird cams
- **Ocean** - underwater reef cams, ocean views
- **Space** - NASA ISS live feed
- **Africa** - African wildlife cams (explore.org)

### Config Fields

**None required.** All sources are free and public. No API keys needed.

Optional future config:
- `regions` - filter by geographic region
- `categories` - filter by animal type

### Estimated Stream Count

| Source | Estimated Streams | Notes |
|--------|------------------|-------|
| Explore.org (online) | 15-30 | Varies by season; ~130 total, many offline |
| Monterey Bay Aquarium | 5-6 | 7am-7pm Pacific only |
| NASA ISS | 1 | Always on |
| Cornell Lab Bird Cams | 3-5 | Seasonal |
| San Diego Zoo | 3-5 | If confirmed |
| **Total** | **~25-45** | At any given time |

## Risks

1. **YouTube video IDs change per broadcast.** Live stream video IDs are not permanent.
   When a stream ends and restarts, a new ID is assigned. The plugin must handle stale IDs
   gracefully. Mitigation: validate at refresh time, cache with short TTL, provide fallback.

2. **Seasonal availability.** Many explore.org cameras are seasonal (e.g., bear cams run
   June-October). The plugin should use the `is_offline` field from the API and only
   return active cameras.

3. **Monterey Bay Aquarium hours.** Cams only stream 7am-7pm Pacific. Outside those hours,
   YouTube shows a placeholder. This is acceptable -- the stream URL is still valid.

4. **Explore.org Wowza streams may not be publicly accessible.** The Wowza HLS URLs
   require knowing the exact stream name, which isn't in the public API. YouTube is the
   more reliable path for explore.org content.

5. **Rate limiting / scraping fragility.** If the plugin scrapes YouTube pages for video
   IDs, this could break if YouTube changes their page structure. The explore.org API is
   more stable but doesn't provide video IDs.

6. **No API keys required.** YouTube watch URLs work without authentication. The explore.org
   API is public and unauthenticated. NASA HLS is open.

## Implementation Notes

- Use `host_http_request` to call `https://explore.org/api/livecams` for the camera catalog
- Use `host_kv_set/get` to cache the camera list and YouTube video IDs (TTL: 1 hour)
- For thumbnails, the explore.org API provides `thumbnail_large_url` and `stillframe` URLs
- The `refresh()` function should filter to only `active: true` and `is_offline: false` cameras
- YouTube URLs format: `https://www.youtube.com/watch?v={VIDEO_ID}`
- NASA HLS URL is static and never changes
- The `describe()` function should set layout to grid view, grouped by category
