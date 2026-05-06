use super::*;

// ============================================================
// Sample catalog data for testing
// ============================================================

fn sample_catalog_json() -> &'static str {
    r#"[
        {
            "id": "2F72BC1E-3D76-456C-81EB-842EBA488C27",
            "accessibilityLabel": "Africa and the Middle East",
            "name": "Africa and the Middle East",
            "pointsOfInterest": {
                "0": "Over the Horn of Africa",
                "45": "The southeastern Arabian Peninsula",
                "115": "The Gulf of Oman"
            },
            "type": "space",
            "timeOfDay": "day",
            "src": {
                "H2641080p": "https://sylvan.apple.com/Videos/comp_A103_C002_SDR_2K_AVC.mov",
                "H2651080p": "https://sylvan.apple.com/Aerials/2x/Videos/comp_A103_C002_SDR_2K_HEVC.mov",
                "H2654k": "https://sylvan.apple.com/Aerials/2x/Videos/comp_A103_C002_SDR_4K_HEVC.mov"
            }
        },
        {
            "id": "A837FA8C-C643-4705-AE92-074E91E1A0C8",
            "accessibilityLabel": "Jellyfish",
            "name": "Jellyfish",
            "pointsOfInterest": {},
            "type": "underwater",
            "timeOfDay": "day",
            "src": {
                "H2641080p": "https://sylvan.apple.com/Videos/comp_jellies_SDR_2K_AVC.mov",
                "H2651080p": "https://sylvan.apple.com/Aerials/2x/Videos/comp_jellies_SDR_2K_HEVC.mov",
                "H2654k": "https://sylvan.apple.com/Aerials/2x/Videos/comp_jellies_SDR_4K_HEVC.mov"
            }
        },
        {
            "id": "3E94AE98-EAF2-4B09-96E3-452F46BC114E",
            "accessibilityLabel": "Hong Kong",
            "name": "Hong Kong",
            "pointsOfInterest": {
                "0": "Hong Kong Island from Victoria Harbour",
                "60": "Kowloon waterfront"
            },
            "type": "cityscape",
            "timeOfDay": "night",
            "src": {
                "H2641080p": "https://sylvan.apple.com/Videos/comp_HK004_SDR_2K_AVC.mov",
                "H2651080p": "https://sylvan.apple.com/Aerials/2x/Videos/comp_HK004_SDR_2K_HEVC.mov",
                "H2654k": "https://sylvan.apple.com/Aerials/2x/Videos/comp_HK004_SDR_4K_HEVC.mov"
            }
        },
        {
            "id": "89B1643B-06DD-4DEC-B1B0-774A38A47A27",
            "accessibilityLabel": "Greenland",
            "name": "Greenland",
            "pointsOfInterest": {
                "0": "Glacial fjord"
            },
            "type": "landscape",
            "timeOfDay": "day",
            "src": {
                "H2641080p": "https://sylvan.apple.com/Videos/comp_GL_SDR_2K_AVC.mov",
                "H2651080p": "https://sylvan.apple.com/Aerials/2x/Videos/comp_GL_SDR_2K_HEVC.mov",
                "H2654k": "https://sylvan.apple.com/Aerials/2x/Videos/comp_GL_SDR_4K_HEVC.mov"
            }
        },
        {
            "id": "CC70558D-3F3B-4B28-8B93-6E6C333D46FA",
            "accessibilityLabel": "Night lights over Europe",
            "name": "Night lights over Europe",
            "pointsOfInterest": {},
            "type": "space",
            "timeOfDay": "night",
            "src": {
                "H2641080p": "https://sylvan.apple.com/Videos/comp_EU_night_SDR_2K_AVC.mov",
                "H2651080p": "",
                "H2654k": ""
            }
        }
    ]"#
}

fn sample_entry_no_h264() -> &'static str {
    r#"[
        {
            "id": "AAAA-BBBB",
            "accessibilityLabel": "Fallback Test",
            "name": "Fallback Test",
            "pointsOfInterest": {},
            "type": "landscape",
            "timeOfDay": "",
            "src": {
                "H2641080p": "",
                "H2651080p": "https://sylvan.apple.com/Aerials/2x/Videos/fallback_SDR_2K_HEVC.mov",
                "H2654k": "https://sylvan.apple.com/Aerials/2x/Videos/fallback_SDR_4K_HEVC.mov"
            }
        }
    ]"#
}

fn sample_entry_no_urls() -> &'static str {
    r#"[
        {
            "id": "CCCC-DDDD",
            "accessibilityLabel": "",
            "name": "",
            "pointsOfInterest": {},
            "type": "",
            "src": {
                "H2641080p": "",
                "H2651080p": "",
                "H2654k": ""
            }
        }
    ]"#
}

// ============================================================
// Tests: catalog parsing
// ============================================================

#[test]
fn test_parse_catalog_valid() {
    let data = sample_catalog_json().as_bytes();
    let entries = parse_catalog(data).expect("should parse valid catalog");
    assert_eq!(entries.len(), 5);
}

#[test]
fn test_parse_catalog_empty_array() {
    let entries = parse_catalog(b"[]").expect("should parse empty array");
    assert!(entries.is_empty());
}

#[test]
fn test_parse_catalog_invalid_json() {
    let result = parse_catalog(b"not json at all");
    assert!(result.is_err());
}

#[test]
fn test_parse_catalog_entry_fields() {
    let entries = parse_catalog(sample_catalog_json().as_bytes()).unwrap();
    let first = &entries[0];

    assert_eq!(first.id, "2F72BC1E-3D76-456C-81EB-842EBA488C27");
    assert_eq!(first.accessibility_label, "Africa and the Middle East");
    assert_eq!(first.name, "Africa and the Middle East");
    assert_eq!(first.video_type, "space");
    assert_eq!(first.time_of_day, "day");
    assert_eq!(first.points_of_interest.len(), 3);
    assert_eq!(
        first.src.h264_1080p,
        "https://sylvan.apple.com/Videos/comp_A103_C002_SDR_2K_AVC.mov"
    );
}

// ============================================================
// Tests: URL extraction
// ============================================================

#[test]
fn test_h264_url_preferred() {
    let entries = parse_catalog(sample_catalog_json().as_bytes()).unwrap();
    let stream = video_to_stream(&entries[0]).expect("should produce a stream");
    assert!(
        stream.url.contains("AVC"),
        "expected H.264 AVC URL, got: {}",
        stream.url
    );
}

#[test]
fn test_fallback_to_h265_1080p_when_h264_empty() {
    let entries = parse_catalog(sample_entry_no_h264().as_bytes()).unwrap();
    let stream = video_to_stream(&entries[0]).expect("should produce a stream");
    assert!(
        stream.url.contains("HEVC") && stream.url.contains("2K"),
        "expected HEVC 1080p fallback URL, got: {}",
        stream.url
    );
}

#[test]
fn test_no_stream_when_all_urls_empty() {
    let entries = parse_catalog(sample_entry_no_urls().as_bytes()).unwrap();
    let stream = video_to_stream(&entries[0]);
    assert!(stream.is_none(), "should return None when no URLs available");
}

#[test]
fn test_all_urls_are_apple_cdn() {
    let entries = parse_catalog(sample_catalog_json().as_bytes()).unwrap();
    let streams = catalog_to_streams(&entries);
    for s in &streams {
        assert!(
            s.url.starts_with("https://sylvan.apple.com/"),
            "URL should be from Apple CDN, got: {}",
            s.url
        );
    }
}

// ============================================================
// Tests: grouping logic
// ============================================================

#[test]
fn test_group_name_mapping() {
    assert_eq!(group_name("space"), "Space");
    assert_eq!(group_name("underwater"), "Underwater");
    assert_eq!(group_name("landscape"), "Landscape");
    assert_eq!(group_name("cityscape"), "Cityscape");
    assert_eq!(group_name(""), "Other");
    assert_eq!(group_name("SPACE"), "Space");
    assert_eq!(group_name("Underwater"), "Underwater");
    assert_eq!(group_name("newtype"), "Newtype");
}

#[test]
fn test_streams_grouped_correctly() {
    let entries = parse_catalog(sample_catalog_json().as_bytes()).unwrap();
    let streams = catalog_to_streams(&entries);

    let groups: Vec<&str> = streams.iter().map(|s| s.group.as_str()).collect();

    // Entries: space, underwater, cityscape, landscape, space
    assert_eq!(groups[0], "Space");
    assert_eq!(groups[1], "Underwater");
    assert_eq!(groups[2], "Cityscape");
    assert_eq!(groups[3], "Landscape");
    assert_eq!(groups[4], "Space");
}

#[test]
fn test_time_of_day_tags() {
    let entries = parse_catalog(sample_catalog_json().as_bytes()).unwrap();
    let streams = catalog_to_streams(&entries);

    // First entry: day
    assert_eq!(streams[0].tags, Some(vec!["day".to_string()]));

    // Hong Kong: night
    let hk = streams.iter().find(|s| s.name == "Hong Kong").unwrap();
    assert_eq!(hk.tags, Some(vec!["night".to_string()]));
}

// ============================================================
// Tests: display name and description
// ============================================================

#[test]
fn test_display_name_prefers_accessibility_label() {
    let entries = parse_catalog(sample_catalog_json().as_bytes()).unwrap();
    let name = display_name(&entries[0]);
    assert_eq!(name, "Africa and the Middle East");
}

#[test]
fn test_display_name_fallback_to_name() {
    let entry = VideoEntry {
        id: "test-id".to_string(),
        name: "Test Name".to_string(),
        accessibility_label: String::new(),
        video_type: "space".to_string(),
        time_of_day: String::new(),
        points_of_interest: HashMap::new(),
        src: VideoSources::default(),
    };
    assert_eq!(display_name(&entry), "Test Name");
}

#[test]
fn test_display_name_fallback_to_id() {
    let entry = VideoEntry {
        id: "ABCDEF12-3456".to_string(),
        name: String::new(),
        accessibility_label: String::new(),
        video_type: "space".to_string(),
        time_of_day: String::new(),
        points_of_interest: HashMap::new(),
        src: VideoSources::default(),
    };
    assert_eq!(display_name(&entry), "Aerial ABCDEF12");
}

#[test]
fn test_build_description_with_pois() {
    let entries = parse_catalog(sample_catalog_json().as_bytes()).unwrap();
    let desc = build_description(&entries[0]).expect("should have description");
    // POIs sorted by timestamp key: 0, 45, 115
    assert!(desc.contains("Horn of Africa"));
    assert!(desc.contains("Arabian Peninsula"));
    assert!(desc.contains("Gulf of Oman"));
}

#[test]
fn test_build_description_empty_pois() {
    let entries = parse_catalog(sample_catalog_json().as_bytes()).unwrap();
    // Jellyfish has empty pointsOfInterest
    let desc = build_description(&entries[1]);
    assert!(desc.is_none());
}

#[test]
fn test_stream_episode_name_is_description() {
    let entries = parse_catalog(sample_catalog_json().as_bytes()).unwrap();
    let stream = video_to_stream(&entries[0]).unwrap();
    assert!(stream.episode_name.is_some());
    assert!(stream.episode_name.unwrap().contains("Horn of Africa"));
}

// ============================================================
// Tests: full catalog conversion
// ============================================================

#[test]
fn test_catalog_to_streams_count() {
    let entries = parse_catalog(sample_catalog_json().as_bytes()).unwrap();
    let streams = catalog_to_streams(&entries);
    // All 5 entries have at least one URL
    assert_eq!(streams.len(), 5);
}

#[test]
fn test_catalog_to_streams_skips_no_url_entries() {
    let entries = parse_catalog(sample_entry_no_urls().as_bytes()).unwrap();
    let streams = catalog_to_streams(&entries);
    assert!(streams.is_empty());
}

#[test]
fn test_all_streams_are_movie_type() {
    let entries = parse_catalog(sample_catalog_json().as_bytes()).unwrap();
    let streams = catalog_to_streams(&entries);
    for s in &streams {
        assert_eq!(s.vod_type, "movie");
    }
}

#[test]
fn test_stream_ids_match_entry_ids() {
    let entries = parse_catalog(sample_catalog_json().as_bytes()).unwrap();
    let streams = catalog_to_streams(&entries);
    for (entry, stream) in entries.iter().zip(streams.iter()) {
        assert_eq!(entry.id, stream.id);
    }
}

#[test]
fn test_no_stream_has_logo() {
    let entries = parse_catalog(sample_catalog_json().as_bytes()).unwrap();
    let streams = catalog_to_streams(&entries);
    for s in &streams {
        assert!(s.logo.is_none());
    }
}
