# Radio Garden Plugin

Live radio stations from 12,000+ cities worldwide via the Radio Garden API.

**Language:** Rust | **Binary:** 183KB | **Streams:** 1000+

## Data Source

[Radio Garden](https://radio.garden) — free, no API key required.

## Features

- **Multi-place:** Configure multiple cities in a single source
- **Place search:** Interactive location search via the `interact` function
- **KV caching:** Places list (~13,000 entries) cached to avoid re-fetching on every search
- **Deduplication:** Same station in nearby cities is only included once
- **Audio-only:** Streams are MP3 (Icecast/Shoutcast), MediaHub handles audio-only playback

## Stream Format

- **Name:** Station title
- **URL:** `https://radio.garden/api/ara/content/listen/{channelID}/channel.mp3`
- **Group:** City name (from configured places)

## Config

| Field | Type | Description |
|-------|------|-------------|
| `places` | Custom (place-picker) | JSON array of `[{"id":"xxx","name":"London"}, ...]` |

## Build

```bash
cargo build --release --target wasm32-wasip1
# Output: target/wasm32-wasip1/release/radiogarden.wasm
```
