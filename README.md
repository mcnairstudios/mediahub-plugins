# MediaHub Plugins

WASM source plugins for [MediaHub](https://github.com/mcnairstudios/mediahub). Drop a `.wasm` file into the plugins directory — no recompilation needed. Platform independent, small (90KB-423KB).

## Plugins

| Plugin | Description | Streams |
|--------|-------------|---------|
| [demo](plugins/demo/) | Test streams (NASA Live, Big Buck Bunny, etc.) | 6 |
| [space](plugins/space/) | Space launches from all agencies (Launch Library 2) | ~700 |
| [radiogarden](plugins/radiogarden/) | Live radio stations worldwide (Radio Garden) | 1000+ |
| [trailers](plugins/trailers/) | Movie trailers from TMDB | ~40 |

## Install

Pre-built binaries are in [`dist/`](dist/) — copy to MediaHub and restart:

```bash
cp dist/*.wasm $MEDIAHUB_PLUGINS_DIR/
```

## Build

```bash
make all          # Build everything
make install MEDIAHUB_PLUGINS_DIR=/path  # Build and deploy
```

## Write Your Own

- [Getting Started with Rust](docs/GETTING-STARTED-rust.md)
- [Getting Started with Go](docs/GETTING-STARTED-go.md)
- [Full Plugin Reference](PLUGIN_GUIDE.md)

## License

Same as [MediaHub](https://github.com/mcnairstudios/mediahub).
