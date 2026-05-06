# Traffic Cameras Plugin - Feasibility Plan

## Verdict: FEASIBLE (USA only, with caveats)

The plugin is feasible primarily using **US state DOT HLS video streams**. The UK
and most other countries only provide refreshing JPEG images, not true video.

---

## Research Summary

### What works (actual HLS video streams, no API key)

**Caltrans (California DOT)** - PRIMARY SOURCE
- Official JSON API: `https://cwwp2.dot.ca.gov/data/d{N}/cctv/cctvStatusD{NN}.json`
- No API key required. No authentication. Free.
- 12 districts, 3,476 cameras total, **2,181 with HLS video** (`streamingVideoURL` field)
- Stream format: `https://wzmedia.dot.ca.gov/D{N}/{camera}.stream/playlist.m3u8`
- H.264 video, HLS protocol
- JSON includes: lat/lon, route, direction, location name, district, county
- Tested and confirmed working (HTTP 200, valid m3u8 with avc1 codec)

**Wisconsin DOT (511WI)**
- HLS streams at `https://cctv1.dot.wi.gov:443/rtplive/{CAMERA_ID}/playlist.m3u8`
- Streams are publicly accessible without auth (confirmed HTTP 200)
- However, the *camera listing API* requires a free developer key
- ~300 cameras with video

**Delaware DOT**
- HLS streams at `http://video.deldot.gov:1935/live/{CAMERA_ID}.stream/playlist.m3u8`
- Streams publicly accessible (confirmed HTTP 200)
- ~295 cameras with video
- No public JSON API found for camera discovery

### What does NOT work

**UK - National Highways / Traffic England**
- All public feeds are **refreshing JPEG images** (updated every few minutes)
- Live video requires login to HETC system (restricted to operational staff)
- No public video streams. NOT FEASIBLE.

**UK - TfL JamCams (London)**
- 900+ cameras but ALL are **JPEG images refreshing every 3-5 minutes**
- TfL Unified API provides image URLs only (requires free API key anyway)
- NOT FEASIBLE for video.

**Netherlands (Rijkswaterstaat)**
- Motorway feeds are **JPEG snapshots refreshing every 1-2 minutes**
- No public HLS video streams found
- NOT FEASIBLE.

**Australia (NSW, Queensland)**
- Live Traffic NSW: images refreshing every 60 seconds
- QLDTraffic: JPEG snapshots
- NOT FEASIBLE.

**Canada (Ontario 511)**
- Appears to be JPEG images
- NOT FEASIBLE for video.

**Japan (JARTIC)**
- No public API. Camera feeds accessed through web portals only.
- NOT FEASIBLE.

**511 APIs (NY, GA, LA, etc.)**
- These 511 systems have HLS video streams BUT require a free developer API key
- The streams themselves may be publicly accessible, but discovering camera IDs
  requires the API key
- Violates the "no API keys" constraint for the discovery/listing step

### Third-party aggregator: OpenTrafficCamMap
- GitHub: `AidanWelch/OpenTrafficCamMap`
- 7,029 US cameras in `cameras/USA.json`
- 2,926 with M3U8 HLS streams (but many are stale/dead)
- Could be used as a supplementary data source
- License: needs verification

---

## Recommended Architecture

### Data Sources (in priority order)

1. **Caltrans JSON API** (primary, ~2,181 video streams)
   - Reliable, official, well-structured, no auth needed
   - Covers all of California across 12 districts

2. **OpenTrafficCamMap USA.json** (supplementary)
   - Adds Alabama, Delaware, Georgia, Wisconsin streams
   - Data may be stale; needs liveness checking
   - Could add ~500-1000 additional working streams

### Plugin Design (Rust WASM)

```
describe() -> Descriptor
  type: "trafficcams"
  label: "Traffic Cameras"
  short_label: "CAMS"
  config_fields:
    - "regions" (multi-select of Caltrans districts + other states)
  view: { layout: "grouped_list", group_by: "group", searchable: true }

refresh(config) -> RefreshResponse
  1. For each selected Caltrans district:
     - GET https://cwwp2.dot.ca.gov/data/d{N}/cctv/cctvStatusD{NN}.json
     - Filter to cameras where streamingVideoURL is non-empty and inService=true
     - Map to Stream { id, name, url (HLS), group (district/route), tags }
  2. Optionally fetch OpenTrafficCamMap data for non-CA states
  3. Return combined stream list

interact(action) -> Results
  - "search_cameras": search by route, location, or county
```

### Stream Object Fields
```json
{
  "id": "caltrans-d7-cctv-196",
  "name": "I-5 at Burbank Blvd (NB)",
  "url": "https://wzmedia.dot.ca.gov/D7/CCTV-196.stream/playlist.m3u8",
  "group": "District 7 - Los Angeles",
  "logo": "",
  "vod_type": "",
  "tags": ["california", "i-5", "los-angeles", "northbound"]
}
```

### Grouping Strategy
- Group by Caltrans district (each maps to a geographic region):
  - D1: Northwest, D2: Northeast, D3: Sacramento, D4: Bay Area,
    D5: Central Coast, D6: Fresno, D7: Los Angeles, D8: San Bernardino,
    D9: Bishop, D10: Stockton, D11: San Diego, D12: Orange County
- For non-CA states: group by state name

---

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Caltrans streams go offline | Individual cameras unavailable | Large pool (2000+); mark inService=false cameras |
| Caltrans changes API format | Plugin breaks | Simple JSON; unlikely to change often |
| OpenTrafficCamMap data goes stale | Dead stream URLs | Use as secondary source; Caltrans is primary |
| wzmedia.dot.ca.gov rate limits | Refresh blocked | Cache camera list in KV store; refresh hourly |
| Non-US video sources remain image-only | Limited geographic coverage | Accept US-only scope; revisit if other countries add HLS |

## Estimated Effort
- Small/Medium: 1-2 days
- Mostly straightforward HTTP+JSON parsing
- Pattern matches radiogarden plugin closely
- Main complexity: parsing Caltrans JSON structure (nested cctv objects)

## Open Questions
1. Should we include streams from OpenTrafficCamMap or keep it Caltrans-only for reliability?
2. Should the plugin periodically test stream liveness (HEAD request to m3u8)?
3. Is US-only coverage acceptable, or should we wait for more countries to offer video?
