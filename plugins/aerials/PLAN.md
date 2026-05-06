# Aerials Plugin

## Description

Streams Apple's stunning 4K aerial screensaver videos -- cityscapes, landscapes, underwater scenes, and Earth from the ISS. No API key required; videos are served directly from Apple's CDN.

## Language

Go (TinyGo) -- consistent with the majority of existing plugins.

## Source of Video URLs

The plugin fetches the community-maintained video catalog from:

    https://raw.githubusercontent.com/OrangeJedi/Aerial/master/videos.json

This JSON file contains ~220 videos with URLs on `sylvan.apple.com`. Each entry includes:
- `id` (UUID)
- `name` and `accessibilityLabel`
- `type` (space, landscape, underwater, cityscape)
- `timeOfDay` (day, night, or unset)
- `pointsOfInterest` (timestamped location descriptions)
- `src` with three quality tiers:
  - `H2641080p` -- H.264 1080p (.mov) under `/Videos/`
  - `H2651080p` -- HEVC 1080p (.mov) under `/Aerials/2x/Videos/`
  - `H2654k` -- HEVC 4K (.mov) under `/Aerials/2x/Videos/`

Verified: both H.264 and HEVC URLs return HTTP 200 from `sylvan.apple.com` with `Content-Type: video/quicktime` and `Access-Control-Allow-Origin: *`. Files range from ~160-250 MB.

## How Videos Are Grouped

Primary grouping by `type`:
- **Space** -- Earth from the ISS (day and night passes)
- **Landscape** -- Grand Canyon, Yosemite, Iceland, Patagonia, etc.
- **Underwater** -- Ocean and reef scenes
- **Cityscape** -- Dubai, Hong Kong, New York, San Francisco, London, etc.

Secondary grouping (or tag) by `timeOfDay`: day / night.

## Config Fields

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| quality | select | no | H2641080p | Video quality tier (H.264 1080p, HEVC 1080p, HEVC 4K) |

H.264 1080p is the safest default for broad device compatibility. The 4K HEVC option can be offered but requires HEVC-capable playback.

No API key or authentication needed.

## Estimated Stream Count

~220 videos across 4 categories. The OrangeJedi catalog is periodically updated as Apple adds new content.

## Refresh Strategy

1. Fetch `videos.json` from GitHub (or cache via `host_kv_get/set` with a 24-hour TTL).
2. Parse the JSON array.
3. For each entry, emit a stream using the URL from the configured quality tier.
4. Group by `type`, tag with `timeOfDay`.

## Risks

- **Apple could change or remove CDN URLs.** This has not happened since 2015 when the community started tracking them, but it remains possible. The URLs have CORS headers and year-long cache-control, suggesting Apple tolerates external access.
- **Large file sizes.** Individual videos are 150-500 MB. This is fine for streaming (the CDN supports range requests) but users on metered connections should be aware.
- **HEVC compatibility.** Not all players support HEVC. Defaulting to H.264 mitigates this.
- **OrangeJedi repo could go offline.** Fallback: the JSON could be embedded in the plugin as a static list, or fetched from the JohnCoates/Aerial project instead.

## API Keys

None required. All content is served from Apple's public CDN without authentication.
