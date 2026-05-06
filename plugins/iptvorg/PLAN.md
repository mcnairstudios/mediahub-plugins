# iptv-org Plugin Plan

## Overview

- **Plugin name:** iptv-org
- **Label:** Live TV (iptv-org)
- **Short label:** IPTV
- **Description:** Free live TV channels from the iptv-org community directory
- **Language:** Go (TinyGo) -- consistent with the demo plugin and simpler for JSON parsing
- **Type:** live

## API Endpoints

All endpoints are static JSON files hosted on GitHub Pages. No API key required. No authentication. No rate limiting (standard GitHub Pages CDN).

| Endpoint | Purpose |
|---|---|
| `https://iptv-org.github.io/api/streams.json` | Stream URLs (primary data source) |
| `https://iptv-org.github.io/api/categories.json` | Category list for reference |
| `https://iptv-org.github.io/api/logos.json` | Channel logos keyed by channel ID |

### Data shapes

**streams.json** entry:
```json
{
  "channel": null,
  "feed": null,
  "title": "Bloomberg TV",
  "url": "https://example.com/stream.m3u8",
  "quality": "1080p",
  "label": null,
  "user_agent": null,
  "referrer": null
}
```

**logos.json** entry:
```json
{
  "channel": "Bloomberg.us",
  "feed": null,
  "in_use": true,
  "tags": [],
  "width": 512,
  "height": 512,
  "format": "PNG",
  "url": "https://i.imgur.com/example.png"
}
```

## Key Finding: channel/feed fields are null

In the current streams.json snapshot, **all entries have `channel: null` and `feed: null`**. This means we cannot join streams to channels.json to get country or category metadata. The only usable fields per stream are `title`, `url`, `quality`, and `label`.

This is still viable -- streams have descriptive titles and real HLS URLs -- but it limits our ability to group by country or category automatically.

## Grouping Strategy

Since we cannot join to channel metadata, grouping options are:

1. **By quality** -- group streams by their `quality` field (1080p, 720p, 480p, etc.). Simple but not very useful for browsing.
2. **By label** -- some streams have labels like "Geo-blocked" or "Not 24/7", but most are null.
3. **Flat list with search** -- present all streams as a single searchable list. Given ~1000+ streams, this is workable with the `searchable: true` view config.

**Recommended approach:** Flat searchable list. The `GroupBy` field will be set to `"group"` but all streams will have group set to `"Live TV"` (or by quality tier if the user prefers). Search is the primary navigation method.

## Config Fields

None required. The API is entirely public with no configuration needed.

Optional future config fields:
- **quality_filter** (select): Filter by minimum quality (e.g., 720p+, 1080p+)
- **hide_geoblocked** (boolean): Exclude streams labeled "Geo-blocked"

## Estimated Stream Count

Approximately **1,000-1,300 streams** based on the current streams.json. This is a manageable size for a single refresh call.

## Stream Mapping

Each iptv-org stream maps to the plugin Stream type as follows:

| Plugin field | Source |
|---|---|
| `id` | Hash or index-based ID (no natural unique ID in the data) |
| `name` | `title` field |
| `url` | `url` field (already HLS .m3u8 in nearly all cases) |
| `group` | Quality tier or static "Live TV" |
| `logo` | Empty (cannot join to logos.json without channel ID) |
| `tags` | Derived from `quality` and `label` fields |

## Risks and Limitations

1. **No channel linkage:** The `channel` field in streams.json is universally null, preventing joins to channels.json for country/category/logo data. This may change in future iptv-org releases; if it does, the plugin can be updated to support richer grouping.

2. **Stream reliability:** These are community-maintained links. Streams go offline frequently. The plugin cannot verify liveness at refresh time without testing each URL (impractical for 1000+ streams).

3. **Geo-blocking:** Some streams are geo-restricted (labeled "Geo-blocked"). They will appear in the list but may not play for all users.

4. **Referrer requirements:** A small number of streams require a specific `Referer` header. The current host `host_http_request` is used for fetching the JSON catalog, not for playback -- the media player handles playback. If the player does not send the required referrer, those streams will fail.

5. **NSFW content:** The channels.json has an `is_nsfw` flag and there is an "XXX" category, but since we cannot join streams to channels, we cannot reliably filter adult content. The streams.json data itself does not indicate NSFW status.

6. **Payload size:** streams.json is a single large JSON file (~1-2 MB). This is fine for a single HTTP fetch but should be cached using `host_kv_set` with a reasonable TTL (e.g., 1 hour).

7. **No logos:** Without channel linkage, we cannot associate logos with streams. The UI will show streams without thumbnails.

## Feasibility Assessment

**FEASIBLE.** The API is:
- Fully public, no API key needed
- Returns real playable HLS (.m3u8) video stream URLs
- Well-structured JSON
- Reasonable payload size

The main limitation (null channel field preventing metadata joins) reduces the richness of the plugin but does not block core functionality. The plugin will deliver a searchable catalog of ~1000+ live TV streams with working video URLs.
