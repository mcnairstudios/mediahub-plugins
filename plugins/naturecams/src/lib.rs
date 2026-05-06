use serde::Serialize;
use serde_json::Value;
use std::slice;

#[cfg(test)]
mod tests;

// ============================================================
// Host function imports
// ============================================================

#[cfg(not(test))]
extern "C" {
    fn host_log(level: u32, msg_ptr: u32, msg_len: u32);
    fn host_http_request(
        url_ptr: u32, url_len: u32,
        method_ptr: u32, method_len: u32,
        headers_ptr: u32, headers_len: u32,
        body_ptr: u32, body_len: u32,
    ) -> u64;
    fn host_kv_get(key_ptr: u32, key_len: u32) -> u64;
    fn host_kv_set(key_ptr: u32, key_len: u32, val_ptr: u32, val_len: u32);
}

// ============================================================
// Memory management exports
// ============================================================

#[no_mangle]
pub extern "C" fn alloc(size: u32) -> u32 {
    let mut buf: Vec<u8> = Vec::with_capacity(size as usize);
    buf.resize(size as usize, 0);
    let ptr = buf.as_ptr() as u32;
    std::mem::forget(buf);
    ptr
}

#[no_mangle]
pub extern "C" fn dealloc(_ptr: u32, _size: u32) {
    // No-op in WASM — memory reclaimed on module close.
}

// ============================================================
// Helpers
// ============================================================

fn pack_ptr_len(ptr: u32, len: u32) -> u64 {
    ((ptr as u64) << 32) | (len as u64)
}

#[allow(dead_code)]
fn unpack_ptr_len(packed: u64) -> (u32, u32) {
    let ptr = (packed >> 32) as u32;
    let len = (packed & 0xFFFFFFFF) as u32;
    (ptr, len)
}

fn read_input(ptr: u32, len: u32) -> Vec<u8> {
    unsafe { slice::from_raw_parts(ptr as *const u8, len as usize).to_vec() }
}

fn return_json<T: Serialize>(value: &T) -> u64 {
    match serde_json::to_vec(value) {
        Ok(data) => {
            let ptr = data.as_ptr() as u32;
            let len = data.len() as u32;
            std::mem::forget(data);
            pack_ptr_len(ptr, len)
        }
        Err(_) => 0,
    }
}

#[cfg(not(test))]
fn log_info(msg: &str) {
    let bytes = msg.as_bytes();
    unsafe { host_log(1, bytes.as_ptr() as u32, bytes.len() as u32) }
}

#[cfg(not(test))]
fn log_error(msg: &str) {
    let bytes = msg.as_bytes();
    unsafe { host_log(3, bytes.as_ptr() as u32, bytes.len() as u32) }
}

#[cfg(test)]
fn log_info(_msg: &str) {}

#[cfg(test)]
fn log_error(_msg: &str) {}

#[cfg(not(test))]
fn http_get(url: &str) -> Option<Vec<u8>> {
    let url_bytes = url.as_bytes();
    let method = b"GET";
    let headers = b"{}";
    let body = b"";

    let result = unsafe {
        host_http_request(
            url_bytes.as_ptr() as u32, url_bytes.len() as u32,
            method.as_ptr() as u32, method.len() as u32,
            headers.as_ptr() as u32, headers.len() as u32,
            body.as_ptr() as u32, body.len() as u32,
        )
    };

    log_info(&format!("http result raw: {}", result));

    if result == 0 {
        log_info("http result is 0");
        return None;
    }

    let (ptr, len) = unpack_ptr_len(result);
    log_info(&format!("http unpacked ptr={} len={}", ptr, len));
    if len == 0 {
        return None;
    }

    Some(unsafe { slice::from_raw_parts(ptr as *const u8, len as usize).to_vec() })
}

#[allow(dead_code)]
#[cfg(not(test))]
fn kv_get(key: &str) -> Option<String> {
    let kb = key.as_bytes();
    let result = unsafe { host_kv_get(kb.as_ptr() as u32, kb.len() as u32) };
    if result == 0 {
        return None;
    }
    let (ptr, len) = unpack_ptr_len(result);
    if len == 0 {
        return None;
    }
    let data = unsafe { slice::from_raw_parts(ptr as *const u8, len as usize) };
    Some(String::from_utf8_lossy(data).to_string())
}

#[allow(dead_code)]
#[cfg(not(test))]
fn kv_set(key: &str, value: &str) {
    let kb = key.as_bytes();
    let vb = value.as_bytes();
    unsafe {
        host_kv_set(
            kb.as_ptr() as u32, kb.len() as u32,
            vb.as_ptr() as u32, vb.len() as u32,
        )
    }
}

// ============================================================
// Data types -- Plugin metadata
// ============================================================

#[derive(Serialize)]
struct Descriptor {
    r#type: &'static str,
    label: &'static str,
    short_label: &'static str,
    color: &'static str,
    version: &'static str,
    description: &'static str,
    config_fields: Vec<Value>,
    view: View,
    interactions: Vec<Value>,
}

#[derive(Serialize)]
struct View {
    layout: &'static str,
    group_by: &'static str,
    searchable: bool,
    sortable: bool,
}

// ============================================================
// Data types -- Refresh output
// ============================================================

#[derive(Serialize, Clone, Debug)]
pub struct Stream {
    pub id: String,
    pub name: String,
    pub url: String,
    pub group: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logo: Option<String>,
    pub vod_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub episode_name: Option<String>,
}

#[derive(Serialize)]
struct RefreshResponse {
    streams: Vec<Stream>,
}

// ============================================================
// Hardcoded stream definitions
// ============================================================

struct HardcodedCam {
    id: &'static str,
    name: &'static str,
    url: &'static str,
    group: &'static str,
    logo: Option<&'static str>,
    tags: &'static [&'static str],
    episode_name: Option<&'static str>,
}

#[allow(dead_code)]
const YOUTUBE_BASE: &str = "https://www.youtube.com/watch?v=";

/// Monterey Bay Aquarium cams -- these are long-running YouTube live stream IDs.
/// Video IDs can change when streams restart; these are representative defaults.
const HARDCODED_CAMS: &[HardcodedCam] = &[
    // === Aquarium: Monterey Bay Aquarium ===
    HardcodedCam {
        id: "mba-shark-cam",
        name: "Monterey Bay Aquarium - Shark Cam",
        url: "https://www.youtube.com/watch?v=eXMCJkm1FBo",
        group: "Aquarium",
        logo: Some("https://www.montereybayaquarium.org/globalassets/mba/images/og-image.jpg"),
        tags: &["live", "aquarium", "sharks"],
        episode_name: Some("Live view of the Open Sea exhibit featuring sharks, sea turtles, and schooling fish"),
    },
    HardcodedCam {
        id: "mba-sea-otter-cam",
        name: "Monterey Bay Aquarium - Sea Otter Cam",
        url: "https://www.youtube.com/watch?v=n5fHGdpSr-4",
        group: "Aquarium",
        logo: Some("https://www.montereybayaquarium.org/globalassets/mba/images/og-image.jpg"),
        tags: &["live", "aquarium", "otters"],
        episode_name: Some("Live view of rescued southern sea otters in their indoor habitat"),
    },
    HardcodedCam {
        id: "mba-jellyfish-cam",
        name: "Monterey Bay Aquarium - Jellyfish Cam",
        url: "https://www.youtube.com/watch?v=YLTH2vOAnMA",
        group: "Aquarium",
        logo: Some("https://www.montereybayaquarium.org/globalassets/mba/images/og-image.jpg"),
        tags: &["live", "aquarium", "jellyfish"],
        episode_name: Some("Moon jellies drifting in a kreisel tank with changing LED lights"),
    },
    HardcodedCam {
        id: "mba-kelp-forest-cam",
        name: "Monterey Bay Aquarium - Kelp Forest Cam",
        url: "https://www.youtube.com/watch?v=eQ-ala2-F_8",
        group: "Aquarium",
        logo: Some("https://www.montereybayaquarium.org/globalassets/mba/images/og-image.jpg"),
        tags: &["live", "aquarium", "kelp"],
        episode_name: Some("One of the tallest kelp forest exhibits in the world, home to leopard sharks and sardines"),
    },
    HardcodedCam {
        id: "mba-open-sea-cam",
        name: "Monterey Bay Aquarium - Open Sea Cam",
        url: "https://www.youtube.com/watch?v=eXMCJkm1FBo",
        group: "Aquarium",
        logo: Some("https://www.montereybayaquarium.org/globalassets/mba/images/og-image.jpg"),
        tags: &["live", "aquarium", "ocean"],
        episode_name: Some("Million-gallon Open Sea exhibit with tuna, sea turtles, and sharks"),
    },
    HardcodedCam {
        id: "mba-aviary-cam",
        name: "Monterey Bay Aquarium - Aviary Cam",
        url: "https://www.youtube.com/watch?v=xXKmCUoSJLg",
        group: "Aquarium",
        logo: Some("https://www.montereybayaquarium.org/globalassets/mba/images/og-image.jpg"),
        tags: &["live", "aquarium", "birds"],
        episode_name: Some("Shorebirds and waterfowl in the aquarium's aviary exhibit"),
    },

    // === Birds: Cornell Lab Bird Cams ===
    HardcodedCam {
        id: "cornell-feeder-cam",
        name: "Cornell Lab - FeederWatch Cam",
        url: "https://www.youtube.com/watch?v=N609loYkFJo",
        group: "Birds",
        logo: Some("https://www.allaboutbirds.org/cams/assets/images/cornell-lab-logo.png"),
        tags: &["live", "birds", "feeder"],
        episode_name: Some("Bird feeder at the Cornell Lab of Ornithology in Ithaca, New York"),
    },
    HardcodedCam {
        id: "cornell-panama-cam",
        name: "Cornell Lab - Panama Fruit Feeder Cam",
        url: "https://www.youtube.com/watch?v=0oXpcGqMRCU",
        group: "Birds",
        logo: Some("https://www.allaboutbirds.org/cams/assets/images/cornell-lab-logo.png"),
        tags: &["live", "birds", "tropical"],
        episode_name: Some("Tropical birds visiting a fruit feeder in the Panama rainforest canopy"),
    },
    HardcodedCam {
        id: "cornell-red-tailed-hawk",
        name: "Cornell Lab - Red-tailed Hawk Nest",
        url: "https://www.youtube.com/watch?v=EM7B4oflBiA",
        group: "Birds",
        logo: Some("https://www.allaboutbirds.org/cams/assets/images/cornell-lab-logo.png"),
        tags: &["live", "birds", "hawk", "nest"],
        episode_name: Some("Red-tailed hawk nest cam at Cornell University campus"),
    },

    // === Wildlife: Explore.org partner cams ===
    HardcodedCam {
        id: "explore-katmai-bears",
        name: "Katmai National Park - Brown Bear Cam",
        url: "https://www.youtube.com/watch?v=qjBDel1jHnI",
        group: "Wildlife",
        logo: Some("https://explore.org/assets/explore-org-og.jpg"),
        tags: &["live", "wildlife", "bears"],
        episode_name: Some("Brown bears fishing for salmon at Brooks Falls, Katmai National Park, Alaska"),
    },
    HardcodedCam {
        id: "explore-african-waterhole",
        name: "Africam - Tembe Elephant Park",
        url: "https://www.youtube.com/watch?v=iO4Mcr4xvBc",
        group: "Wildlife",
        logo: Some("https://explore.org/assets/explore-org-og.jpg"),
        tags: &["live", "wildlife", "elephants", "africa"],
        episode_name: Some("Live view of the waterhole at Tembe Elephant Park, South Africa"),
    },
    HardcodedCam {
        id: "explore-kitten-rescue",
        name: "Kitten Rescue Sanctuary",
        url: "https://www.youtube.com/watch?v=mWnCd8erEXs",
        group: "Wildlife",
        logo: Some("https://explore.org/assets/explore-org-og.jpg"),
        tags: &["live", "wildlife", "cats", "rescue"],
        episode_name: Some("Rescued kittens playing at the Kitten Rescue sanctuary in Los Angeles"),
    },
    HardcodedCam {
        id: "explore-service-dog",
        name: "Service Dog Project - Great Dane Puppies",
        url: "https://www.youtube.com/watch?v=vJnhgRSFAsQ",
        group: "Wildlife",
        logo: Some("https://explore.org/assets/explore-org-og.jpg"),
        tags: &["live", "wildlife", "dogs", "puppies"],
        episode_name: Some("Great Dane puppies being raised as future service dogs"),
    },

    // === Space: NASA ISS ===
    HardcodedCam {
        id: "nasa-iss-live",
        name: "NASA ISS - Live Earth View",
        url: "https://ntv1.akamaized.net/hls/live/2014075/NASA-NTV1-HLS/master.m3u8",
        group: "Space",
        logo: Some("https://www.nasa.gov/wp-content/themes/flavor/images/nasa-logo.svg"),
        tags: &["live", "space", "iss"],
        episode_name: Some("Live video from the International Space Station as it orbits Earth"),
    },
    HardcodedCam {
        id: "nasa-iss-media",
        name: "NASA TV - Media Channel",
        url: "https://ntv2.akamaized.net/hls/live/2013923/NASA-NTV2-HLS/master.m3u8",
        group: "Space",
        logo: Some("https://www.nasa.gov/wp-content/themes/flavor/images/nasa-logo.svg"),
        tags: &["live", "space", "nasa"],
        episode_name: Some("NASA Television media channel with live coverage of missions and events"),
    },
];

/// Convert a HardcodedCam into a Stream.
fn hardcoded_to_stream(cam: &HardcodedCam) -> Stream {
    Stream {
        id: cam.id.to_string(),
        name: cam.name.to_string(),
        url: cam.url.to_string(),
        group: cam.group.to_string(),
        logo: cam.logo.map(|s| s.to_string()),
        vod_type: "live".to_string(),
        tags: if cam.tags.is_empty() {
            None
        } else {
            Some(cam.tags.iter().map(|s| s.to_string()).collect())
        },
        episode_name: cam.episode_name.map(|s| s.to_string()),
    }
}

/// Build the full list of hardcoded streams.
pub fn build_hardcoded_streams() -> Vec<Stream> {
    HARDCODED_CAMS.iter().map(hardcoded_to_stream).collect()
}

// ============================================================
// Explore.org API integration
// ============================================================

/// Map an explore.org category string to our group names.
fn map_explore_category(category: &str) -> &'static str {
    let lower = category.to_lowercase();
    if lower.contains("bird") || lower.contains("owl") || lower.contains("eagle")
        || lower.contains("osprey") || lower.contains("hawk") || lower.contains("heron")
        || lower.contains("hummingbird")
    {
        "Birds"
    } else if lower.contains("ocean") || lower.contains("reef") || lower.contains("aquarium")
        || lower.contains("fish") || lower.contains("shark") || lower.contains("coral")
        || lower.contains("underwater") || lower.contains("jellyfish") || lower.contains("sea")
    {
        "Aquarium"
    } else {
        "Wildlife"
    }
}

/// Parse the explore.org API response and return streams for active, online cams.
pub fn parse_explore_cams(body: &[u8]) -> Vec<Stream> {
    let value: Value = match serde_json::from_slice(body) {
        Ok(v) => v,
        Err(e) => {
            log_error(&format!("explore.org JSON parse error: {}", e));
            return Vec::new();
        }
    };

    // The API can return either a top-level array or an object with a key.
    let cams = match &value {
        Value::Array(arr) => arr.clone(),
        Value::Object(obj) => {
            if let Some(Value::Array(arr)) = obj.get("cams").or_else(|| obj.get("livecams")).or_else(|| obj.get("results")) {
                arr.clone()
            } else {
                log_error("explore.org response has no recognizable cam array");
                return Vec::new();
            }
        }
        _ => {
            log_error("explore.org response is not an array or object");
            return Vec::new();
        }
    };

    let mut streams = Vec::new();

    for cam in &cams {
        // Skip offline or inactive cams
        let is_offline = cam.get("is_offline").and_then(|v| v.as_bool()).unwrap_or(true);
        let active = cam.get("active").and_then(|v| v.as_bool()).unwrap_or(false);
        if is_offline || !active {
            continue;
        }

        let slug = cam.get("slug").and_then(|v| v.as_str()).unwrap_or("");
        let title = cam.get("title").and_then(|v| v.as_str()).unwrap_or("");
        if slug.is_empty() || title.is_empty() {
            continue;
        }

        // Determine the category/group
        let category = cam.get("category").and_then(|v| v.as_str()).unwrap_or("");
        let group = map_explore_category(if category.is_empty() { title } else { category });

        // Try to get a thumbnail
        let thumbnail = cam.get("thumbnail_large_url")
            .or_else(|| cam.get("stillframe"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        // Description
        let description = cam.get("description")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        // Build an explore.org web URL -- not a direct video stream, but functional
        let url = format!("https://explore.org/livecams/{}", slug);

        let id = format!("explore-{}", slug);

        streams.push(Stream {
            id,
            name: title.to_string(),
            url,
            group: group.to_string(),
            logo: thumbnail,
            vod_type: "live".to_string(),
            tags: Some(vec!["live".to_string(), "explore.org".to_string()]),
            episode_name: description,
        });
    }

    log_info(&format!("parsed {} active explore.org cams", streams.len()));
    streams
}

// ============================================================
// Plugin exports
// ============================================================

#[no_mangle]
pub extern "C" fn describe() -> u64 {
    let desc = Descriptor {
        r#type: "naturecams",
        label: "Nature Cams",
        short_label: "NATURE",
        color: "#4caf50",
        version: "1.0.0",
        description: "Live wildlife, aquarium, bird, and space cameras from around the world",
        config_fields: vec![],
        view: View {
            layout: "grid",
            group_by: "group",
            searchable: true,
            sortable: true,
        },
        interactions: vec![],
    };
    return_json(&desc)
}

#[no_mangle]
pub extern "C" fn refresh(config_ptr: u32, config_len: u32) -> u64 {
    let _ = read_input(config_ptr, config_len);

    // Start with hardcoded streams (always available)
    let streams = build_hardcoded_streams();
    log_info(&format!("loaded {} hardcoded streams", streams.len()));

    // Collect the set of hardcoded IDs to avoid duplicates from explore.org
    #[allow(unused_variables)]
    let hardcoded_ids: Vec<String> = streams.iter().map(|s| s.id.clone()).collect();

    // Need mutability for explore.org additions in non-test builds
    #[allow(unused_mut)]
    let mut streams = streams;

    // Optionally fetch explore.org live cams API
    #[cfg(not(test))]
    {
        match http_get("https://explore.org/api/livecams") {
            Some(body) => {
                let explore_streams = parse_explore_cams(&body);
                for s in explore_streams {
                    // Skip if we already have a hardcoded version
                    if !hardcoded_ids.contains(&s.id) {
                        streams.push(s);
                    }
                }
            }
            None => {
                log_error("failed to fetch explore.org API; using hardcoded cams only");
            }
        }
    }

    log_info(&format!("total streams: {}", streams.len()));
    return_json(&RefreshResponse { streams })
}

#[no_mangle]
pub extern "C" fn interact(action_ptr: u32, action_len: u32) -> u64 {
    let _ = read_input(action_ptr, action_len);
    let data = b"{}";
    let ptr = data.as_ptr() as u32;
    let len = data.len() as u32;
    pack_ptr_len(ptr, len)
}
