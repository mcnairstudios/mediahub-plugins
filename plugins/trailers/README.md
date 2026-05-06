# Movie Trailers Plugin

Trending and now-playing movie trailers from TMDB.

**Language:** TinyGo | **Binary:** 423KB | **Streams:** ~40

## Data Source

[TMDB API v3](https://developer.themoviedb.org) — requires a free API key.

## Stream Format

- **Name:** "Movie Title - Trailer Name"
- **URL:** YouTube watch URL (resolved by MediaHub's YouTube resolver at playback)
- **Group:** "Trending" or "Now Playing"
- **Logo:** TMDB poster image (w300)
- **Tags:** None
- **Episode Name:** Movie overview/description

## Config

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `tmdb_key` | password | Yes | TMDB API key (get free at themoviedb.org) |

## Build

```bash
tinygo build -o trailers.wasm -target=wasi -no-debug ./
```
