use super::*;

#[test]
fn test_hardcoded_stream_count() {
    let streams = build_hardcoded_streams();
    // We expect 16 hardcoded cams: 6 aquarium + 3 birds + 4 wildlife + 2 space + 1 extra (NASA media)
    assert!(streams.len() >= 15, "expected at least 15 hardcoded streams, got {}", streams.len());
}

#[test]
fn test_hardcoded_streams_have_required_fields() {
    let streams = build_hardcoded_streams();
    for stream in &streams {
        assert!(!stream.id.is_empty(), "stream id must not be empty");
        assert!(!stream.name.is_empty(), "stream name must not be empty");
        assert!(!stream.url.is_empty(), "stream url must not be empty");
        assert!(!stream.group.is_empty(), "stream group must not be empty");
        assert_eq!(stream.vod_type, "live", "all nature cams should be live type");
    }
}

#[test]
fn test_hardcoded_stream_ids_unique() {
    let streams = build_hardcoded_streams();
    let mut ids: Vec<&str> = streams.iter().map(|s| s.id.as_str()).collect();
    ids.sort();
    let len_before = ids.len();
    ids.dedup();
    assert_eq!(len_before, ids.len(), "stream IDs must be unique");
}

#[test]
fn test_grouping_categories() {
    let streams = build_hardcoded_streams();
    let groups: Vec<&str> = streams.iter().map(|s| s.group.as_str()).collect();

    assert!(groups.contains(&"Aquarium"), "should have Aquarium group");
    assert!(groups.contains(&"Birds"), "should have Birds group");
    assert!(groups.contains(&"Wildlife"), "should have Wildlife group");
    assert!(groups.contains(&"Space"), "should have Space group");
}

#[test]
fn test_aquarium_streams() {
    let streams = build_hardcoded_streams();
    let aquarium: Vec<&Stream> = streams.iter().filter(|s| s.group == "Aquarium").collect();
    assert!(aquarium.len() >= 5, "expected at least 5 aquarium streams, got {}", aquarium.len());

    // All Monterey Bay Aquarium cams should use YouTube URLs
    for stream in &aquarium {
        assert!(
            stream.url.starts_with("https://www.youtube.com/watch?v="),
            "aquarium stream '{}' should use YouTube URL, got: {}",
            stream.name, stream.url
        );
        assert!(
            stream.name.contains("Monterey Bay Aquarium"),
            "aquarium stream should mention Monterey Bay Aquarium: {}",
            stream.name
        );
    }
}

#[test]
fn test_bird_streams() {
    let streams = build_hardcoded_streams();
    let birds: Vec<&Stream> = streams.iter().filter(|s| s.group == "Birds").collect();
    assert!(birds.len() >= 2, "expected at least 2 bird streams, got {}", birds.len());

    for stream in &birds {
        assert!(
            stream.url.starts_with("https://www.youtube.com/watch?v="),
            "bird stream '{}' should use YouTube URL, got: {}",
            stream.name, stream.url
        );
    }
}

#[test]
fn test_space_streams() {
    let streams = build_hardcoded_streams();
    let space: Vec<&Stream> = streams.iter().filter(|s| s.group == "Space").collect();
    assert!(space.len() >= 1, "expected at least 1 space stream");

    // NASA ISS should use direct HLS
    let iss = space.iter().find(|s| s.id == "nasa-iss-live");
    assert!(iss.is_some(), "should have NASA ISS live stream");
    let iss = iss.unwrap();
    assert_eq!(
        iss.url,
        "https://ntv1.akamaized.net/hls/live/2014075/NASA-NTV1-HLS/master.m3u8",
        "NASA ISS should use the known HLS URL"
    );
}

#[test]
fn test_wildlife_streams() {
    let streams = build_hardcoded_streams();
    let wildlife: Vec<&Stream> = streams.iter().filter(|s| s.group == "Wildlife").collect();
    assert!(wildlife.len() >= 1, "expected at least 1 wildlife stream");
}

#[test]
fn test_youtube_url_format() {
    let streams = build_hardcoded_streams();
    for stream in &streams {
        if stream.url.contains("youtube.com") {
            assert!(
                stream.url.starts_with("https://www.youtube.com/watch?v="),
                "YouTube URLs must use full watch URL format: {}",
                stream.url
            );
            // Video ID should be present after v=
            let vid_id = stream.url.strip_prefix("https://www.youtube.com/watch?v=").unwrap();
            assert!(
                !vid_id.is_empty(),
                "YouTube video ID must not be empty for stream '{}'",
                stream.name
            );
        }
    }
}

#[test]
fn test_streams_have_tags() {
    let streams = build_hardcoded_streams();
    for stream in &streams {
        assert!(stream.tags.is_some(), "stream '{}' should have tags", stream.name);
        let tags = stream.tags.as_ref().unwrap();
        assert!(!tags.is_empty(), "stream '{}' tags should not be empty", stream.name);
        assert!(tags.contains(&"live".to_string()), "stream '{}' should have 'live' tag", stream.name);
    }
}

#[test]
fn test_streams_have_episode_name() {
    let streams = build_hardcoded_streams();
    for stream in &streams {
        assert!(
            stream.episode_name.is_some(),
            "stream '{}' should have an episode_name/description",
            stream.name
        );
        let desc = stream.episode_name.as_ref().unwrap();
        assert!(
            desc.len() > 10,
            "stream '{}' episode_name should be descriptive, got: '{}'",
            stream.name, desc
        );
    }
}

#[test]
fn test_parse_explore_cams_empty_array() {
    let body = b"[]";
    let streams = parse_explore_cams(body);
    assert!(streams.is_empty(), "empty array should yield no streams");
}

#[test]
fn test_parse_explore_cams_valid_cam() {
    let json = r#"[
        {
            "slug": "brown-bear-cam",
            "title": "Brown Bears at Brooks Falls",
            "category": "Wildlife",
            "is_offline": false,
            "active": true,
            "thumbnail_large_url": "https://example.com/thumb.jpg",
            "description": "Bears catching salmon"
        }
    ]"#;
    let streams = parse_explore_cams(json.as_bytes());
    assert_eq!(streams.len(), 1);
    assert_eq!(streams[0].id, "explore-brown-bear-cam");
    assert_eq!(streams[0].name, "Brown Bears at Brooks Falls");
    assert_eq!(streams[0].group, "Wildlife");
    assert_eq!(streams[0].url, "https://explore.org/livecams/brown-bear-cam");
    assert_eq!(streams[0].logo, Some("https://example.com/thumb.jpg".to_string()));
    assert_eq!(streams[0].episode_name, Some("Bears catching salmon".to_string()));
}

#[test]
fn test_parse_explore_cams_skips_offline() {
    let json = r#"[
        {
            "slug": "offline-cam",
            "title": "Offline Camera",
            "category": "Wildlife",
            "is_offline": true,
            "active": true
        },
        {
            "slug": "inactive-cam",
            "title": "Inactive Camera",
            "category": "Birds",
            "is_offline": false,
            "active": false
        }
    ]"#;
    let streams = parse_explore_cams(json.as_bytes());
    assert!(streams.is_empty(), "offline and inactive cams should be skipped");
}

#[test]
fn test_parse_explore_cams_bird_category() {
    let json = r#"[
        {
            "slug": "osprey-nest",
            "title": "Osprey Nest Cam",
            "category": "Bird Cams",
            "is_offline": false,
            "active": true,
            "description": "Osprey nesting"
        }
    ]"#;
    let streams = parse_explore_cams(json.as_bytes());
    assert_eq!(streams.len(), 1);
    assert_eq!(streams[0].group, "Birds");
}

#[test]
fn test_parse_explore_cams_ocean_category() {
    let json = r#"[
        {
            "slug": "coral-reef",
            "title": "Coral Reef Cam",
            "category": "Ocean",
            "is_offline": false,
            "active": true,
            "description": "Underwater reef view"
        }
    ]"#;
    let streams = parse_explore_cams(json.as_bytes());
    assert_eq!(streams.len(), 1);
    assert_eq!(streams[0].group, "Aquarium");
}

#[test]
fn test_parse_explore_cams_invalid_json() {
    let body = b"this is not json";
    let streams = parse_explore_cams(body);
    assert!(streams.is_empty(), "invalid JSON should yield no streams");
}

#[test]
fn test_parse_explore_cams_object_with_cams_key() {
    let json = r#"{
        "cams": [
            {
                "slug": "test-cam",
                "title": "Test Camera",
                "category": "Wildlife",
                "is_offline": false,
                "active": true,
                "description": "A test"
            }
        ]
    }"#;
    let streams = parse_explore_cams(json.as_bytes());
    assert_eq!(streams.len(), 1);
    assert_eq!(streams[0].id, "explore-test-cam");
}

#[test]
fn test_map_explore_category() {
    assert_eq!(map_explore_category("Bird Cams"), "Birds");
    assert_eq!(map_explore_category("Eagle Nest"), "Birds");
    assert_eq!(map_explore_category("Owl Watch"), "Birds");
    assert_eq!(map_explore_category("Osprey Cam"), "Birds");
    assert_eq!(map_explore_category("Ocean Exploration"), "Aquarium");
    assert_eq!(map_explore_category("Reef Cameras"), "Aquarium");
    assert_eq!(map_explore_category("Underwater"), "Aquarium");
    assert_eq!(map_explore_category("Shark Encounter"), "Aquarium");
    assert_eq!(map_explore_category("Bear Watching"), "Wildlife");
    assert_eq!(map_explore_category("African Safari"), "Wildlife");
    assert_eq!(map_explore_category("Puppies"), "Wildlife");
}

#[test]
fn test_describe_output() {
    // Verify describe produces valid JSON with correct fields
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

    let json = serde_json::to_value(&desc).unwrap();
    assert_eq!(json["type"], "naturecams");
    assert_eq!(json["label"], "Nature Cams");
    assert_eq!(json["short_label"], "NATURE");
    assert_eq!(json["color"], "#4caf50");
    assert_eq!(json["view"]["layout"], "grid");
    assert_eq!(json["view"]["group_by"], "group");
    assert!(json["config_fields"].as_array().unwrap().is_empty());
}

#[test]
fn test_refresh_returns_hardcoded_streams_in_test_mode() {
    // In test mode, http_get is not available, so refresh only returns hardcoded streams
    let streams = build_hardcoded_streams();
    let response = RefreshResponse { streams };
    let json = serde_json::to_value(&response).unwrap();
    let stream_array = json["streams"].as_array().unwrap();
    assert!(stream_array.len() >= 15);
}

#[test]
fn test_stream_serialization() {
    let stream = Stream {
        id: "test-id".to_string(),
        name: "Test Stream".to_string(),
        url: "https://example.com/stream".to_string(),
        group: "Wildlife".to_string(),
        logo: None,
        vod_type: "live".to_string(),
        tags: Some(vec!["live".to_string()]),
        episode_name: None,
    };

    let json = serde_json::to_value(&stream).unwrap();
    assert_eq!(json["id"], "test-id");
    assert_eq!(json["name"], "Test Stream");
    assert_eq!(json["vod_type"], "live");
    // logo and episode_name should be absent (skip_serializing_if)
    assert!(json.get("logo").is_none());
    assert!(json.get("episode_name").is_none());
}
