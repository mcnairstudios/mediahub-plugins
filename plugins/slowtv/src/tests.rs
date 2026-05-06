use super::*;
use std::collections::HashMap;

#[test]
fn test_build_streams_returns_all_videos() {
    let streams = build_streams();
    assert_eq!(streams.len(), CURATED_VIDEOS.len());
}

#[test]
fn test_all_urls_are_youtube_watch_urls() {
    let streams = build_streams();
    for stream in &streams {
        assert!(
            stream.url.starts_with("https://www.youtube.com/watch?v="),
            "Stream '{}' has invalid URL: {}",
            stream.name,
            stream.url
        );
    }
}

#[test]
fn test_all_ids_prefixed_with_slowtv() {
    let streams = build_streams();
    for stream in &streams {
        assert!(
            stream.id.starts_with("slowtv-"),
            "Stream id '{}' missing slowtv- prefix",
            stream.id
        );
    }
}

#[test]
fn test_all_ids_unique() {
    let streams = build_streams();
    let mut seen = std::collections::HashSet::new();
    for stream in &streams {
        assert!(
            seen.insert(stream.id.clone()),
            "Duplicate stream id: {}",
            stream.id
        );
    }
}

#[test]
fn test_all_streams_have_group() {
    let streams = build_streams();
    for stream in &streams {
        assert!(
            !stream.group.is_empty(),
            "Stream '{}' has empty group",
            stream.name
        );
    }
}

#[test]
fn test_grouping_categories_present() {
    let streams = build_streams();
    let groups: std::collections::HashSet<&str> = streams.iter().map(|s| s.group.as_str()).collect();

    let expected = [
        "Train Journeys",
        "Boat & Ship",
        "Fireplace & Cabin",
        "Nature",
        "Ocean",
        "City Walks",
    ];

    for cat in &expected {
        assert!(groups.contains(cat), "Missing expected group: {}", cat);
    }
}

#[test]
fn test_group_counts() {
    let streams = build_streams();
    let mut counts: HashMap<String, usize> = HashMap::new();
    for stream in &streams {
        *counts.entry(stream.group.clone()).or_insert(0) += 1;
    }

    // Each category should have at least 2 videos
    for (group, count) in &counts {
        assert!(
            *count >= 2,
            "Group '{}' only has {} video(s), expected at least 2",
            group,
            count
        );
    }
}

#[test]
fn test_all_streams_have_logo() {
    let streams = build_streams();
    for stream in &streams {
        assert!(
            stream.logo.is_some(),
            "Stream '{}' has no logo/thumbnail",
            stream.name
        );
        let logo = stream.logo.as_ref().unwrap();
        assert!(
            logo.starts_with("https://img.youtube.com/vi/"),
            "Stream '{}' has unexpected logo URL: {}",
            stream.name,
            logo
        );
    }
}

#[test]
fn test_all_streams_are_movie_vod_type() {
    let streams = build_streams();
    for stream in &streams {
        assert_eq!(
            stream.vod_type, "movie",
            "Stream '{}' has unexpected vod_type: {}",
            stream.name, stream.vod_type
        );
    }
}

#[test]
fn test_all_streams_tagged_slow_tv() {
    let streams = build_streams();
    for stream in &streams {
        let tags = stream.tags.as_ref().expect("Stream should have tags");
        assert!(
            tags.contains(&"slow-tv".to_string()),
            "Stream '{}' missing slow-tv tag",
            stream.name
        );
    }
}

#[test]
fn test_stream_name_includes_duration() {
    let streams = build_streams();
    for stream in &streams {
        // Each name should end with a parenthesized duration like "(10h)"
        assert!(
            stream.name.contains('(') && stream.name.ends_with(')'),
            "Stream name '{}' does not include duration in parentheses",
            stream.name
        );
    }
}

#[test]
fn test_youtube_url_helper() {
    assert_eq!(
        youtube_url("abc123"),
        "https://www.youtube.com/watch?v=abc123"
    );
}

#[test]
fn test_youtube_thumbnail_helper() {
    assert_eq!(
        youtube_thumbnail("abc123"),
        "https://img.youtube.com/vi/abc123/hqdefault.jpg"
    );
}

#[test]
fn test_pack_ptr_len_roundtrip() {
    let ptr: u32 = 0x1234;
    let len: u32 = 0x5678;
    let packed = pack_ptr_len(ptr, len);
    let (p, l) = unpack_ptr_len(packed);
    assert_eq!(p, ptr);
    assert_eq!(l, len);
}

#[test]
fn test_descriptor_serialization() {
    let desc = Descriptor {
        r#type: "slowtv",
        label: "Slow TV",
        short_label: "SLOW",
        color: "#2e7d32",
        version: "1.0.0",
        description: "Curated long-form ambient videos",
        config_fields: vec![],
        view: View {
            layout: "grouped_list",
            group_by: "group",
            searchable: true,
            sortable: true,
        },
        interactions: vec![],
    };

    let json = serde_json::to_value(&desc).unwrap();
    assert_eq!(json["type"], "slowtv");
    assert_eq!(json["label"], "Slow TV");
    assert_eq!(json["short_label"], "SLOW");
    assert_eq!(json["color"], "#2e7d32");
    assert_eq!(json["view"]["layout"], "grouped_list");
    assert_eq!(json["config_fields"].as_array().unwrap().len(), 0);
}

#[test]
fn test_refresh_response_serialization() {
    let resp = RefreshResponse {
        streams: build_streams(),
    };

    let json = serde_json::to_string(&resp).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    let streams = parsed["streams"].as_array().unwrap();
    assert_eq!(streams.len(), CURATED_VIDEOS.len());

    // Verify first stream has all expected fields
    let first = &streams[0];
    assert!(first.get("id").is_some());
    assert!(first.get("name").is_some());
    assert!(first.get("url").is_some());
    assert!(first.get("group").is_some());
    assert!(first.get("logo").is_some());
    assert!(first.get("vod_type").is_some());
}

#[test]
fn test_known_video_present() {
    let streams = build_streams();
    let arctic_winter = streams
        .iter()
        .find(|s| s.id == "slowtv-3rDjPLvOShM");
    assert!(
        arctic_winter.is_some(),
        "Expected Nordlandsbanen Winter video not found"
    );
    let stream = arctic_winter.unwrap();
    assert_eq!(stream.group, "Train Journeys");
    assert_eq!(
        stream.url,
        "https://www.youtube.com/watch?v=3rDjPLvOShM"
    );
}
