# Radio Browser Plugin

## Overview

- **Plugin name:** radiobrowser
- **Label:** Radio Browser
- **Description:** Browse 90,000+ internet radio stations worldwide from the community-driven Radio Browser directory
- **Language:** Rust — better suited for parsing large JSON responses (90k+ stations), stronger typing for the complex search API
- **Color:** `#ff9800` (orange — community/open-source feel)

## API

- **Base URL:** `https://de1.api.radio-browser.info`
- **Auth:** None required — fully public API, no keys
- **Rate limits:** Courtesy limit; the API recommends using a descriptive User-Agent header
- **Multiple servers:** `de1`, `nl1`, `at1` — use `de1` as primary

### Endpoints used

| Endpoint                                    | Purpose                        |
|--------------------------------------------|--------------------------------|
| `/json/stations/bytag/{tag}?limit=100`     | Browse by genre/tag            |
| `/json/stations/bycountry/{country}?limit=100` | Browse by country          |
| `/json/stations/search?name={q}&limit=50`  | Search by name                 |
| `/json/tags?order=stationcount&reverse=true&limit=50` | List popular tags    |
| `/json/countries?order=stationcount&reverse=true&limit=50` | List popular countries |

### Key fields per station

| Field           | Example                                  |
|----------------|------------------------------------------|
| `stationuuid`  | `db93a00f-9191-46ab-9e87-ec9b373b3eee`  |
| `name`         | `Arrow Classic Rock`                      |
| `url_resolved` | `http://stream.gal.io/arrow`             |
| `homepage`     | `https://www.arrow.nl/`                   |
| `favicon`      | `https://www.arrow.nl/.../logo.png`       |
| `tags`         | `"80s,rock,classic rock"`                 |
| `country`      | `The Netherlands`                         |
| `codec`        | `MP3`                                     |
| `bitrate`      | `192`                                     |

### Stream URL strategy

The `url_resolved` field contains the actual direct stream URL (MP3, AAC, HLS, etc.). These are ready to use — no playlist parsing needed.

## Grouping

Streams grouped by the **browsing mode** selected in config:
- **By tag/genre:** Group = the selected tag (e.g., "jazz", "rock", "classical")
- **By country:** Group = country name
- **By search:** Group = "Search Results"

When multiple tags or countries are selected, each becomes its own group.

## Config fields

| Key       | Label    | Type     | Required | Default |
|-----------|----------|----------|----------|---------|
| `mode`    | Browse by | select  | yes      | `tag`   |
| `tags`    | Genres   | text     | no       | `jazz,rock,classical` |
| `countries` | Countries | text   | no       | (empty) |

- `mode` select options: `tag`, `country`
- `tags` and `countries` are comma-separated lists
- Only the field matching the current `mode` is used

## Estimated stream count

Per refresh: 100-500 streams (limited per tag/country query to keep responses fast).
Total available: 90,000+ stations in the database.

## View config

- **Layout:** `grouped_list`
- **Group by:** `group` (tag or country name)
- **Searchable:** true
- **Sortable:** true (by bitrate, name, or listener votes)

## Plugin functions

### `describe()`
Returns metadata with config fields for mode, tags, and countries. Declares a `search_stations` interaction.

### `refresh(config)`
1. Read `mode` from config
2. If mode is `tag`: for each tag in the comma-separated `tags` field:
   - HTTP GET `/json/stations/bytag/{tag}?limit=100&order=votes&reverse=true`
   - Parse stations, set `group` = tag name
3. If mode is `country`: for each country in `countries` field:
   - HTTP GET `/json/stations/bycountry/{country}?limit=100&order=votes&reverse=true`
   - Parse stations, set `group` = country name
4. Deduplicate by `stationuuid`
5. For each station, emit a stream:
   - `id` = `stationuuid`
   - `name` = station `name`
   - `url` = `url_resolved` (fallback to `url`)
   - `group` = tag or country name
   - `logo` = `favicon`
   - `tags` = split comma-separated `tags` field

### `interact(action)`
- **`search_stations`**: Takes a `query` param, calls `/json/stations/search?name={query}&limit=50`, returns results for the UI to display.

## Risks and limitations

- **Dead streams:** Radio Browser is community-maintained. Some stations may be offline or have broken URLs. The API provides a `lastcheckok` field that can be used to filter (add `?lastcheckok=1` to queries).
- **Large responses:** Limiting to 100 stations per tag/country keeps response sizes manageable.
- **Server availability:** The `de1` server could go down. Mitigation: could rotate through `de1`, `nl1`, `at1` on failure, but this adds complexity — start with `de1` only.
- **Station quality varies:** Unlike curated services, quality and metadata completeness varies widely. Some stations lack favicons, proper tags, or have low bitrates.
- **URL scheme:** Some `url_resolved` values use HTTP (not HTTPS). This is expected for internet radio streams and should work fine for audio playback.
