# Demo Streams Plugin

Free public test streams for verifying MediaHub playback.

**Language:** TinyGo | **Binary:** 325KB | **Streams:** 6

## Streams

| Name | Type | Source |
|------|------|--------|
| NASA Live | Live HLS | akamaized.net |
| Bloomberg TV | Live HLS | akamaized.net |
| Big Buck Bunny | VOD | mux.dev |
| Elephant's Dream | VOD | mux.dev |
| Sintel | VOD | mux.dev |
| Tears of Steel | VOD | mux.dev |

## Config

No configuration needed — streams are hardcoded.

## Build

```bash
tinygo build -o demo.wasm -target=wasi -no-debug ./
```
