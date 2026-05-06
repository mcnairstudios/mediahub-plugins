# Getting Started: TinyGo Plugin

Write a MediaHub source plugin in TinyGo. Familiar Go syntax, larger binaries (~325-423KB).

## Prerequisites

```bash
# macOS
brew install tinygo-org/tools/tinygo

# Linux
wget https://github.com/tinygo-org/tinygo/releases/download/v0.41.1/tinygo_0.41.1_amd64.deb
sudo dpkg -i tinygo_0.41.1_amd64.deb

# Verify
tinygo version
```

## Create a New Plugin

```bash
mkdir my-plugin && cd my-plugin
go mod init my-plugin
```

**main.go:**
```go
package main

import (
	"encoding/json"
	"unsafe"
)

// Host function imports (provided by MediaHub)
//go:wasmimport env host_log
func hostLog(level uint32, msgPtr uint32, msgLen uint32)

//go:wasmimport env host_http_request
func hostHTTPRequest(urlPtr, urlLen, methodPtr, methodLen, headersPtr, headersLen, bodyPtr, bodyLen uint32) uint64

//go:wasmimport env host_kv_get
func hostKVGet(keyPtr, keyLen uint32) uint64

//go:wasmimport env host_kv_set
func hostKVSet(keyPtr, keyLen, valPtr, valLen uint32)

// Memory management (required exports)
//export alloc
func alloc(size uint32) uint32 {
	buf := make([]byte, size)
	return uint32(uintptr(unsafe.Pointer(&buf[0])))
}

//export dealloc
func dealloc(ptr uint32, size uint32) {}

// Helpers
func packPtrLen(ptr, length uint32) uint64 {
	return (uint64(ptr) << 32) | uint64(length)
}

func unpackPtrLen(packed uint64) (uint32, uint32) {
	return uint32(packed >> 32), uint32(packed & 0xFFFFFFFF)
}

func returnJSON(v any) uint64 {
	data, err := json.Marshal(v)
	if err != nil || len(data) == 0 {
		return 0
	}
	ptr := uint32(uintptr(unsafe.Pointer(&data[0])))
	return packPtrLen(ptr, uint32(len(data)))
}

func readInput(ptr, length uint32) []byte {
	return unsafe.Slice((*byte)(unsafe.Pointer(uintptr(ptr))), length)
}

func logInfo(msg string) {
	b := []byte(msg)
	hostLog(1, uint32(uintptr(unsafe.Pointer(&b[0]))), uint32(len(b)))
}

func httpGet(url string) []byte {
	urlBytes := []byte(url)
	methodBytes := []byte("GET")
	headersBytes := []byte("{}")
	result := hostHTTPRequest(
		uint32(uintptr(unsafe.Pointer(&urlBytes[0]))), uint32(len(urlBytes)),
		uint32(uintptr(unsafe.Pointer(&methodBytes[0]))), uint32(len(methodBytes)),
		uint32(uintptr(unsafe.Pointer(&headersBytes[0]))), uint32(len(headersBytes)),
		0, 0, // no body — NEVER use &emptySlice[0]
	)
	if result == 0 {
		return nil
	}
	ptr, length := unpackPtrLen(result)
	if length == 0 {
		return nil
	}
	return unsafe.Slice((*byte)(unsafe.Pointer(uintptr(ptr))), length)
}

// Plugin exports

//export describe
func describe() uint64 {
	return returnJSON(map[string]any{
		"type":          "my-plugin",
		"label":         "My Plugin",
		"short_label":   "MP",
		"color":         "#4caf50",
		"version":       "1.0.0",
		"description":   "My custom source plugin",
		"config_fields": []any{},
		"view": map[string]any{
			"layout":     "grouped_list",
			"group_by":   "group",
			"searchable": true,
		},
	})
}

//export refresh
func refresh(configPtr, configLen uint32) uint64 {
	_ = readInput(configPtr, configLen)
	logInfo("my-plugin: refreshing")

	// Fetch data from your API
	body := httpGet("https://api.example.com/streams")
	if body == nil {
		return returnJSON(map[string]any{"streams": []any{}})
	}

	// Parse and build streams
	streams := []map[string]any{
		{
			"id":    "example-1",
			"name":  "Example Stream",
			"url":   "https://example.com/stream.m3u8",
			"group": "Examples",
			"tags":  []string{"live"},
		},
	}

	return returnJSON(map[string]any{"streams": streams})
}

//export interact
func interact(actionPtr, actionLen uint32) uint64 {
	_ = readInput(actionPtr, actionLen)
	return returnJSON(map[string]any{})
}

func main() {}
```

**Makefile:**
```makefile
.PHONY: build clean

build:
	tinygo build -o my-plugin.wasm -target=wasi -no-debug ./

clean:
	rm -f my-plugin.wasm
```

## Build

```bash
tinygo build -o my-plugin.wasm -target=wasi -no-debug ./
# Output: my-plugin.wasm (~325KB)
```

## Install

```bash
cp my-plugin.wasm $MEDIAHUB_PLUGINS_DIR/
# Restart MediaHub — plugin loads automatically
```

## Gotchas

- **Empty body:** Always pass `0, 0` for HTTP body pointer/length on GET requests. Never write `&emptySlice[0]` — TinyGo's dead code elimination will strip the host import entirely.
- **`func main() {}`** is required but must be empty — TinyGo calls it on WASM init.
- **No goroutines** — TinyGo WASM doesn't support concurrency.
- **Limited stdlib** — `net/http`, `os`, `fmt.Println` are not available. Use host functions instead.

## Reference

- [Demo plugin](../plugins/demo/) — simplest possible plugin, zero HTTP calls
- [Trailers plugin](../plugins/trailers/) — TinyGo plugin with TMDB API, pagination, YouTube URLs
