use super::*;

// ============================================================
// Sample API responses for testing
// ============================================================

fn sample_boot_response() -> &'static str {
    r#"{
        "sessionToken": "eyJhbGciOiJIUzI1NiJ9.test-token.signature",
        "stitcherParams": "sessionID=abc123&sid=def456&deviceId=test-device&deviceDNT=0&deviceMake=Chrome&deviceModel=web&deviceType=web&deviceVersion=15.0&marketingRegion=GB&appName=web&appVersion=7.0.0",
        "servers": {
            "stitcher": "https://cfd-v4-service-channel-stitcher-use1-1.prd.pluto.tv"
        },
        "session": {
            "activeRegion": "GB"
        },
        "refreshInSec": 28800
    }"#
}

fn sample_boot_response_minimal() -> &'static str {
    r#"{
        "sessionToken": "minimal-token-xyz",
        "stitcherParams": "",
        "servers": {},
        "session": {}
    }"#
}

fn sample_boot_response_no_token() -> &'static str {
    r#"{
        "servers": {
            "stitcher": "https://example.com"
        },
        "session": {
            "activeRegion": "US"
        }
    }"#
}

fn sample_channels_response() -> &'static str {
    r#"[
        {
            "_id": "5dca4b2c18e5c40009f672ba",
            "name": "Pluto TV Movies",
            "slug": "pluto-tv-movies",
            "number": 2,
            "category": "Movies",
            "summary": "Watch free movies on Pluto TV",
            "isStitched": true,
            "stitched": {
                "urls": [{
                    "type": "hls",
                    "url": "https://old-stitcher.example.com/channel/5dca4b2c18e5c40009f672ba/master.m3u8"
                }]
            }
        },
        {
            "_id": "5dca4b6918e5c40009f672e0",
            "name": "Comedy Central",
            "slug": "comedy-central",
            "number": 68,
            "category": "Comedy",
            "summary": "The home of comedy",
            "isStitched": true,
            "stitched": {
                "urls": [{
                    "type": "hls",
                    "url": "https://old-stitcher.example.com/channel/5dca4b6918e5c40009f672e0/master.m3u8"
                }]
            }
        },
        {
            "_id": "5f4d31d882327b00078a8524",
            "name": "Pluto TV Crime",
            "slug": "pluto-tv-crime",
            "number": 45,
            "category": "Crime Drama",
            "summary": "True crime and crime drama",
            "isStitched": true,
            "stitched": {
                "urls": [{
                    "type": "hls",
                    "url": "https://old-stitcher.example.com/channel/5f4d31d882327b00078a8524/master.m3u8"
                }]
            }
        },
        {
            "_id": "abc123kidschannel",
            "name": "Nickelodeon",
            "slug": "nickelodeon",
            "number": 100,
            "category": "Kids",
            "summary": "Kids TV shows",
            "isStitched": true
        },
        {
            "_id": "5e8c7ae69ccd3200073c4e4f",
            "name": "Another Movie Channel",
            "slug": "another-movies",
            "number": 3,
            "category": "Movies",
            "summary": "More movies",
            "isStitched": true
        }
    ]"#
}

fn sample_channels_empty() -> &'static str {
    "[]"
}

fn sample_channels_with_missing_fields() -> &'static str {
    r#"[
        {
            "_id": "valid-channel-1",
            "name": "Valid Channel",
            "slug": "valid-channel",
            "number": 10,
            "category": "News",
            "isStitched": true
        },
        {
            "_id": "",
            "name": "No ID Channel",
            "slug": "no-id",
            "number": 11,
            "category": "News",
            "isStitched": true
        },
        {
            "_id": "no-name-channel",
            "name": "",
            "slug": "no-name",
            "number": 12,
            "category": "Sports",
            "isStitched": true
        },
        {
            "_id": "no-category-channel",
            "name": "No Category",
            "slug": "no-category",
            "number": 13,
            "isStitched": true
        },
        {
            "_id": "no-number-channel",
            "name": "No Number",
            "slug": "no-number",
            "category": "Music",
            "isStitched": true
        }
    ]"#
}

fn sample_channels_with_non_stitched() -> &'static str {
    r#"[
        {
            "_id": "stitched-channel",
            "name": "Stitched Channel",
            "slug": "stitched",
            "number": 1,
            "category": "Movies",
            "isStitched": true
        },
        {
            "_id": "not-stitched-channel",
            "name": "Not Stitched",
            "slug": "not-stitched",
            "number": 2,
            "category": "Movies",
            "isStitched": false
        },
        {
            "_id": "missing-stitched-field",
            "name": "Missing isStitched",
            "slug": "missing-field",
            "number": 3,
            "category": "Movies"
        }
    ]"#
}

fn sample_channels_with_system_channels() -> &'static str {
    r#"[
        {
            "_id": "normal-channel",
            "name": "Normal Channel",
            "slug": "normal-channel",
            "number": 1,
            "category": "Movies",
            "isStitched": true
        },
        {
            "_id": "announce-channel",
            "name": "Announcement",
            "slug": "announcement-main",
            "number": 999,
            "category": "System",
            "isStitched": true
        },
        {
            "_id": "privacy-channel",
            "name": "Privacy Policy",
            "slug": "privacy-policy-info",
            "number": 998,
            "category": "System",
            "isStitched": true
        }
    ]"#
}

fn sample_channels_with_duplicates() -> &'static str {
    r#"[
        {
            "_id": "dup-channel-1",
            "name": "Channel First",
            "slug": "channel-first",
            "number": 1,
            "category": "Movies",
            "isStitched": true
        },
        {
            "_id": "dup-channel-1",
            "name": "Channel Duplicate",
            "slug": "channel-dup",
            "number": 2,
            "category": "Comedy",
            "isStitched": true
        },
        {
            "_id": "unique-channel",
            "name": "Unique Channel",
            "slug": "unique",
            "number": 3,
            "category": "News",
            "isStitched": true
        }
    ]"#
}

// ============================================================
// Boot response parsing tests
// ============================================================

#[test]
fn test_parse_boot_response_full() {
    let data = parse_boot_response(sample_boot_response().as_bytes()).unwrap();
    assert_eq!(data.session_token, "eyJhbGciOiJIUzI1NiJ9.test-token.signature");
    assert_eq!(
        data.stitcher_base,
        "https://cfd-v4-service-channel-stitcher-use1-1.prd.pluto.tv"
    );
    assert!(data.stitcher_params.contains("sessionID=abc123"));
    assert!(data.stitcher_params.contains("marketingRegion=GB"));
    assert_eq!(data.active_region, "GB");
}

#[test]
fn test_parse_boot_response_minimal() {
    let data = parse_boot_response(sample_boot_response_minimal().as_bytes()).unwrap();
    assert_eq!(data.session_token, "minimal-token-xyz");
    // When servers.stitcher is missing, falls back to default
    assert_eq!(
        data.stitcher_base,
        "https://cfd-v4-service-channel-stitcher-use1-1.prd.pluto.tv"
    );
    assert_eq!(data.stitcher_params, "");
    assert_eq!(data.active_region, "unknown");
}

#[test]
fn test_parse_boot_response_no_token() {
    let result = parse_boot_response(sample_boot_response_no_token().as_bytes());
    assert!(result.is_none(), "should return None when sessionToken is missing");
}

#[test]
fn test_parse_boot_response_invalid_json() {
    let result = parse_boot_response(b"not valid json {{{");
    assert!(result.is_none(), "should return None for invalid JSON");
}

#[test]
fn test_parse_boot_response_empty_body() {
    let result = parse_boot_response(b"");
    assert!(result.is_none(), "should return None for empty body");
}

// ============================================================
// Channel parsing tests
// ============================================================

fn make_test_session() -> SessionData {
    SessionData {
        session_token: "test-token".to_string(),
        stitcher_base: "https://stitcher.example.com".to_string(),
        stitcher_params: "sessionID=test123&sid=abc".to_string(),
        active_region: "US".to_string(),
    }
}

#[test]
fn test_parse_channels_basic() {
    let session = make_test_session();
    let streams = parse_channels(sample_channels_response().as_bytes(), &session);

    assert_eq!(streams.len(), 5);

    // First channel
    assert_eq!(streams[0].id, "5dca4b2c18e5c40009f672ba");
    assert_eq!(streams[0].name, "Pluto TV Movies");
    assert_eq!(streams[0].group, "Movies");
    assert_eq!(
        streams[0].url,
        "https://stitcher.example.com/v2/stitch/hls/channel/5dca4b2c18e5c40009f672ba/master.m3u8?sessionID=test123&sid=abc&jwt=test-token&masterJWTPassthrough=true&includeExtendedEvents=true"
    );
    assert_eq!(
        streams[0].logo.as_deref(),
        Some("https://images.pluto.tv/channels/5dca4b2c18e5c40009f672ba/colorLogoPNG.png")
    );
    assert_eq!(streams[0].vod_type, "");
    assert_eq!(streams[0].tags, Some(vec!["2".to_string()]));
}

#[test]
fn test_parse_channels_url_construction_with_params() {
    let session = make_test_session();
    let streams = parse_channels(sample_channels_response().as_bytes(), &session);

    // All URLs should use the stitcher base with /v2/stitch/ path and JWT params
    for stream in &streams {
        assert!(
            stream.url.starts_with("https://stitcher.example.com/v2/stitch/hls/channel/"),
            "URL should use stitcher base with /v2/stitch/: {}",
            stream.url
        );
        assert!(
            stream.url.contains("&jwt=test-token&"),
            "URL should contain JWT token: {}",
            stream.url
        );
        assert!(
            stream.url.contains("masterJWTPassthrough=true"),
            "URL should contain masterJWTPassthrough: {}",
            stream.url
        );
        assert!(
            stream.url.contains("includeExtendedEvents=true"),
            "URL should contain includeExtendedEvents: {}",
            stream.url
        );
    }
}

#[test]
fn test_parse_channels_url_without_stitcher_params() {
    let session = SessionData {
        session_token: "my-jwt-token".to_string(),
        stitcher_base: "https://stitcher.example.com".to_string(),
        stitcher_params: String::new(),
        active_region: "US".to_string(),
    };
    let streams = parse_channels(sample_channels_response().as_bytes(), &session);

    // When stitcher_params is empty, URL should start with jwt= directly
    for stream in &streams {
        assert!(
            stream.url.contains("?jwt=my-jwt-token&masterJWTPassthrough=true"),
            "URL should have jwt as first param when stitcher_params empty: {}",
            stream.url
        );
    }
}

#[test]
fn test_parse_channels_empty_list() {
    let session = make_test_session();
    let streams = parse_channels(sample_channels_empty().as_bytes(), &session);
    assert!(streams.is_empty());
}

#[test]
fn test_parse_channels_invalid_json() {
    let session = make_test_session();
    let streams = parse_channels(b"not json at all", &session);
    assert!(streams.is_empty());
}

#[test]
fn test_parse_channels_missing_fields() {
    let session = make_test_session();
    let streams = parse_channels(sample_channels_with_missing_fields().as_bytes(), &session);

    // Only "Valid Channel", "No Category", and "No Number" should survive
    // (empty id and empty name are skipped)
    assert_eq!(streams.len(), 3);

    assert_eq!(streams[0].id, "valid-channel-1");
    assert_eq!(streams[0].name, "Valid Channel");
    assert_eq!(streams[0].group, "News");
    assert_eq!(streams[0].tags, Some(vec!["10".to_string()]));

    // Channel with no category gets "Uncategorized"
    assert_eq!(streams[1].id, "no-category-channel");
    assert_eq!(streams[1].group, "Uncategorized");

    // Channel with no number gets no tags
    assert_eq!(streams[2].id, "no-number-channel");
    assert_eq!(streams[2].name, "No Number");
    assert_eq!(streams[2].group, "Music");
    assert!(streams[2].tags.is_none());
}

// ============================================================
// isStitched filtering tests
// ============================================================

#[test]
fn test_non_stitched_channels_are_skipped() {
    let session = make_test_session();
    let streams = parse_channels(sample_channels_with_non_stitched().as_bytes(), &session);

    // Only the channel with isStitched: true should be included
    assert_eq!(streams.len(), 1);
    assert_eq!(streams[0].id, "stitched-channel");
    assert_eq!(streams[0].name, "Stitched Channel");
}

// ============================================================
// System channel filtering tests
// ============================================================

#[test]
fn test_system_channels_are_skipped() {
    let session = make_test_session();
    let streams = parse_channels(sample_channels_with_system_channels().as_bytes(), &session);

    // Only the normal channel should survive (announcement and privacy-policy slugs filtered)
    assert_eq!(streams.len(), 1);
    assert_eq!(streams[0].id, "normal-channel");
    assert_eq!(streams[0].name, "Normal Channel");
}

// ============================================================
// Deduplication tests
// ============================================================

#[test]
fn test_duplicate_channels_are_deduplicated() {
    let session = make_test_session();
    let streams = parse_channels(sample_channels_with_duplicates().as_bytes(), &session);

    // Should have 2 channels: the first occurrence of dup-channel-1 and unique-channel
    assert_eq!(streams.len(), 2);
    assert_eq!(streams[0].id, "dup-channel-1");
    assert_eq!(streams[0].name, "Channel First"); // First occurrence wins
    assert_eq!(streams[1].id, "unique-channel");
}

// ============================================================
// Category grouping tests
// ============================================================

#[test]
fn test_channels_grouped_by_category() {
    let session = make_test_session();
    let streams = parse_channels(sample_channels_response().as_bytes(), &session);

    // Collect groups
    let mut groups: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();
    for s in &streams {
        groups
            .entry(s.group.clone())
            .or_default()
            .push(s.name.clone());
    }

    // Movies category should have 2 channels
    assert_eq!(groups.get("Movies").map(|v| v.len()), Some(2));
    assert!(groups["Movies"].contains(&"Pluto TV Movies".to_string()));
    assert!(groups["Movies"].contains(&"Another Movie Channel".to_string()));

    // Comedy has 1
    assert_eq!(groups.get("Comedy").map(|v| v.len()), Some(1));
    assert!(groups["Comedy"].contains(&"Comedy Central".to_string()));

    // Crime Drama has 1
    assert_eq!(groups.get("Crime Drama").map(|v| v.len()), Some(1));

    // Kids has 1
    assert_eq!(groups.get("Kids").map(|v| v.len()), Some(1));
}

// ============================================================
// Session data serialization tests
// ============================================================

#[test]
fn test_session_data_roundtrip() {
    let session = SessionData {
        session_token: "my-token-123".to_string(),
        stitcher_base: "https://stitcher.pluto.tv".to_string(),
        stitcher_params: "sid=x&deviceId=y".to_string(),
        active_region: "US".to_string(),
    };

    let json = serde_json::to_string(&session).unwrap();
    let parsed: Value = serde_json::from_str(&json).unwrap();

    assert_eq!(
        parsed.get("session_token").unwrap().as_str().unwrap(),
        "my-token-123"
    );
    assert_eq!(
        parsed.get("stitcher_base").unwrap().as_str().unwrap(),
        "https://stitcher.pluto.tv"
    );
    assert_eq!(
        parsed.get("stitcher_params").unwrap().as_str().unwrap(),
        "sid=x&deviceId=y"
    );
    assert_eq!(
        parsed.get("active_region").unwrap().as_str().unwrap(),
        "US"
    );
}

// ============================================================
// Boot URL construction tests
// ============================================================

#[test]
fn test_build_boot_url() {
    let url = build_boot_url("device-123", "client-456");

    assert!(url.starts_with("https://boot.pluto.tv/v4/start?"));
    assert!(url.contains("appName=web"));
    assert!(url.contains("appVersion="));
    assert!(url.contains("clientID=client-456"));
    assert!(url.contains("deviceId=device-123"));
    assert!(url.contains("deviceMake="));
    assert!(url.contains("deviceModel=web"));
    assert!(url.contains("deviceType=web"));
    assert!(url.contains("deviceVersion="));
    assert!(url.contains("clientModelNumber="));
    assert!(url.contains("deviceDNT=0"));
}

// ============================================================
// Stream logo URL tests
// ============================================================

#[test]
fn test_channel_logo_urls() {
    let session = make_test_session();
    let streams = parse_channels(sample_channels_response().as_bytes(), &session);

    for stream in &streams {
        let expected_logo = format!(
            "https://images.pluto.tv/channels/{}/colorLogoPNG.png",
            stream.id
        );
        assert_eq!(stream.logo.as_deref(), Some(expected_logo.as_str()));
    }
}

// ============================================================
// Descriptor tests (test via serialization, not raw pointer)
// ============================================================

#[test]
fn test_descriptor_fields() {
    let desc = Descriptor {
        r#type: "plutotv",
        label: "Pluto TV",
        short_label: "PLUTO",
        color: "#00b4ff",
        version: "1.0.0",
        description: "Free live TV channels from Pluto TV",
        config_fields: vec![],
        view: View {
            layout: "grouped_list",
            group_by: "group",
            searchable: true,
            sortable: true,
        },
        interactions: vec![],
    };

    let json = serde_json::to_string(&desc).unwrap();
    let parsed: Value = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed.get("type").unwrap().as_str().unwrap(), "plutotv");
    assert_eq!(parsed.get("label").unwrap().as_str().unwrap(), "Pluto TV");
    assert_eq!(parsed.get("short_label").unwrap().as_str().unwrap(), "PLUTO");
    assert_eq!(parsed.get("color").unwrap().as_str().unwrap(), "#00b4ff");
    assert_eq!(parsed.get("version").unwrap().as_str().unwrap(), "1.0.0");

    let view = parsed.get("view").unwrap();
    assert_eq!(view.get("layout").unwrap().as_str().unwrap(), "grouped_list");
    assert_eq!(view.get("group_by").unwrap().as_str().unwrap(), "group");
    assert_eq!(view.get("searchable").unwrap().as_bool().unwrap(), true);
    assert_eq!(view.get("sortable").unwrap().as_bool().unwrap(), true);

    assert!(parsed.get("config_fields").unwrap().as_array().unwrap().is_empty());
    assert!(parsed.get("interactions").unwrap().as_array().unwrap().is_empty());
}

// ============================================================
// RefreshResponse serialization tests
// ============================================================

#[test]
fn test_refresh_response_serialization() {
    let response = RefreshResponse {
        streams: vec![
            Stream {
                id: "ch1".to_string(),
                name: "Channel One".to_string(),
                url: "https://example.com/ch1.m3u8".to_string(),
                group: "Movies".to_string(),
                logo: Some("https://example.com/logo1.png".to_string()),
                vod_type: String::new(),
                tags: Some(vec!["42".to_string()]),
            },
            Stream {
                id: "ch2".to_string(),
                name: "Channel Two".to_string(),
                url: "https://example.com/ch2.m3u8".to_string(),
                group: "News".to_string(),
                logo: None,
                vod_type: String::new(),
                tags: None,
            },
        ],
    };

    let json = serde_json::to_string(&response).unwrap();
    let parsed: Value = serde_json::from_str(&json).unwrap();

    let streams = parsed.get("streams").unwrap().as_array().unwrap();
    assert_eq!(streams.len(), 2);

    // First stream has logo and tags
    assert!(streams[0].get("logo").is_some());
    assert!(streams[0].get("tags").is_some());

    // Second stream should skip None fields
    assert!(streams[1].get("logo").is_none());
    assert!(streams[1].get("tags").is_none());
}

// ============================================================
// Pack/unpack helper tests
// ============================================================

#[test]
fn test_pack_unpack_roundtrip() {
    let ptr: u32 = 0x12345678;
    let len: u32 = 0x0000ABCD;
    let packed = pack_ptr_len(ptr, len);
    let (p, l) = unpack_ptr_len(packed);
    assert_eq!(p, ptr);
    assert_eq!(l, len);
}

#[test]
fn test_pack_unpack_zero() {
    let packed = pack_ptr_len(0, 0);
    let (p, l) = unpack_ptr_len(packed);
    assert_eq!(p, 0);
    assert_eq!(l, 0);
}

#[test]
fn test_pack_unpack_max_values() {
    let packed = pack_ptr_len(u32::MAX, u32::MAX);
    let (p, l) = unpack_ptr_len(packed);
    assert_eq!(p, u32::MAX);
    assert_eq!(l, u32::MAX);
}
