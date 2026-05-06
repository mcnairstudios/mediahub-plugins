# Trending Videos Plugin - Design Plan

## Feasibility: YES

This plugin is feasible. No API key is required. YouTube watch URLs work as stream
URLs (the trailers plugin already does this).

## API: Piped (not Invidious)

### Why not Invidious?

Invidious is effectively dead for API use. Testing on 2026-05-06:

- `vid.puffyan.us` - 502
- `invidious.nerdvpn.de` - 401
- `yewtu.be` - 403
- `inv.nadeko.net` - 403
- `iv.datura.network` - ECONNREFUSED
- `inv.thepixora.com` - 403 (was the *only* instance listed as api-enabled)
- The official instance list (`api.invidious.io/instances.json`) shows only 10
  instances total, with just 1 claiming API support (which also 403s).

**Invidious is not viable.** Most instances have disabled their API or gone offline.

### Why Piped?

Piped is the successor proxy project and its API actually works:

- **Trending:** `GET /trending?region={CC}` -- returns JSON array of videos
- **Search:** `GET /search?q={query}&filter=videos` -- returns paginated results
- Tested working instance: `api.piped.private.coffee` (96.84% uptime/24h, 98.72%/7d)
- Instance list at: `piped-instances.kavin.rocks`

### Piped response fields (per video)

```
url              "/watch?v=XXXXX"  (relative -- prepend https://www.youtube.com)
title            string
thumbnail        URL (proxied through Piped)
uploaderName     string
uploaderAvatar   URL
uploaderVerified bool
views            number
duration         seconds (int)
uploadedDate     "2 hours ago" (human-readable)
uploaded         unix timestamp (ms)
shortDescription string
isShort          bool
```

No direct playable stream URLs are returned -- only watch-page URLs and metadata.
This is fine: the trailers plugin already uses `https://www.youtube.com/watch?v=` as
the stream URL, so the MediaHub runtime clearly handles YouTube watch URLs.

## Architecture

### Language

Rust, compiled to WASM (`cdylib`), following the `space` and `radiogarden` plugins.

### Instance Failover

This is the key reliability concern. Strategy:

1. **Hardcoded instance list** (3-5 known-good Piped API instances), ordered by
   historical reliability.
2. **On refresh**, try each instance in order until one responds with HTTP 200.
3. **Cache the last working instance** in KV store (`host_kv_set`/`host_kv_get`) so
   subsequent refreshes try it first.
4. If all instances fail, return an empty stream list with an error log.

This is simple, robust, and requires no external instance-list fetching at runtime.

### Config Fields

| Key      | Label           | Type   | Required | Default |
|----------|-----------------|--------|----------|---------|
| `region` | Trending Region | select | false    | `US`    |

No API key needed. Region is the only config. Provide common options:
US, GB, CA, AU, DE, FR, JP, BR, IN, etc.

### Groups / Categories

Piped trending supports `type` parameter for categories:

- `music` -- `/trending?region=US&type=music`
- `gaming` -- `/trending?region=US&type=gaming`
- `movies` -- `/trending?region=US&type=movies`
- (default) -- general trending

Each category becomes a group in the stream list:
- "Trending"
- "Trending Music"
- "Trending Gaming"
- "Trending Movies"

### Stream Mapping

```
stream.id          = videoId (extracted from url field)
stream.name        = title
stream.url         = "https://www.youtube.com/watch?v=" + videoId
stream.group        = category group name
stream.logo        = thumbnail URL (use Piped proxy URL directly)
stream.vod_type    = "movie"
stream.tags        = [uploaderName]
stream.episode_name = shortDescription (truncated)
```

Filter out `isShort: true` entries -- shorts are not useful as streams.

### Plugin Descriptor

```
type:        "trending"
label:       "Trending Videos"
short_label: "TRENDING"
color:       "#ff0000"
version:     "1.0.0"
layout:      "grouped_list"
group_by:    "group"
searchable:  true
```

### File Structure

```
plugins/trending/
  Cargo.toml
  src/lib.rs
  README.md
```

## Implementation Outline

1. `describe()` -- return plugin metadata with region config field.
2. `refresh(config)` -- for each Piped category (general, music, gaming, movies):
   - Call `/trending?region={region}&type={type}` on the working Piped instance.
   - Parse JSON array, filter out shorts, map to stream structs.
   - On instance failure, try next instance in list.
   - Cache working instance in KV.
3. `interact()` -- no-op (return `{}`).

Target: ~40-60 streams across 4 groups.

## Risks and Mitigations

| Risk | Severity | Mitigation |
|------|----------|------------|
| Piped instances go down | Medium | Hardcoded failover list of 3-5 instances; KV-cached last-good |
| Piped project dies (like Invidious API) | High | Plugin is small/simple; can swap to whatever proxy emerges next |
| YouTube blocks Piped instances | Medium | Same as above; community spins up new instances regularly |
| Thumbnail proxy URLs break | Low | Thumbnails are nice-to-have; fall back to `i.ytimg.com` direct URLs |
| Rate limiting | Low | Trending only needs 4 requests per refresh; very light usage |

## Summary

- **API:** Piped (Invidious is dead for API use)
- **Auth:** None required
- **Stream URLs:** YouTube watch URLs (same as trailers plugin)
- **Reliability:** Instance failover with KV-cached last-good instance
- **Effort:** Small -- ~200-300 lines of Rust, follows existing plugin patterns
