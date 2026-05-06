# MediaHub Plugins

WASM source plugins for [MediaHub](https://github.com/mcnairstudios/mediahub). Drop a `.wasm` file into the plugins directory — no recompilation needed. Platform independent, small (90KB-423KB).

## Plugins

### Live TV & Video

| Plugin | Description | Streams |
|--------|-------------|---------|
| [iptvorg](plugins/iptvorg/) | Free live TV channels from iptv-org (HLS) | 15,000+ |
| [plutotv](plugins/plutotv/) | Free live TV — movies, sports, news, kids (HLS) | 250+ |
| [trending](plugins/trending/) | YouTube trending videos via Piped API | Dynamic |
| [peertube](plugins/peertube/) | Federated video from PeerTube instances (HLS/MP4) | Thousands |

### Movies & TV

| Plugin | Description | Streams |
|--------|-------------|---------|
| [publicdomain](plugins/publicdomain/) | Public domain classic films from Internet Archive | 27,000+ |
| [archive](plugins/archive/) | Internet Archive — films, concerts, documentaries | 36,000+ |
| [trailers](plugins/trailers/) | Movie trailers from TMDB | ~40 |

### Radio & Music

| Plugin | Description | Streams |
|--------|-------------|---------|
| [radiobrowser](plugins/radiobrowser/) | Internet radio stations worldwide | 55,000+ |
| [somafm](plugins/somafm/) | Curated ad-free internet radio | 63 |
| [radiogarden](plugins/radiogarden/) | Live radio stations worldwide (Radio Garden) | 1,000+ |
| [ccmixter](plugins/ccmixter/) | Creative Commons music and remixes | 70,000+ |

### Podcasts & Audiobooks

| Plugin | Description | Streams |
|--------|-------------|---------|
| [podcasts](plugins/podcasts/) | Podcasts via Apple iTunes Search API | Millions |
| [librivox](plugins/librivox/) | Free public domain audiobooks | 19,000+ |
| [oldtimeradio](plugins/oldtimeradio/) | Classic 1930s-1950s radio shows | 8,800+ |

### Live Cameras

| Plugin | Description | Streams |
|--------|-------------|---------|
| [skylinecams](plugins/skylinecams/) | HD city, beach, and landmark webcams (HLS) | 500+ |
| [naturecams](plugins/naturecams/) | Wildlife, aquarium, bird, and ISS cams | 30+ |
| [outdoorcams](plugins/outdoorcams/) | Volcano, ski resort, and surf cams | 37+ |
| [trafficcams](plugins/trafficcams/) | California highway traffic cameras (HLS) | 2,100+ |

### Ambient & Niche

| Plugin | Description | Streams |
|--------|-------------|---------|
| [aerials](plugins/aerials/) | Apple TV 4K aerial screensaver videos | 182 |
| [slowtv](plugins/slowtv/) | Train journeys, fireplaces, nature walks | 28 |
| [nasa](plugins/nasa/) | NASA Video Library — launches, ISS, Mars | 6,900+ |
| [operavision](plugins/operavision/) | Free opera from 44 European opera houses | ~28 |
| [space](plugins/space/) | Space launches from all agencies | ~700 |
| [demo](plugins/demo/) | Test streams (NASA Live, Big Buck Bunny, etc.) | 6 |

## Install

Pre-built binaries are in [`dist/`](dist/) — copy to MediaHub and restart:

```bash
cp dist/*.wasm $MEDIAHUB_PLUGINS_DIR/
```

## Build

```bash
make all          # Build everything (24 plugins)
make test         # Run all Rust plugin tests (569 tests)
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
