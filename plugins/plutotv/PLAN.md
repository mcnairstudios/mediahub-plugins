# Pluto TV Plugin Plan

## Feasibility: CONFIRMED

Pluto TV provides free, ad-supported live TV channels accessible without authentication via
public APIs. HLS streams are playable after a lightweight session bootstrap.

## API Flow (Two Steps)

### Step 1: Bootstrap Session

**Endpoint:** `GET https://boot.pluto.tv/v4/start`

Required query parameters:
- `appName=web`
- `appVersion=7.0.0`
- `clientID=<any-string>`
- `clientModelNumber=1.0`
- `deviceDNT=0`
- `deviceId=<any-string>`
- `deviceMake=Chrome`
- `deviceModel=web`
- `deviceType=web`
- `deviceVersion=15.0`

Returns JSON with:
- `sessionToken` -- JWT (valid ~24 hours, `refreshInSec: 28800`)
- `stitcherParams` -- pre-built query string for stitcher URLs
- `servers.stitcher` -- stitcher base URL (e.g., `https://cfd-v4-service-channel-stitcher-use1-1.prd.pluto.tv`)
- `session.activeRegion` -- detected region (GB, US, etc.)
- `EPG[]` -- partial channel list (starting channel only)

No API key, no user account, no cookies required.

### Step 2: Fetch Channel List

**Endpoint:** `GET https://api.pluto.tv/v2/channels`

No authentication required. Returns 200+ channels as a JSON array.

Each channel includes:
- `_id` -- channel ID
- `name` -- display name (e.g., "Pluto TV Movies")
- `slug` -- URL-friendly identifier
- `number` -- channel number
- `category` -- content category (Movies, Comedy, Crime Drama, Kids, Sports, News, etc.)
- `summary` -- channel description
- `colorLogoPNG` -- logo URL at `https://images.pluto.tv/channels/<id>/colorLogoPNG.png`
- `thumbnail` -- thumbnail URL
- `stitched.urls[0].url` -- pre-built HLS URL (but NOT directly playable)
- `stitched.urls[0].type` -- always `"hls"`

### Step 3: Construct Playable Stream URLs

Combine the stitcher base URL, channel path, and session params:

```
{servers.stitcher}/stitch/hls/channel/{channel._id}/master.m3u8?{stitcherParams}
```

The `stitcherParams` from the boot response includes `sessionID`, `sid`, `deviceId`, `marketingRegion`,
geo coordinates, and other required fields.

The resulting HLS master playlist contains multiple quality variants (640K to 3.3M bitrate)
with subtitles.

## Geo-Restriction Notes

- The API is **region-aware**: the boot endpoint detects the caller's IP and sets `activeRegion`.
- Channel availability varies by region. Testing from GB returned 204 channels. US typically
  returns 250+.
- The channel list from `/v2/channels` is **already filtered by IP geolocation** -- it only
  returns channels available in the caller's region.
- Streams are playable from at least: US, UK, EU countries, parts of Latin America.
- No VPN or spoofing needed -- just returns what's available for the caller's location.
- The plugin should note the detected region in logs so users understand what they're seeing.

## Plugin Architecture (Rust/WASM)

### Exported Functions

Following the `radiogarden` pattern:

1. **`describe()`** -- Return plugin descriptor:
   - `type: "plutotv"`
   - `label: "Pluto TV"`
   - `short_label: "PLUTO"`
   - `color: "#00b4ff"` (Pluto TV brand blue)
   - `layout: "grouped_list"`, `group_by: "group"` (group by category)
   - `searchable: true`, `sortable: true`
   - No config fields needed (no user configuration required)

2. **`refresh(config_ptr, config_len)`** -- Fetch and return all channels:
   - Call boot API to get session token + stitcher params
   - Call `/v2/channels` to get full channel list
   - For each channel, construct the playable HLS URL using stitcher base + channel ID + stitcher params
   - Return streams grouped by `category`
   - Cache the session token in KV store (valid for ~8 hours)

3. **`interact(action_ptr, action_len)`** -- Not needed initially (no config fields).
   Could later support category filtering.

### Stream Object Mapping

```
Stream {
    id:       channel._id,
    name:     channel.name,
    url:      "{stitcher_base}/stitch/hls/channel/{_id}/master.m3u8?{stitcherParams}",
    group:    channel.category,
    logo:     "https://images.pluto.tv/channels/{_id}/colorLogoPNG.png",
    vod_type: "",            // live TV, not VOD
    tags:     [channel.number.to_string()],
}
```

### Session Management

- On `refresh()`, check KV store for a cached session token.
- If missing or expired, call the boot API to get a fresh one.
- The boot API returns `refreshInSec: 28800` (8 hours), so sessions are long-lived.
- Generate a stable `deviceId` and `clientID` (UUID stored in KV) for consistent sessions.

### Error Handling

- If boot API fails, log error and return empty stream list.
- If channel API fails, log error and return empty stream list.
- If a session token expires mid-use, the stitcher will return 400 -- the host player
  should trigger a refresh which will get a new session.

## Dependencies

Same as `radiogarden`:
- `serde` + `serde_json` for JSON parsing
- `wasm32-unknown-unknown` target
- Host functions: `host_http_request`, `host_log`, `host_kv_get`, `host_kv_set`

## Tested API Responses

- `/v2/channels` returns HTTP 200 with 204 channels (from GB)
- Boot API returns HTTP 200 with valid JWT session token
- HLS master playlist returns HTTP 200 with 5 quality variants (640K-3.3M)
- Subtitles included in HLS manifest
- No DRM on HLS streams (plain TS segments)
- All requests work with a simple `User-Agent: Mozilla/5.0` header

## Risks and Mitigations

| Risk | Mitigation |
|------|-----------|
| API is unofficial, could change | Version-pin `appVersion` param; monitor for 400s |
| Session tokens could become stricter | Boot API currently accepts any clientID/deviceId strings |
| Geo-blocking in some regions | Plugin logs detected region; streams are whatever's available |
| Ad stitching in streams | Ads are baked into the HLS stream server-side; no way to skip, but no extra work needed |
| Rate limiting | Cache session tokens (8hr TTL); channel list could also be cached |
