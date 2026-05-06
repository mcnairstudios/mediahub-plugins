# MediaHub Plugins

WASM source plugins for [MediaHub](https://github.com/mcnairstudios/mediahub). Each plugin is a standalone `.wasm` file that provides streams to MediaHub — no recompilation needed. Platform independent, small (90KB-423KB).

## Plugins

| Plugin | Language | Binary | Description | Streams |
|--------|----------|--------|-------------|---------|
| [demo](plugins/demo/) | TinyGo | 325KB | Test streams (NASA Live, Big Buck Bunny, etc.) | 6 |
| [space](plugins/space/) | Rust | 90KB | Space launches from all agencies (Launch Library 2) | ~700 |
| [radiogarden](plugins/radiogarden/) | Rust | 183KB | Live radio stations worldwide (Radio Garden) | 1000+ |
| [trailers](plugins/trailers/) | TinyGo | 423KB | Movie trailers from TMDB | ~40 |

See [plugins/](plugins/) for details on each plugin.

## Quick Install

Pre-built binaries are in `dist/` — just copy to MediaHub:

```bash
cp dist/*.wasm $MEDIAHUB_PLUGINS_DIR/
# Default: $MEDIAHUB_DATA_DIR/plugins/
# Restart MediaHub — plugins load automatically
```

## Build From Source

```bash
# Build all plugins (requires TinyGo + Rust)
make all

# Build individually
make demo
make space
make radiogarden
make trailers

# Install to MediaHub
make install MEDIAHUB_PLUGINS_DIR=/path/to/plugins
```

## Write Your Own Plugin

Get started with one of these guides:

- **[Getting Started with Rust](docs/GETTING-STARTED-rust.md)** — smallest binaries (~90KB), memory safe, best WASM support
- **[Getting Started with TinyGo](docs/GETTING-STARTED-go.md)** — familiar Go syntax, larger binaries (~325KB)

Full contract reference: [PLUGIN_GUIDE.md](PLUGIN_GUIDE.md)

## Architecture

Plugins are pure data transforms — they never touch the DOM or the media pipeline:

```
Config (JSON) → Plugin → Stream List (JSON)
```

MediaHub provides four host functions to plugins:

| Function | Purpose |
|----------|---------|
| `host_http_request` | Make HTTP requests (via MediaHub's HTTP client) |
| `host_log` | Write log messages |
| `host_kv_get` | Read from plugin-scoped cache |
| `host_kv_set` | Write to plugin-scoped cache |

Any language that compiles to WASM with WASI support works: Rust, TinyGo, C/C++, Zig, AssemblyScript.

## Repository Structure

```
├── dist/                  # Pre-built .wasm binaries (platform independent)
├── docs/                  # Getting started guides
│   ├── GETTING-STARTED-rust.md
│   └── GETTING-STARTED-go.md
├── plugins/               # Plugin source code
│   ├── README.md          # Plugin index with language details
│   ├── demo/              # TinyGo — test streams
│   ├── space/             # Rust — space launches
│   ├── radiogarden/       # Rust — radio stations
│   └── trailers/          # TinyGo — movie trailers
├── Makefile               # Build all plugins
├── PLUGIN_GUIDE.md        # Full developer reference
└── README.md              # This file
```

## License

Same as [MediaHub](https://github.com/mcnairstudios/mediahub).
