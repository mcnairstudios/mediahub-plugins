# MediaHub Plugins

WASM source plugins for [MediaHub](https://github.com/mcnairstudios/mediahub). Drop a `.wasm` file into the plugins directory — no recompilation needed. Platform independent, small (38KB-423KB).

## Plugins

### Live TV & Video

| Plugin | Description | Streams |
|--------|-------------|---------|
| [iptvorg](plugins/iptvorg/) | Free live TV channels from iptv-org (HLS) | 15,000+ |
| [plutotv](plugins/plutotv/) | Free live TV — movies, sports, news, kids (HLS) | 250+ |
| [worldnews](plugins/worldnews/) | 24/7 live news — DW, Al Jazeera, NHK, France24 (HLS) | 25 |
| [trending](plugins/trending/) | YouTube trending videos via Piped API | Dynamic |
| [peertube](plugins/peertube/) | Federated video from PeerTube instances (HLS/MP4) | 50-150 |

### Movies & TV

| Plugin | Description | Streams |
|--------|-------------|---------|
| [publicdomain](plugins/publicdomain/) | Public domain classic films by decade (1920s-1970s) | 300+ |
| [archive](plugins/archive/) | Internet Archive — films, concerts, documentaries | 500+ |
| [cartoons](plugins/cartoons/) | Classic cartoons — Popeye, Superman, Betty Boop, Felix | 200+ |
| [trailers](plugins/trailers/) | Movie trailers from TMDB | ~40 |

### Radio & Music

| Plugin | Description | Streams |
|--------|-------------|---------|
| [radiobrowser](plugins/radiobrowser/) | Internet radio stations worldwide | 55,000+ |
| [somafm](plugins/somafm/) | Curated ad-free internet radio | 63 |
| [radiogarden](plugins/radiogarden/) | Live radio stations worldwide (Radio Garden) | 1,000+ |
| [ccmixter](plugins/ccmixter/) | Creative Commons music and remixes | 50-100 |

### Podcasts & Audiobooks

| Plugin | Description | Streams |
|--------|-------------|---------|
| [podcasts](plugins/podcasts/) | Podcasts via Apple iTunes Search API | 100+ |
| [librivox](plugins/librivox/) | Free public domain audiobooks | 500-2,000 |
| [oldtimeradio](plugins/oldtimeradio/) | Classic 1930s-1950s radio shows | 300+ |
| [ted](plugins/ted/) | TED Talks with direct MP4 video | 50-200 |

### Live Cameras

| Plugin | Description | Streams |
|--------|-------------|---------|
| [skylinecams](plugins/skylinecams/) | HD city, beach, and landmark webcams (HLS) | 500+ |
| [naturecams](plugins/naturecams/) | Wildlife, aquarium, bird, and ISS cams | 16+ |
| [outdoorcams](plugins/outdoorcams/) | Volcano, ski resort, and surf cams | 29 |
| [trafficcams](plugins/trafficcams/) | California highway traffic cameras (HLS) | 2,100+ |

### Science & Education

| Plugin | Description | Streams |
|--------|-------------|---------|
| [sciencetube](plugins/sciencetube/) | Science YouTube — Veritasium, 3Blue1Brown, PBS Space Time | ~210 |
| [nasa](plugins/nasa/) | NASA Video Library — launches, ISS, Mars | 100+ |
| [space](plugins/space/) | Space launches from all agencies | ~700 |

### Ambient & Niche

| Plugin | Description | Streams |
|--------|-------------|---------|
| [aerials](plugins/aerials/) | Apple TV 4K aerial screensaver videos | 182 |
| [slowtv](plugins/slowtv/) | Train journeys, fireplaces, nature walks | 28 |
| [operavision](plugins/operavision/) | Free opera from 44 European opera houses | ~28 |
| [demo](plugins/demo/) | Test streams (NASA Live, Big Buck Bunny, etc.) | 6 |

## Install

Pre-built binaries are in [`dist/`](dist/) — copy to MediaHub and restart:

```bash
cp dist/*.wasm $MEDIAHUB_PLUGINS_DIR/
```

## Build

```bash
make all          # Build everything (29 plugins)
make test         # Run all Rust plugin tests (725 tests)
make install MEDIAHUB_PLUGINS_DIR=/path  # Build and deploy
```

Build individual plugins:

```bash
make somafm       # Build just SomaFM
make plutotv      # Build just Pluto TV
```

## Write Your Own

- [Getting Started with Rust](docs/GETTING-STARTED-rust.md)
- [Getting Started with Go](docs/GETTING-STARTED-go.md)
- [Full Plugin Reference](PLUGIN_GUIDE.md)

## License

Same as [MediaHub](https://github.com/mcnairstudios/mediahub).
