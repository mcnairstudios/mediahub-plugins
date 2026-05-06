# Public Domain Movies Plugin

## Plugin Metadata

- **Name:** `publicdomain`
- **Label:** Public Domain Movies
- **Short Label:** PDM
- **Description:** Classic public domain feature films from the 1920s-1960s, sourced from Internet Archive
- **Language:** Go (TinyGo, consistent with other plugins)
- **Type:** `video`

## Data Source

Internet Archive's Feature Films collection, accessed via two public endpoints (no API key required):

### Discovery Endpoint

```
https://archive.org/advancedsearch.php?q=collection:feature_films+AND+mediatype:movies+AND+year:[1920+TO+1965]+AND+format:h.264&fl=identifier,title,description,year,creator&rows=200&output=json&sort=downloads+desc
```

- Returns JSON with `identifier`, `title`, `description`, `year`, `creator`
- ~7,265 items have h.264 (MP4) files available in the 1920-1965 range
- Sorting by `downloads desc` surfaces well-known classics first

### Item Metadata Endpoint

```
https://archive.org/metadata/{identifier}
```

- Returns full metadata including the `files` array
- Filter for entries where `format` is `"h.264"` or `"MPEG4"`
- Each MP4 file maps to a playable video URL

### Playable Video URL Pattern

```
https://archive.org/download/{identifier}/{filename}
```

Example:
```
https://archive.org/download/millie-1931/Millie%20(1931).mp4
https://archive.org/download/silent-the-great-outdoors/The%20Great%20Outdoors.mp4
```

## File Selection Strategy

1. Fetch item metadata via `/metadata/{identifier}`
2. Filter `files` array for `format == "h.264"` or `format == "MPEG4"`
3. If multiple MP4 files exist, prefer the h.264 derivative (better codec, smaller size)
4. Each item typically produces one stream (one film = one MP4)
5. Skip items with no MP4/h.264 file

## Grouping Strategy

Group by **decade**:
- "1920s" -- Silent era classics
- "1930s" -- Early talkies, pre-Code Hollywood
- "1940s" -- Film noir, wartime films
- "1950s" -- Sci-fi, drama
- "1960s" -- Late classic era

The `year` field from the search API makes this straightforward.

## Stream Object Shape

```json
{
  "id": "millie-1931",
  "name": "Millie (1931)",
  "url": "https://archive.org/download/millie-1931/Millie%20(1931).mp4",
  "group": "1930s",
  "logo": "https://archive.org/services/img/millie-1931",
  "vod_type": "movie",
  "year": "1931",
  "tags": ["drama"]
}
```

## Config Fields

| Key | Label | Type | Required | Default |
|-----|-------|------|----------|---------|
| `max_films` | Max films to load | `number` | No | `100` |
| `decade` | Filter by decade | `select` | No | `all` |

Options for `decade`: all, 1920s, 1930s, 1940s, 1950s, 1960s. Selecting a decade narrows the year range in the search query.

## Refresh Flow

1. Build search query based on config (apply decade filter to year range if set)
2. Call discovery endpoint with `rows={max_films}`, sorted by downloads
3. For each returned item, call `/metadata/{identifier}`
4. Extract the best MP4 file from the files array
5. Build stream entries with decade grouping and year metadata
6. Cache results using `host_kv_set` (archive.org data is static)

## Estimated Stream Count

- With `max_films=100`: 100 films (one stream per film, typically)
- With `max_films=500`: 500 films
- Total available: ~7,265 films with MP4 files in the 1920-1965 range
- Expanding to all years: 12,075+ films

## API Key Requirement

**NO** -- Internet Archive's search and metadata APIs are fully public.

## Risks

1. **Rate limiting:** One metadata request per film. With 100 films, that is 101 HTTP requests per refresh. Mitigate with caching and reasonable defaults.
2. **Content quality variance:** Some "feature films" in the collection are actually shorts, home movies, or radio programs miscategorized. Sorting by downloads helps surface genuine films.
3. **File size:** MP4 files can be large (400MB+). This is fine for streaming (the URL is passed to the player, not downloaded by the plugin), but users need adequate bandwidth.
4. **Missing year metadata:** Some items lack a `year` field. These can be grouped under "Unknown Decade" or parsed from the title if it contains a year in parentheses.
5. **Non-English content:** The collection includes films in many languages. No reliable language metadata exists for filtering.
6. **Thumbnail quality:** `https://archive.org/services/img/{identifier}` provides thumbnails but quality varies -- some are auto-generated frames, not proper movie posters.
7. **WikiFlix alternative:** WikiFlix (wikiflix.org) was investigated but the domain now redirects through tracking parameters to what appears to be a parked/ad page. It is not a viable data source.
