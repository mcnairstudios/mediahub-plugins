# SomaFM Plugin

## Overview

- **Plugin name:** somafm
- **Label:** SomaFM
- **Description:** Curated internet radio channels from SomaFM — ambient, electronic, indie, and more
- **Language:** Go (TinyGo) — matches the demo plugin pattern, simple JSON parsing, no complex data wrangling needed
- **Color:** `#2196f3` (SomaFM blue)

## API

- **Endpoint:** `https://api.somafm.com/channels.json`
- **Auth:** None required — fully public API
- **Rate limits:** None documented; single request returns all channels
- **Response:** JSON object with a `channels` array

### Key fields per channel

| Field        | Example                                              |
|-------------|------------------------------------------------------|
| `id`        | `groovesalad`                                        |
| `title`     | `Groove Salad`                                       |
| `description` | `A nicely chilled plate of ambient/downtempo...`   |
| `genre`     | `ambient\|electronic`                                |
| `listeners` | `2254`                                               |
| `image`     | `https://api.somafm.com/img/groovesalad120.png`     |
| `largeimage`| `https://api.somafm.com/logos/256/groovesalad256.png`|
| `playlists` | Array of `{url, format, quality}` entries            |

### Stream URL strategy

The API returns `.pls` playlist URLs, not direct streams. However, the direct stream URLs follow a predictable pattern:

```
https://ice2.somafm.com/{channel_id}-256-mp3
```

The plugin will construct direct MP3 stream URLs from the channel `id` field, avoiding the need to fetch and parse `.pls` files. Fallback: use the first `.pls` URL if direct construction fails (many players handle `.pls` natively).

## Grouping

Streams grouped by **genre** (the `genre` field, pipe-delimited — use the first genre token as the group name). Typical groups: ambient, electronic, indie, rock, jazz, lounge, etc.

## Config fields

None required. SomaFM has around 30-40 curated channels — small enough to show all at once.

## Estimated stream count

~35 channels (SomaFM is a curated service, not a directory).

## View config

- **Layout:** `grouped_list`
- **Group by:** `group` (genre)
- **Searchable:** true
- **Sortable:** true (by listener count)

## Plugin functions

### `describe()`
Returns metadata with no config fields, grouped_list layout.

### `refresh(config)`
1. HTTP GET `https://api.somafm.com/channels.json`
2. Parse the `channels` array
3. For each channel, emit a stream:
   - `id` = channel `id`
   - `name` = channel `title`
   - `url` = `https://ice2.somafm.com/{id}-256-mp3`
   - `group` = first genre from pipe-delimited `genre` field
   - `logo` = `largeimage` field (256px logos)
   - `tags` = split `genre` by `|`

### `interact(action)`
No interactions needed — the channel list is small and static enough that client-side search suffices.

## Risks and limitations

- **Stream URL pattern:** The `ice2.somafm.com/{id}-256-mp3` pattern is observed but not formally documented. If it changes, the plugin would need to fetch and parse `.pls` files instead. Mitigation: validate against a known channel on first build.
- **Small catalog:** Only ~35 channels. This is a feature (curated quality), not a bug.
- **No search API:** Not needed given the small catalog size.
