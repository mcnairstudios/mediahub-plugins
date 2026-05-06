# Space Launches Plugin

Space launch webcasts from all agencies worldwide via the Launch Library 2 API.

**Language:** Rust | **Binary:** 90KB | **Streams:** ~700

## Data Source

[Launch Library 2](https://ll.thespacedevs.com) by The Space Devs — free, no API key required.

- Past launches (detailed mode): up to 500, paginated
- Upcoming launches (list mode): up to 250, paginated
- Grouped by launch provider (SpaceX, Rocket Lab, ULA, Blue Origin, Arianespace, ISRO, etc.)

## Stream Format

- **Name:** Launch name with date, e.g. "Starlink Group 12-7 (Jan 14, 2025)"
- **URL:** YouTube webcast link (resolved by MediaHub's YouTube resolver at playback)
- **Group:** Launch provider name
- **Logo:** Mission patch / launch image
- **Tags:** Status (success, failure, go, tbc)

## Config

No configuration needed.

## Build

```bash
cargo build --release --target wasm32-wasip1
# Output: target/wasm32-wasip1/release/space_rust.wasm
```

## Rate Limiting

The LL2 free tier allows 15 requests per hour. The plugin fetches up to 15 pages per refresh. Avoid triggering manual refreshes too frequently.
