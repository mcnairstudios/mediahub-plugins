# Old Time Radio Plugin

## Plugin Metadata

- **Name:** `oldtimeradio`
- **Label:** Old Time Radio
- **Short Label:** OTR
- **Description:** Classic radio shows from the 1930s-1950s golden age of radio, sourced from Internet Archive
- **Language:** Go (TinyGo, consistent with other plugins)
- **Type:** `audio`

## Data Source

Internet Archive's Old Time Radio collection, accessed via two public endpoints (no API key required):

### Discovery Endpoint

```
https://archive.org/advancedsearch.php?q=collection:oldtimeradio+AND+format:"VBR MP3"&fl=identifier,title,description,creator&rows=200&output=json&sort=downloads+desc
```

- Returns JSON with `identifier`, `title`, `description`, `creator`
- ~7,614 items have MP3 files available (of ~8,855 total)
- Sorting by `downloads desc` surfaces the most popular shows first

### Item Metadata Endpoint

```
https://archive.org/metadata/{identifier}
```

- Returns full metadata including the `files` array
- Filter files array for entries where `format` is `"VBR MP3"`
- Each MP3 file name maps to a playable URL

### Playable Audio URL Pattern

```
https://archive.org/download/{identifier}/{filename}
```

Example:
```
https://archive.org/download/OTRR_Gunsmoke_Singles/Gunsmoke%2052-04-26%20(001)%20Billy%20the%20Kid.mp3
```

## File Selection Strategy

1. Fetch item metadata via `/metadata/{identifier}`
2. Filter `files` array for `format == "VBR MP3"`
3. Each MP3 file becomes one stream entry
4. Use the MP3 filename (minus extension) as the episode name
5. Items with only `.ra` (RealAudio) files are skipped -- most popular items have MP3s

## Grouping Strategy

Group by **show name** (derived from the item title or `creator` field):
- "Gunsmoke" -- 36 episodes
- "Yours Truly, Johnny Dollar" -- multi-episode collection
- "Dragnet" -- multi-episode collection
- "Lights Out" -- individual episodes
- etc.

For single-episode items, group under "Miscellaneous" or by the `creator` field.

## Stream Object Shape

```json
{
  "id": "OTRR_Gunsmoke_Singles__001",
  "name": "Billy the Kid",
  "url": "https://archive.org/download/OTRR_Gunsmoke_Singles/Gunsmoke%2052-04-26%20(001)%20Billy%20the%20Kid.mp3",
  "group": "Gunsmoke",
  "logo": "https://archive.org/services/img/OTRR_Gunsmoke_Singles",
  "tags": ["drama", "western"],
  "episode_name": "Billy the Kid"
}
```

## Config Fields

| Key | Label | Type | Required | Default |
|-----|-------|------|----------|---------|
| `max_shows` | Max shows to load | `number` | No | `50` |

Minimal config. The `max_shows` field limits how many archive.org items are fetched (each item may contain multiple episodes). This controls both load time and the number of HTTP requests during refresh.

## Refresh Flow

1. Call discovery endpoint with `rows={max_shows}`, sorted by downloads
2. For each returned item, call `/metadata/{identifier}`
3. Extract MP3 files from the files array
4. Build stream entries with proper grouping
5. Cache results using `host_kv_set` (archive.org data is static)

## Estimated Stream Count

- With `max_shows=50`: approximately 500-2,000 individual episodes
- With `max_shows=200`: approximately 5,000+ episodes
- Total available: tens of thousands of MP3 episodes across 7,614 items

## API Key Requirement

**NO** -- Internet Archive's search and metadata APIs are fully public.

## Risks

1. **Rate limiting:** Fetching metadata for many items requires one HTTP request per item. Mitigate by caching aggressively and limiting `max_shows`.
2. **Format variety:** Some older items only have `.ra` (RealAudio) files with no MP3 derivative. These must be skipped. The query filter `format:"VBR MP3"` pre-filters at the discovery level.
3. **Large metadata responses:** Some collection items (e.g., `lumedwards` -- "Old Time Radio Researchers Collection") are massive aggregations with thousands of files. These may need special handling or exclusion.
4. **File naming inconsistency:** Episode titles must be parsed from filenames, which vary in format across uploaders. Some use `Show YY-MM-DD (NNN) Title.mp3`, others use different conventions.
5. **Thumbnail availability:** `https://archive.org/services/img/{identifier}` provides item thumbnails but not all items have good cover art.
