# Outdoor Cams Plugin - PLAN

## Plugin Metadata

- **Name:** outdoorcams
- **Display Name:** Outdoor Cams
- **Description:** Live 24/7 video streams of volcanoes, surf/beach cams, and ski resort webcams from YouTube
- **Language:** Go (TinyGo) -- consistent with most plugins in the repo
- **Version:** 0.1.0

## Feasibility Summary

**FEASIBLE.** All three categories (volcano, surf/beach, ski) have confirmed 24/7 YouTube live video streams. YouTube watch URLs are already used by the trailers plugin, so the host player can handle them. No API keys are required -- all streams are free public YouTube live streams.

## Confirmed Video Stream Sources

### Volcano Cams (CONFIRMED VIDEO -- strongest category)

**USGS Official (YouTube @usgs channel):**
1. Kilauea V1cam -- west Halema'uma'u crater (youtube.com/usgs/live)
2. Kilauea V3cam -- south Halema'uma'u crater
3. USGS maintains multiple live volcano cams on their YouTube channel

**AfarTV (YouTube @afartv channel):**
4. Mount Etna, Sicily -- HD
5. Mayon Volcano, Philippines -- 4K
6. Popocatepetl, Mexico -- 4K
7. Iceland Volcanoes & Auroras -- 4K
8. Semeru Volcano, Java, Indonesia -- 4K
9. Merapi Volcano, Indonesia -- 4K
10. Santa Maria Volcano, Guatemala -- 4K
11. Fuego Volcano, Guatemala -- 4K

**Volcoholics / Wild Horizons (YouTube @volcanolivestreaming):**
12. Multi-cam volcano monitoring stream (8+ cameras)
13. Iceland volcano eruption streams
14. Kilauea multi-cam stream
15. Global volcano monitoring multi-cam

**Other confirmed YouTube volcano streams:**
16. 24/7 Kilauea Volcano Watch multi-cam (youtube.com/watch?v=FVdmnpJ2kM0)
17. Kilauea Volcano Livestream Cam A (youtube.com/watch?v=iws3rh5vLAQ)

### Surf / Beach Cams (CONFIRMED VIDEO)

**Explore.org (YouTube channel: Explore Oceans / Explore Live Nature Cams):**
1. Pipeline Cam, North Shore, Oahu (youtube.com/watch?v=DY5RYp4sxYc)
2. Waimea Bay Cam, Oahu (youtube.com/watch?v=wnNrd-VjLsQ)
3. Hawaii Humpback Whale Marine Sanctuary, Maui (youtube.com/watch?v=6i0yI_pfg7k)

**AfarTV:**
4. Waimea Bay, Oahu -- 4K surf cam

**Independent YouTube streams (confirmed from aggregator lists):**
5. Soggy Dollar Bar, British Virgin Islands (youtube.com/watch?v=LXWVYoBluT4)
6. Santa Monica Beach Cam (youtube.com/watch?v=OWbI6WtlI-k)
7. Pacifica Pier and Beach Live 4K, California (youtube.com/watch?v=zYi_5AF6B2A)
8. 30A Beach Cam, Santa Rosa Beach, Florida (youtube.com/watch?v=ftGfQqCA184)
9. Hamptons Main Beach Cam, East Hampton, NY (youtube.com/watch?v=Ba2cLC3xUpU)
10. Glass Beach at Fort Bragg, California (youtube.com/watch?v=rxBBRLWF0mM)
11. Ambergris Caye, Belize (youtube.com/watch?v=r_XFhbOQ-Jo)
12. Calichi at Picture Point, St. John USVI (youtube.com/watch?v=m7c12NY6xok)

### Ski Resort Cams (CONFIRMED VIDEO -- seasonal)

**YouTube live streams found:**
1. Ski Panorama -- 24/7, 200 webcams across 8 countries (youtube.com/watch?v=HVt5n0CDRF8)
2. Grouse Mountain 4K, Vancouver (youtube.com/watch?v=-XM7S9nm9js)
3. Ski Resort Webcams -- New England multi-cam (youtube.com/watch?v=oRyJBAIOto0)
4. Palisades Tahoe Live (youtube.com/watch?v=8xEgLRLR7u0)
5. Mount Washington Alpine Resort (youtube.com/watch?v=k7loUrt8HkM)
6. Ski Sundown Live Webcam (youtube.com/watch?v=2zVEuh_7rKk)
7. Transalpina Ski Resort, Romania (youtube.com/watch?v=1t9RkU0khvo)
8. Pomerelle Mountain Ski Resort (youtube.com/watch?v=uM-oftYVFGA)

## Sources NOT Used (and why)

| Source | Format | Why excluded |
|---|---|---|
| VolcanoDiscovery | Static images (12s-15min refresh) | Not video |
| Surfline | Video, but paywalled | Requires subscription for most cams |
| SkylineWebcams | HLS video (m3u8) | Embed restricted to webcam hosts; stream URLs are authenticated/dynamic; scraping violates ToS |
| OnTheSnow / SnowEye | Mostly static images | Embedded resort-specific players, not standardized video |
| HDOnTap | Video streams | Proprietary player, no public YouTube/HLS URLs |
| WebcamTaxi | Video streams | Proprietary player, unclear licensing for URL extraction |

## Architecture

### Stream List: Hardcoded with Periodic Review

YouTube video IDs for 24/7 live streams are relatively stable (they reuse the same video ID for ongoing live streams). The plugin will ship with a hardcoded list of stream entries.

Each entry:
```json
{
  "id": "volcano-etna-afartv",
  "name": "Mount Etna, Italy (4K)",
  "url": "https://www.youtube.com/watch?v=VIDEO_ID",
  "group": "Volcanoes",
  "tags": ["volcano", "italy", "4k", "24/7"],
  "logo": ""
}
```

### Why NOT auto-discover via YouTube API

- YouTube Data API v3 requires an API key (violates no-API-key requirement)
- YouTube channel RSS feeds exist but only list recent uploads, not current live streams
- YouTube "live" page scraping is fragile and against ToS

### Maintenance Strategy

- Hardcode ~30-40 streams at launch
- Group channels by reliability tier (USGS and AfarTV are most stable)
- Periodic manual updates when streams go offline or new ones appear
- Could add a community-maintained JSON file hosted on GitHub that the plugin fetches (using host_http_request) to allow updates without rebuilding WASM

### Optional: Remote Stream List

To avoid rebuilding WASM for stream changes, the plugin could fetch a JSON file from a GitHub raw URL on refresh (with KV cache). This keeps the WASM binary stable while allowing the stream list to evolve. Fallback to built-in list if fetch fails.

## Grouping Strategy

Primary groups (shown as tabs/sections in the UI):
- **Volcanoes** (~15-17 streams)
- **Beach & Surf** (~12 streams)
- **Ski Resorts** (~8 streams, seasonal)

Optional sub-grouping by region via tags: Hawaii, Europe, North America, Asia, Caribbean.

## Config Fields

**None required.** All streams are free public YouTube live streams.

Optional future config:
- `show_offline` (bool) -- whether to show streams that may be offline
- `regions` (multi-select) -- filter by region

## Estimated Stream Count

| Category | Confirmed | Potential |
|---|---|---|
| Volcanoes | 17 | 20+ |
| Beach & Surf | 12 | 20+ |
| Ski Resorts | 8 | 12+ |
| **Total** | **37** | **52+** |

## Risks

1. **YouTube video IDs can change.** Long-running live streams sometimes get new video IDs when restarted. Mitigation: use channel-level links where possible, maintain a remote JSON list.

2. **Streams go offline.** Volcano cams go dark when eruptions stop; ski cams may shut down in summer. Mitigation: tag streams as seasonal, document expected availability.

3. **YouTube rate limiting.** Opening many YouTube URLs in sequence could trigger rate limits in the player. Mitigation: this is a host/player concern, not a plugin concern.

4. **Seasonal content.** Ski cams are most interesting Oct-Apr (Northern Hemisphere). Beach cams are year-round. Volcano cams depend on eruption activity. Mitigation: clearly label seasonal streams.

5. **Copyright/ToS.** We are only linking to public YouTube live streams (not embedding or scraping). This is equivalent to bookmarking and should be fine.

## Implementation Notes

- Use Go (TinyGo) to match the majority of plugins in this repo
- Follow the pattern from `plugins/demo/main.go`
- The `describe()` function returns plugin metadata with group "Outdoor Cams"
- The `refresh()` function returns the hardcoded (or fetched) stream list
- The `interact()` function can be minimal (no special interactions needed)
- Each stream URL is a standard `https://www.youtube.com/watch?v=VIDEO_ID` URL
- The trailers plugin already proves YouTube URLs work in the host player
