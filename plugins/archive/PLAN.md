# Internet Archive Plugin Plan

## Plugin Metadata

- **Name:** Internet Archive
- **Type:** `archive`
- **Label:** Internet Archive
- **Short Label:** IA
- **Color:** #428bca
- **Language:** Go (TinyGo, consistent with other plugins in this repo)
- **Description:** Public domain movies, classic TV, old-time radio, and live concert recordings from the Internet Archive

## API Overview

The Internet Archive provides a fully public, unauthenticated JSON API. No API key is required.

### Endpoints Used

| Endpoint | Purpose |
|----------|---------|
| `https://archive.org/advancedsearch.php?q=...&fl=...&rows=N&output=json` | Search/browse items by collection, mediatype, and keywords |
| `https://archive.org/metadata/{identifier}` | Get full metadata and file list for a single item |
| `https://archive.org/download/{identifier}/{filename}` | Direct file access (playable MP4/MP3 URLs) |

### Search API Details

- Query syntax: `mediatype:movies AND collection:feature_films`
- Field selection via `fl=` param (identifier, title, description, year, collection)
- Pagination via `rows=` and `page=` params
- Returns JSON with `response.numFound` and `response.docs[]`

### Metadata API Details

- Returns JSON with `metadata` (title, description, year, collection, mediatype) and `files[]`
- Each file entry has: `name`, `format`, `size`, `source` (original/derivative)
- Video derivatives are typically format `h.264` with `.mp4` extension
- Audio derivatives are typically format `VBR MP3` with `.mp3` extension

## Collections to Feature

| Collection ID | Description | Approx. Items | Media Type |
|---------------|-------------|---------------|------------|
| `feature_films` | Public domain feature films | ~27,000 | movies |
| `prelinger` | Prelinger Archives (educational, industrial, ephemeral films) | ~10,000 | movies |
| `oldtimeradio` | Old-time radio shows (drama, comedy, mystery) | ~8,700 | audio |
| `GratefulDead` | Grateful Dead live concert recordings | ~18,000 | etree |
| `classic_tv` | Classic television shows | TBD | movies |
| `silent_films` | Silent era films | TBD | movies |
| `film_noir` | Film noir classics | TBD | movies |
| `scifi` | Public domain sci-fi films | TBD | movies |

Initial implementation should focus on: `feature_films`, `prelinger`, `oldtimeradio`, and `GratefulDead`.

## Stream Mapping

### How items map to streams

1. **refresh()** calls the advanced search API for each enabled collection, fetching N items per collection (configurable, default 50)
2. For each item returned, call the metadata API to find the best playable file
3. Construct stream with:
   - `id`: item identifier (e.g., `la-guitarra-de-gardel-1949`)
   - `name`: item title
   - `url`: `https://archive.org/download/{identifier}/{best_file_name}` -- direct MP4/MP3 URL
   - `group`: collection display name (e.g., "Feature Films", "Old Time Radio")
   - `logo`: `https://archive.org/services/img/{identifier}` (thumbnail service)
   - `vod_type`: `movie` for films, omitted for audio
   - `year`: from metadata when available
   - `tags`: derived from collection and mediatype

### Best file selection logic

For video items, prefer in order:
1. h.264 / `.mp4` derivative (most compatible)
2. MPEG4 derivative
3. Ogg Video / `.ogv`
4. Original file as fallback

For audio items, prefer in order:
1. VBR MP3 / `.mp3`
2. Ogg Vorbis / `.ogg`
3. Original FLAC (less compatible but available)

## Config Fields

| Key | Label | Type | Default | Description |
|-----|-------|------|---------|-------------|
| `collections` | Collections | `select` (multi) | `feature_films,prelinger,oldtimeradio` | Which collections to include |
| `items_per_collection` | Items per collection | `number` | `50` | How many items to fetch per collection |
| `sort` | Sort by | `select` | `downloads desc` | Sort order (downloads, date, titleSorter) |

## View Config

```json
{
  "layout": "grouped_list",
  "group_by": "group",
  "searchable": true,
  "sortable": true
}
```

## Interactions

- **search**: Use the advanced search API with user query appended to collection filters. Returns matching items across enabled collections.

## Estimated Stream Count

With default settings (50 items x 3 collections): ~150 streams per refresh.

Maximum practical: 200 items x 4 collections = 800 streams (limited by metadata API calls during refresh).

## Performance Considerations

- **Two-phase fetch**: Search API returns item identifiers, but a second metadata call per item is needed to find the best file URL. This is the main bottleneck.
- **Mitigation 1**: Use `host_kv_set/get` to cache identifier-to-URL mappings, avoiding repeated metadata lookups.
- **Mitigation 2**: For some collections, the file naming is predictable (e.g., `{identifier}.mp4`), so metadata calls can be skipped with a heuristic and fallback.
- **Mitigation 3**: Limit items_per_collection to keep refresh times reasonable.

## Risks

| Risk | Severity | Mitigation |
|------|----------|------------|
| File format variety -- some items lack MP4 derivatives | Medium | Best-file selection logic with fallback chain; skip items with no playable format |
| Two API calls per item slows refresh | Medium | KV caching, predictable URL heuristics, reasonable default limits |
| Some items may be dark/unavailable | Low | Check `is_dark` flag in metadata, skip those items |
| Large file sizes (500MB+ movies) | Low | This is a player concern, not a plugin concern; streams are URLs |
| Rate limiting on metadata API | Low | Archive.org is generous; cache aggressively to reduce calls |
| Some audio items (etree) have many individual track files | Medium | For concerts, either pick first track or list each track as a separate stream |

## API Key Requirement

**None.** The Internet Archive API is fully public and requires no authentication. Confirmed by successful unauthenticated fetches of search results and item metadata.
