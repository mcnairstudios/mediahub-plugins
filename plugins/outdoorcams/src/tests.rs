use super::*;

#[test]
fn test_build_streams_returns_all_cams() {
    let streams = build_streams();
    assert_eq!(streams.len(), cam_count());
    assert!(streams.len() >= 28, "expected at least 28 streams, got {}", streams.len());
}

#[test]
fn test_all_urls_are_youtube_watch_links() {
    let streams = build_streams();
    for s in &streams {
        assert!(
            s.url.starts_with("https://www.youtube.com/watch?v="),
            "stream '{}' has invalid URL: {}",
            s.id,
            s.url
        );
        // Video ID should be non-empty (at least 1 char after ?v=)
        let vid_id = s.url.strip_prefix("https://www.youtube.com/watch?v=").unwrap();
        assert!(!vid_id.is_empty(), "stream '{}' has empty video ID", s.id);
    }
}

#[test]
fn test_groups_are_correct() {
    let groups = get_groups();
    assert_eq!(groups, vec!["Volcanoes", "Beach & Surf", "Ski Resorts"]);
}

#[test]
fn test_every_stream_belongs_to_a_valid_group() {
    let groups = get_groups();
    let streams = build_streams();
    for s in &streams {
        assert!(
            groups.contains(&s.group),
            "stream '{}' has unknown group '{}'",
            s.id,
            s.group
        );
    }
}

#[test]
fn test_stream_ids_are_unique() {
    let streams = build_streams();
    let mut ids: Vec<&str> = streams.iter().map(|s| s.id.as_str()).collect();
    ids.sort();
    for window in ids.windows(2) {
        assert_ne!(window[0], window[1], "duplicate stream id: {}", window[0]);
    }
}

#[test]
fn test_volcano_streams_count() {
    let streams = build_streams();
    let volcanoes: Vec<_> = streams.iter().filter(|s| s.group == "Volcanoes").collect();
    assert!(volcanoes.len() >= 10, "expected at least 10 volcano streams, got {}", volcanoes.len());
}

#[test]
fn test_beach_surf_streams_count() {
    let streams = build_streams();
    let beach: Vec<_> = streams.iter().filter(|s| s.group == "Beach & Surf").collect();
    assert!(beach.len() >= 10, "expected at least 10 beach/surf streams, got {}", beach.len());
}

#[test]
fn test_ski_streams_count() {
    let streams = build_streams();
    let ski: Vec<_> = streams.iter().filter(|s| s.group == "Ski Resorts").collect();
    assert!(ski.len() >= 8, "expected at least 8 ski streams, got {}", ski.len());
}

#[test]
fn test_all_streams_have_vod_type_live() {
    let streams = build_streams();
    for s in &streams {
        assert_eq!(s.vod_type, "live", "stream '{}' has wrong vod_type", s.id);
    }
}

#[test]
fn test_all_streams_have_tags() {
    let streams = build_streams();
    for s in &streams {
        assert!(s.tags.is_some(), "stream '{}' should have tags", s.id);
        let tags = s.tags.as_ref().unwrap();
        assert!(!tags.is_empty(), "stream '{}' should have at least one tag", s.id);
    }
}

#[test]
fn test_specific_stream_url_construction() {
    let streams = build_streams();
    let pipeline = streams.iter().find(|s| s.id == "surf-pipeline-explore").unwrap();
    assert_eq!(pipeline.url, "https://www.youtube.com/watch?v=DY5RYp4sxYc");
    assert_eq!(pipeline.group, "Beach & Surf");
    assert_eq!(pipeline.name, "Pipeline Cam, North Shore, Oahu");
}

#[test]
fn test_specific_volcano_stream() {
    let streams = build_streams();
    let kilauea = streams.iter().find(|s| s.id == "volcano-kilauea-multicam").unwrap();
    assert_eq!(kilauea.url, "https://www.youtube.com/watch?v=FVdmnpJ2kM0");
    assert_eq!(kilauea.group, "Volcanoes");
}

#[test]
fn test_specific_ski_stream() {
    let streams = build_streams();
    let grouse = streams.iter().find(|s| s.id == "ski-grouse-mountain").unwrap();
    assert_eq!(grouse.url, "https://www.youtube.com/watch?v=-XM7S9nm9js");
    assert_eq!(grouse.group, "Ski Resorts");
}

#[test]
fn test_stream_names_are_nonempty() {
    let streams = build_streams();
    for s in &streams {
        assert!(!s.name.is_empty(), "stream '{}' has empty name", s.id);
    }
}

#[test]
fn test_refresh_response_serialization() {
    let streams = build_streams();
    let resp = RefreshResponse { streams };
    let json = serde_json::to_string(&resp).expect("serialization should succeed");
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("should parse back");
    assert!(parsed["streams"].is_array());
    let arr = parsed["streams"].as_array().unwrap();
    assert_eq!(arr.len(), cam_count());

    // Check a sample entry has expected fields
    let first = &arr[0];
    assert!(first["id"].is_string());
    assert!(first["name"].is_string());
    assert!(first["url"].is_string());
    assert!(first["group"].is_string());
    assert!(first["vod_type"].is_string());
    assert!(first["tags"].is_array());
}

#[test]
fn test_pack_unpack_ptr_len() {
    let ptr: u32 = 0x12345678;
    let len: u32 = 0x00ABCDEF;
    let packed = pack_ptr_len(ptr, len);
    let (p, l) = unpack_ptr_len(packed);
    assert_eq!(p, ptr);
    assert_eq!(l, len);
}
