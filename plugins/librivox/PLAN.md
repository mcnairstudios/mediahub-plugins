# LibriVox Plugin Plan

## Overview

- **Plugin name:** LibriVox
- **Type:** `librivox`
- **Label:** LibriVox Audiobooks
- **Short label:** LBV
- **Description:** Free public domain audiobooks from LibriVox, with direct MP3 playback via Internet Archive
- **Language:** Go (TinyGo, matching the demo plugin pattern)

## API Endpoints

No API key required. All endpoints return JSON when `format=json` is appended.

1. **List audiobooks:** `https://librivox.org/api/feed/audiobooks?format=json&limit=N&offset=M`
   - Supports filtering: `&title=...`, `&author=...`, `&language=...`
   - Returns: id, title, description, language, num_sections, authors[], totaltime, url_librivox, url_zip_file, url_rss
   - Does NOT return per-chapter audio URLs (must use audiotracks endpoint)

2. **List chapters/sections:** `https://librivox.org/api/feed/audiotracks?project_id=BOOK_ID&format=json`
   - Returns: section id, section_number, title, **listen_url** (direct MP3), language, playtime (seconds)
   - The `listen_url` is a direct archive.org MP3 link (e.g., `https://www.archive.org/download/.../file_64kb.mp3`)

3. **Search:** Title and author search via query params on the audiobooks endpoint

## Audio URL Verification

- The `listen_url` from the audiotracks endpoint points directly to `https://www.archive.org/download/.../*.mp3`
- This 302-redirects to `https://archive.org/download/.../*.mp3` which serves the MP3 directly
- No additional hops, authentication, or scraping required
- Files are 64kbps MP3, suitable for audiobook speech content

## Stream Mapping

Each **chapter** (section) becomes one stream:

| Stream field   | Source                                              |
|---------------|-----------------------------------------------------|
| `id`          | `"lbv-{book_id}-{section_number}"`                  |
| `name`        | `"{chapter_title}"` (e.g., "Marseilles - The Arrival") |
| `url`         | `listen_url` from audiotracks API (direct MP3)       |
| `group`       | `"{book_title} - {author}"` (groups chapters by book)|
| `logo`        | None readily available per-book (could use LibriVox logo) |
| `vod_type`    | `"episode"`                                          |
| `episode_name`| `"Ch. {section_number}: {title}"`                    |
| `tags`        | `[language, "audiobook"]`                            |

Books are grouped so all chapters of a book appear together in the UI.

## Config Fields

| Key        | Label       | Type     | Required | Default   |
|-----------|-------------|----------|----------|-----------|
| `language`| Language    | `select` | No       | `English` |
| `limit`   | Max Books   | `select` | No       | `25`      |

Options for language: English, German, French, Spanish, Chinese, Russian, etc.
Options for limit: 10, 25, 50, 100.

No API key field needed.

## Refresh Strategy

1. Fetch audiobooks list: `GET /api/feed/audiobooks?format=json&language={lang}&limit={limit}`
2. For each book, fetch chapters: `GET /api/feed/audiotracks?project_id={id}&format=json`
3. Flatten all chapters into streams, grouped by book title + author

**Concern:** Step 2 requires one HTTP request per book. For 25 books, that is 26 total requests. This is acceptable but should be noted. Caching via `host_kv_set/get` can reduce repeat fetches.

## Interactions

- **Search:** `interact` can support title/author search by calling the audiobooks API with `&title=` or `&author=` params, then fetching audiotracks for results.

## Estimated Stream Count

- LibriVox has 19,000+ audiobooks, with typically 10-100+ chapters each
- With default config (25 books), a refresh will return roughly 250-2,500 chapter streams
- Pagination via offset allows browsing the full catalog over time

## Risks and Limitations

1. **N+1 HTTP requests:** Each book requires a separate audiotracks call. Mitigated by caching and reasonable default limit.
2. **No cover art in API:** The audiobooks endpoint does not return a cover image URL. The RSS feed has an `itunes:image` field pointing to archive.org, but parsing RSS in WASM adds complexity. Could use a static LibriVox logo for all streams.
3. **No genre filtering:** The API does not support genre/category filtering. Language is the main available filter.
4. **archive.org reliability:** MP3 URLs depend on Internet Archive availability. Generally very reliable but occasionally slow.
5. **302 redirect on MP3 URLs:** The `www.archive.org` URLs redirect to `archive.org`. Most audio players handle this transparently.
6. **Rate limiting:** LibriVox API docs do not mention rate limits, but aggressive polling should be avoided. The refresh interval should be generous (e.g., hourly or manual).

## Feasibility Assessment

**FEASIBLE.** The LibriVox API is open, returns structured JSON, requires no authentication, and provides direct MP3 URLs via the audiotracks endpoint. The plugin pattern maps cleanly: chapters become streams, books become groups.
