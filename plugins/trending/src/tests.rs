use super::*;

// ============================================================
// extract_video_id tests
// ============================================================

#[test]
fn test_extract_video_id_standard() {
    assert_eq!(
        extract_video_id("/watch?v=dQw4w9WgXcQ"),
        Some("dQw4w9WgXcQ".to_string())
    );
}

#[test]
fn test_extract_video_id_with_extra_params() {
    assert_eq!(
        extract_video_id("/watch?v=abc123&t=42"),
        Some("abc123".to_string())
    );
}

#[test]
fn test_extract_video_id_empty() {
    assert_eq!(extract_video_id("/watch?v="), None);
}

#[test]
fn test_extract_video_id_missing() {
    assert_eq!(extract_video_id("/watch?x=123"), None);
}

#[test]
fn test_extract_video_id_no_query() {
    assert_eq!(extract_video_id("/watch"), None);
}

// ============================================================
// truncate tests
// ============================================================

#[test]
fn test_truncate_short_string() {
    assert_eq!(truncate("hello", 10), "hello");
}

#[test]
fn test_truncate_exact_length() {
    assert_eq!(truncate("12345", 5), "12345");
}

#[test]
fn test_truncate_long_string() {
    assert_eq!(truncate("hello world", 5), "hello...");
}

#[test]
fn test_truncate_multibyte_utf8() {
    // Should not panic on multi-byte characters
    let s = "cafe\u{0301} au lait"; // "cafe\u{0301}" is e + combining accent
    let result = truncate(s, 6);
    assert!(result.ends_with("..."));
    // Ensure it doesn't panic and produces valid UTF-8
    assert!(result.is_char_boundary(0));
}

// ============================================================
// piped_video_to_stream tests
// ============================================================

fn sample_video() -> serde_json::Value {
    serde_json::json!({
        "url": "/watch?v=dQw4w9WgXcQ",
        "title": "Never Gonna Give You Up",
        "thumbnail": "https://proxy.piped.example/vi/dQw4w9WgXcQ/maxresdefault.jpg",
        "uploaderName": "Rick Astley",
        "uploaderAvatar": "https://proxy.piped.example/avatar.jpg",
        "uploaderVerified": true,
        "views": 1500000000,
        "duration": 212,
        "uploadedDate": "14 years ago",
        "uploaded": 1254268800000i64,
        "shortDescription": "The official video for Rick Astley's Never Gonna Give You Up",
        "isShort": false
    })
}

#[test]
fn test_video_to_stream_basic() {
    let video = sample_video();
    let stream = piped_video_to_stream(&video, "Trending").unwrap();

    assert_eq!(stream.id, "dQw4w9WgXcQ");
    assert_eq!(stream.name, "Never Gonna Give You Up");
    assert_eq!(stream.url, "https://www.youtube.com/watch?v=dQw4w9WgXcQ");
    assert_eq!(stream.group, "Trending");
    assert_eq!(stream.vod_type, "movie");
    assert_eq!(
        stream.logo,
        Some("https://proxy.piped.example/vi/dQw4w9WgXcQ/maxresdefault.jpg".to_string())
    );
    assert_eq!(stream.tags, Some(vec!["Rick Astley".to_string()]));
    assert_eq!(
        stream.episode_name,
        Some("The official video for Rick Astley's Never Gonna Give You Up".to_string())
    );
}

#[test]
fn test_video_to_stream_filters_shorts() {
    let video = serde_json::json!({
        "url": "/watch?v=short123",
        "title": "A Short",
        "isShort": true
    });
    assert!(piped_video_to_stream(&video, "Trending").is_none());
}

#[test]
fn test_video_to_stream_missing_url() {
    let video = serde_json::json!({
        "title": "No URL",
        "isShort": false
    });
    assert!(piped_video_to_stream(&video, "Trending").is_none());
}

#[test]
fn test_video_to_stream_no_uploader() {
    let video = serde_json::json!({
        "url": "/watch?v=xyz789",
        "title": "Mystery Video",
        "isShort": false
    });
    let stream = piped_video_to_stream(&video, "Trending Music").unwrap();
    assert_eq!(stream.id, "xyz789");
    assert_eq!(stream.group, "Trending Music");
    assert!(stream.tags.is_none());
    assert!(stream.episode_name.is_none());
    assert!(stream.logo.is_none());
}

#[test]
fn test_video_to_stream_truncates_long_description() {
    let long_desc = "A".repeat(300);
    let video = serde_json::json!({
        "url": "/watch?v=longdesc",
        "title": "Long Description Video",
        "shortDescription": long_desc,
        "isShort": false
    });
    let stream = piped_video_to_stream(&video, "Trending").unwrap();
    let ep = stream.episode_name.unwrap();
    assert!(ep.len() <= 203); // 200 + "..."
    assert!(ep.ends_with("..."));
}

// ============================================================
// Trending response parsing tests
// ============================================================

#[test]
fn test_parse_trending_response() {
    let response = serde_json::json!([
        {
            "url": "/watch?v=vid1",
            "title": "Video One",
            "thumbnail": "https://example.com/thumb1.jpg",
            "uploaderName": "Creator One",
            "shortDescription": "First video",
            "isShort": false
        },
        {
            "url": "/watch?v=vid2",
            "title": "Video Two",
            "thumbnail": "https://example.com/thumb2.jpg",
            "uploaderName": "Creator Two",
            "shortDescription": "Second video",
            "isShort": false
        },
        {
            "url": "/watch?v=short1",
            "title": "A Short",
            "isShort": true
        }
    ]);

    let videos: Vec<serde_json::Value> = serde_json::from_value(response).unwrap();
    let streams: Vec<Stream> = videos
        .iter()
        .filter_map(|v| piped_video_to_stream(v, "Trending"))
        .collect();

    // Shorts should be filtered out
    assert_eq!(streams.len(), 2);
    assert_eq!(streams[0].id, "vid1");
    assert_eq!(streams[0].name, "Video One");
    assert_eq!(streams[1].id, "vid2");
    assert_eq!(streams[1].name, "Video Two");
}

#[test]
fn test_parse_trending_empty_array() {
    let response = serde_json::json!([]);
    let videos: Vec<serde_json::Value> = serde_json::from_value(response).unwrap();
    let streams: Vec<Stream> = videos
        .iter()
        .filter_map(|v| piped_video_to_stream(v, "Trending"))
        .collect();
    assert!(streams.is_empty());
}

// ============================================================
// Search response parsing tests
// ============================================================

#[test]
fn test_parse_search_results_basic() {
    let body = serde_json::json!({
        "items": [
            {
                "url": "/watch?v=search1",
                "title": "Search Result 1",
                "thumbnail": "https://example.com/s1.jpg",
                "uploaderName": "Uploader",
                "shortDescription": "A search result",
                "isShort": false
            },
            {
                "url": "/watch?v=search2",
                "title": "Search Result 2",
                "isShort": false
            }
        ],
        "nextpage": "sometoken",
        "suggestion": null,
        "corrected": false
    });

    let body_bytes = serde_json::to_vec(&body).unwrap();
    let streams = parse_search_results(&body_bytes);

    assert_eq!(streams.len(), 2);
    assert_eq!(streams[0].id, "search1");
    assert_eq!(streams[0].name, "Search Result 1");
    assert_eq!(streams[0].group, "Search Results");
    assert_eq!(streams[0].url, "https://www.youtube.com/watch?v=search1");
    assert_eq!(streams[1].id, "search2");
}

#[test]
fn test_parse_search_results_filters_shorts() {
    let body = serde_json::json!({
        "items": [
            {
                "url": "/watch?v=norm1",
                "title": "Normal Video",
                "isShort": false
            },
            {
                "url": "/watch?v=short1",
                "title": "A Short",
                "isShort": true
            }
        ]
    });

    let body_bytes = serde_json::to_vec(&body).unwrap();
    let streams = parse_search_results(&body_bytes);

    assert_eq!(streams.len(), 1);
    assert_eq!(streams[0].id, "norm1");
}

#[test]
fn test_parse_search_results_empty() {
    let body = serde_json::json!({ "items": [] });
    let body_bytes = serde_json::to_vec(&body).unwrap();
    let streams = parse_search_results(&body_bytes);
    assert!(streams.is_empty());
}

#[test]
fn test_parse_search_results_invalid_json() {
    let streams = parse_search_results(b"not json");
    assert!(streams.is_empty());
}

#[test]
fn test_parse_search_results_missing_items() {
    let body = serde_json::json!({ "error": "something went wrong" });
    let body_bytes = serde_json::to_vec(&body).unwrap();
    let streams = parse_search_results(&body_bytes);
    assert!(streams.is_empty());
}

// ============================================================
// build_trending_url tests
// ============================================================

#[test]
fn test_build_trending_url_general() {
    let url = build_trending_url("https://pipedapi.kavin.rocks", "US", "");
    assert_eq!(url, "https://pipedapi.kavin.rocks/trending?region=US");
}

#[test]
fn test_build_trending_url_music() {
    let url = build_trending_url("https://pipedapi.kavin.rocks", "GB", "music");
    assert_eq!(
        url,
        "https://pipedapi.kavin.rocks/trending?region=GB&type=music"
    );
}

#[test]
fn test_build_trending_url_gaming() {
    let url = build_trending_url("https://api.piped.private.coffee", "JP", "gaming");
    assert_eq!(
        url,
        "https://api.piped.private.coffee/trending?region=JP&type=gaming"
    );
}

// ============================================================
// Stream serialization tests
// ============================================================

#[test]
fn test_stream_serialization_skips_none_fields() {
    let stream = Stream {
        id: "test".to_string(),
        name: "Test".to_string(),
        url: "https://www.youtube.com/watch?v=test".to_string(),
        group: "Trending".to_string(),
        logo: None,
        vod_type: "movie".to_string(),
        tags: None,
        episode_name: None,
    };

    let json = serde_json::to_value(&stream).unwrap();
    assert!(!json.as_object().unwrap().contains_key("logo"));
    assert!(!json.as_object().unwrap().contains_key("tags"));
    assert!(!json.as_object().unwrap().contains_key("episode_name"));
}

#[test]
fn test_stream_serialization_includes_some_fields() {
    let stream = Stream {
        id: "test".to_string(),
        name: "Test".to_string(),
        url: "https://www.youtube.com/watch?v=test".to_string(),
        group: "Trending".to_string(),
        logo: Some("https://example.com/thumb.jpg".to_string()),
        vod_type: "movie".to_string(),
        tags: Some(vec!["Creator".to_string()]),
        episode_name: Some("A description".to_string()),
    };

    let json = serde_json::to_value(&stream).unwrap();
    let obj = json.as_object().unwrap();
    assert_eq!(obj.get("logo").unwrap(), "https://example.com/thumb.jpg");
    assert_eq!(obj.get("tags").unwrap(), &serde_json::json!(["Creator"]));
    assert_eq!(obj.get("episode_name").unwrap(), "A description");
}
