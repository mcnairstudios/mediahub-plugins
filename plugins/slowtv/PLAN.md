# Slow TV Plugin

## Description

Curated collection of long-form ambient videos: Norwegian train journeys, boat rides, nature walks, fireplaces, and more. All content is free on YouTube, played via YouTube watch URLs (same approach as the trailers plugin).

## Language

Go (TinyGo) -- consistent with other plugins.

## Source of Video URLs

A curated, hardcoded list of YouTube video IDs, organized by category. Unlike the aerials plugin, there is no single authoritative JSON feed -- Slow TV is a genre spread across multiple YouTube channels.

### Primary sources (YouTube channels):

- **NRK official uploads** -- The original Slow TV broadcaster. Key videos:
  - Nordlandsbanen (Trondheim to Bodo) in winter, spring, summer (~10 hours each)
  - Bergen Line train journey (~7 hours)
  - Hurtigruten coastal voyage
- **SlowTV Relax&Background** (`youtube.com/c/SlowTVRelaxBackground`) -- Fireplaces, beaches, landscapes, train journeys, boat trips, rain, waterfalls
- **SlowTV** (`youtube.com/channel/UC3ewP9SczRIGGpDXDQvpPyQ`) -- Trains, ships, nature, city walks

### Known video IDs:

| Video ID | Title | Duration |
|----------|-------|----------|
| 3rDjPLvOShM | Train Journey to Norwegian Arctic Circle (Winter) | ~10h |
| cNiN7gOcNI4 | Train Journey to Norwegian Arctic Circle (Spring) | ~10h |
| yCtt26c_AOg | Train Journey to Norwegian Arctic Circle (Summer) | ~10h |
| hXMtC8Kj-sQ | Norwegian Slow TV compilation | varies |

URLs take the form `https://www.youtube.com/watch?v={VIDEO_ID}`, which the host app already supports (as demonstrated by the trailers plugin).

## How Videos Are Grouped

- **Train Journeys** -- Norwegian rail routes, cab-view rides
- **Boat & Ship** -- Hurtigruten coastal voyages, fjord crossings, canal boats
- **Nature** -- Forests, mountains, northern lights, seasons changing
- **Fireplace & Cabin** -- Crackling fire, cozy cabin scenes
- **City Walks** -- Slow walking tours through cities

## Config Fields

None. The list is curated and hardcoded.

## Estimated Stream Count

Initial release: 20-40 videos. Can grow over time as more are curated. Each video is typically 1-10 hours long, so even a modest count provides hundreds of hours of content.

## Refresh Strategy

1. Return the hardcoded list of streams on each `refresh()` call.
2. Optionally, a future version could fetch a curated JSON list from a GitHub repo to allow updates without rebuilding the WASM binary.

## Risks

- **YouTube video availability.** Individual videos can be taken down or made private. The plugin should include enough videos that a few removals do not break the experience. Periodic manual curation is needed.
- **No single authoritative feed.** Unlike aerials (which has a machine-readable catalog), Slow TV content must be manually curated. This means the video list will be static and embedded in the plugin.
- **YouTube region restrictions.** Some NRK content may be geo-restricted. The major train journey videos appear to be globally available based on current evidence.
- **Content licensing ambiguity.** While these are publicly uploaded YouTube videos, some channels may remove content. Sticking to official NRK uploads and large established channels reduces this risk.

## API Keys

None required. All content is free YouTube videos accessed via watch URLs.

## Relationship to Aerials Plugin

These plugins are complementary but distinct:
- **Aerials** = short (2-5 minute) polished 4K clips from Apple, direct .mov files
- **Slow TV** = long-form (1-10 hour) ambient YouTube videos

They could theoretically be merged, but the different content sources, video lengths, and URL types (direct file vs. YouTube) argue for keeping them separate.
