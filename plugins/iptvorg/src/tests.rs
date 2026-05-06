use super::*;

// ============================================================
// Sample JSON data for tests
// ============================================================

const SAMPLE_STREAMS_JSON: &str = r#"[
    {
        "channel": null,
        "feed": null,
        "title": "Bloomberg TV",
        "url": "https://example.com/bloomberg/stream.m3u8",
        "quality": "1080p",
        "label": null,
        "user_agent": null,
        "referrer": null
    },
    {
        "channel": null,
        "feed": null,
        "title": "Al Jazeera English",
        "url": "https://example.com/aljazeera/live.m3u8?token=abc",
        "quality": "720p",
        "label": "Geo-blocked",
        "user_agent": null,
        "referrer": null
    },
    {
        "channel": null,
        "feed": null,
        "title": "Some Radio Station",
        "url": "https://example.com/radio/stream.mp3",
        "quality": null,
        "label": null,
        "user_agent": null,
        "referrer": null
    },
    {
        "channel": null,
        "feed": null,
        "title": "",
        "url": "https://example.com/notitle/stream.m3u8",
        "quality": "480p",
        "label": null,
        "user_agent": null,
        "referrer": null
    },
    {
        "channel": null,
        "feed": null,
        "title": "No URL Channel",
        "url": "",
        "quality": "720p",
        "label": null,
        "user_agent": null,
        "referrer": null
    },
    {
        "channel": null,
        "feed": null,
        "title": "France 24",
        "url": "https://example.com/france24/index.m3u8#fragment",
        "quality": null,
        "label": "Not 24/7",
        "user_agent": "Mozilla/5.0",
        "referrer": "https://france24.com"
    },
    {
        "channel": "NullURL.test",
        "feed": null,
        "title": "Null URL Channel",
        "url": null,
        "quality": "1080p",
        "label": null,
        "user_agent": null,
        "referrer": null
    }
]"#;

// ============================================================
// JSON deserialization tests
// ============================================================

#[test]
fn test_deserialize_streams_json() {
    let streams: Vec<IptvStream> = serde_json::from_str(SAMPLE_STREAMS_JSON).unwrap();
    assert_eq!(streams.len(), 7);

    // First entry: Bloomberg TV with all fields populated
    assert_eq!(streams[0].title.as_deref(), Some("Bloomberg TV"));
    assert_eq!(
        streams[0].url.as_deref(),
        Some("https://example.com/bloomberg/stream.m3u8")
    );
    assert_eq!(streams[0].quality.as_deref(), Some("1080p"));
    assert!(streams[0].label.is_none());
    assert!(streams[0].channel.is_none());

    // Second entry: has both quality and label
    assert_eq!(streams[1].title.as_deref(), Some("Al Jazeera English"));
    assert_eq!(streams[1].quality.as_deref(), Some("720p"));
    assert_eq!(streams[1].label.as_deref(), Some("Geo-blocked"));

    // Sixth entry: has referrer and user_agent
    assert_eq!(streams[5].title.as_deref(), Some("France 24"));
    assert_eq!(streams[5].user_agent.as_deref(), Some("Mozilla/5.0"));
    assert_eq!(streams[5].referrer.as_deref(), Some("https://france24.com"));

    // Seventh entry: null URL
    assert!(streams[6].url.is_none());
    assert_eq!(streams[6].channel.as_deref(), Some("NullURL.test"));
}

#[test]
fn test_deserialize_minimal_entry() {
    let json = r#"[{"title": "Test", "url": "https://x.com/s.m3u8"}]"#;
    let streams: Vec<IptvStream> = serde_json::from_str(json).unwrap();
    assert_eq!(streams.len(), 1);
    assert_eq!(streams[0].title.as_deref(), Some("Test"));
    assert!(streams[0].channel.is_none());
    assert!(streams[0].quality.is_none());
    assert!(streams[0].label.is_none());
}

#[test]
fn test_deserialize_empty_array() {
    let streams: Vec<IptvStream> = serde_json::from_str("[]").unwrap();
    assert!(streams.is_empty());
}

// ============================================================
// HLS URL filtering tests
// ============================================================

#[test]
fn test_is_hls_url_basic() {
    assert!(is_hls_url("https://example.com/stream.m3u8"));
    assert!(is_hls_url("http://cdn.example.com/live/index.m3u8"));
}

#[test]
fn test_is_hls_url_with_query_string() {
    assert!(is_hls_url(
        "https://example.com/stream.m3u8?token=abc123"
    ));
    assert!(is_hls_url(
        "https://example.com/live/index.m3u8?key=val&other=1"
    ));
}

#[test]
fn test_is_hls_url_with_fragment() {
    assert!(is_hls_url("https://example.com/stream.m3u8#section"));
}

#[test]
fn test_is_hls_url_rejects_non_hls() {
    assert!(!is_hls_url("https://example.com/stream.mp3"));
    assert!(!is_hls_url("https://example.com/stream.mp4"));
    assert!(!is_hls_url("https://example.com/stream.ts"));
    assert!(!is_hls_url("https://example.com/stream.flv"));
    assert!(!is_hls_url("https://example.com/page.html"));
    assert!(!is_hls_url(""));
}

#[test]
fn test_is_hls_url_no_false_positive_on_m3u8_in_path() {
    // m3u8 appears in path but is not the extension
    assert!(!is_hls_url("https://example.com/m3u8/stream.mp4"));
}

// ============================================================
// Tag building tests
// ============================================================

#[test]
fn test_build_tags_both() {
    let tags = build_tags(&Some("1080p".to_string()), &Some("Geo-blocked".to_string()));
    assert_eq!(tags, vec!["1080p", "geo-blocked"]);
}

#[test]
fn test_build_tags_quality_only() {
    let tags = build_tags(&Some("720p".to_string()), &None);
    assert_eq!(tags, vec!["720p"]);
}

#[test]
fn test_build_tags_label_only() {
    let tags = build_tags(&None, &Some("Not 24/7".to_string()));
    assert_eq!(tags, vec!["not 24/7"]);
}

#[test]
fn test_build_tags_none() {
    let tags = build_tags(&None, &None);
    assert!(tags.is_empty());
}

#[test]
fn test_build_tags_empty_strings() {
    let tags = build_tags(&Some("".to_string()), &Some("  ".to_string()));
    assert!(tags.is_empty());
}

// ============================================================
// Stream ID generation tests
// ============================================================

#[test]
fn test_make_stream_id() {
    assert_eq!(make_stream_id(0), "iptv-0");
    assert_eq!(make_stream_id(42), "iptv-42");
    assert_eq!(make_stream_id(999), "iptv-999");
}

// ============================================================
// Stream processing (filter + map) tests
// ============================================================

#[test]
fn test_process_streams_filters_and_maps() {
    let raw: Vec<IptvStream> = serde_json::from_str(SAMPLE_STREAMS_JSON).unwrap();
    let streams = process_streams(&raw);

    // From the 7 entries:
    // 0: Bloomberg TV - .m3u8, has title -> KEEP
    // 1: Al Jazeera - .m3u8 (with query), has title -> KEEP
    // 2: Radio Station - .mp3 -> SKIP (not HLS)
    // 3: Empty title - .m3u8 -> SKIP (no title)
    // 4: No URL Channel - empty URL -> SKIP
    // 5: France 24 - .m3u8 (with fragment), has title -> KEEP
    // 6: Null URL Channel - null URL -> SKIP
    assert_eq!(streams.len(), 3);

    // Verify Bloomberg TV
    assert_eq!(streams[0].id, "iptv-0");
    assert_eq!(streams[0].name, "Bloomberg TV");
    assert_eq!(streams[0].url, "https://example.com/bloomberg/stream.m3u8");
    assert_eq!(streams[0].group, "Live TV");
    assert_eq!(streams[0].vod_type, "live");
    assert_eq!(streams[0].tags, vec!["1080p"]);
    assert_eq!(streams[0].logo, "");

    // Verify Al Jazeera English (quality + label tags)
    assert_eq!(streams[1].id, "iptv-1");
    assert_eq!(streams[1].name, "Al Jazeera English");
    assert_eq!(
        streams[1].url,
        "https://example.com/aljazeera/live.m3u8?token=abc"
    );
    assert_eq!(streams[1].tags, vec!["720p", "geo-blocked"]);

    // Verify France 24 (label tag only, no quality)
    assert_eq!(streams[2].id, "iptv-5");
    assert_eq!(streams[2].name, "France 24");
    assert_eq!(streams[2].tags, vec!["not 24/7"]);
}

#[test]
fn test_process_streams_empty_input() {
    let streams = process_streams(&[]);
    assert!(streams.is_empty());
}

#[test]
fn test_process_streams_all_filtered_out() {
    let raw = vec![
        IptvStream {
            channel: None,
            feed: None,
            title: Some("MP4 Stream".to_string()),
            url: Some("https://example.com/video.mp4".to_string()),
            quality: None,
            label: None,
            user_agent: None,
            referrer: None,
        },
        IptvStream {
            channel: None,
            feed: None,
            title: None,
            url: Some("https://example.com/stream.m3u8".to_string()),
            quality: None,
            label: None,
            user_agent: None,
            referrer: None,
        },
    ];
    let streams = process_streams(&raw);
    assert!(streams.is_empty());
}

#[test]
fn test_process_streams_preserves_all_hls() {
    let raw = vec![
        IptvStream {
            channel: None,
            feed: None,
            title: Some("Channel A".to_string()),
            url: Some("https://a.com/live.m3u8".to_string()),
            quality: Some("1080p".to_string()),
            label: None,
            user_agent: None,
            referrer: None,
        },
        IptvStream {
            channel: None,
            feed: None,
            title: Some("Channel B".to_string()),
            url: Some("https://b.com/live.m3u8".to_string()),
            quality: Some("720p".to_string()),
            label: None,
            user_agent: None,
            referrer: None,
        },
        IptvStream {
            channel: None,
            feed: None,
            title: Some("Channel C".to_string()),
            url: Some("https://c.com/live.m3u8".to_string()),
            quality: None,
            label: Some("Premium".to_string()),
            user_agent: None,
            referrer: None,
        },
    ];
    let streams = process_streams(&raw);
    assert_eq!(streams.len(), 3);
    assert_eq!(streams[0].name, "Channel A");
    assert_eq!(streams[1].name, "Channel B");
    assert_eq!(streams[2].name, "Channel C");
    assert_eq!(streams[2].tags, vec!["premium"]);
}

#[test]
fn test_process_streams_group_is_always_live_tv() {
    let raw = vec![IptvStream {
        channel: None,
        feed: None,
        title: Some("Test".to_string()),
        url: Some("https://x.com/s.m3u8".to_string()),
        quality: None,
        label: None,
        user_agent: None,
        referrer: None,
    }];
    let streams = process_streams(&raw);
    assert_eq!(streams.len(), 1);
    assert_eq!(streams[0].group, "Live TV");
}

// ============================================================
// Output serialization tests
// ============================================================

#[test]
fn test_stream_serialization() {
    let stream = Stream {
        id: "iptv-0".to_string(),
        name: "Test Channel".to_string(),
        url: "https://example.com/live.m3u8".to_string(),
        group: "Live TV".to_string(),
        logo: "".to_string(),
        vod_type: "live".to_string(),
        tags: vec!["1080p".to_string()],
    };

    let json = serde_json::to_value(&stream).unwrap();
    assert_eq!(json["id"], "iptv-0");
    assert_eq!(json["name"], "Test Channel");
    assert_eq!(json["url"], "https://example.com/live.m3u8");
    assert_eq!(json["group"], "Live TV");
    assert_eq!(json["logo"], "");
    assert_eq!(json["vod_type"], "live");
    assert_eq!(json["tags"], serde_json::json!(["1080p"]));
}

#[test]
fn test_stream_roundtrip() {
    let stream = Stream {
        id: "iptv-5".to_string(),
        name: "France 24".to_string(),
        url: "https://example.com/france24.m3u8".to_string(),
        group: "Live TV".to_string(),
        logo: "".to_string(),
        vod_type: "live".to_string(),
        tags: vec!["720p".to_string(), "not 24/7".to_string()],
    };

    let serialized = serde_json::to_string(&stream).unwrap();
    let deserialized: Stream = serde_json::from_str(&serialized).unwrap();
    assert_eq!(stream, deserialized);
}
