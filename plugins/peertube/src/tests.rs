use super::*;

// ============================================================
// Test: parse_instance_from_url
// ============================================================

#[test]
fn test_parse_instance_from_url_standard() {
    let url = "https://tube.example.org/videos/watch/abcd-1234-efgh";
    assert_eq!(
        parse_instance_from_url(url),
        Some("tube.example.org".to_string())
    );
}

#[test]
fn test_parse_instance_from_url_with_port() {
    // Some self-hosted instances run on non-standard ports
    let url = "https://peertube.local:9000/videos/watch/some-uuid";
    assert_eq!(
        parse_instance_from_url(url),
        Some("peertube.local:9000".to_string())
    );
}

#[test]
fn test_parse_instance_from_url_http() {
    let url = "http://videos.example.com/w/some-uuid";
    assert_eq!(
        parse_instance_from_url(url),
        Some("videos.example.com".to_string())
    );
}

#[test]
fn test_parse_instance_from_url_no_scheme() {
    let url = "not-a-url";
    assert_eq!(parse_instance_from_url(url), None);
}

#[test]
fn test_parse_instance_from_url_empty() {
    assert_eq!(parse_instance_from_url(""), None);
}

#[test]
fn test_parse_instance_from_url_just_scheme() {
    assert_eq!(parse_instance_from_url("https://"), None);
}

// ============================================================
// Test: format_duration
// ============================================================

#[test]
fn test_format_duration_hours() {
    assert_eq!(format_duration(3723), "1h 02m");
}

#[test]
fn test_format_duration_minutes() {
    assert_eq!(format_duration(330), "5m 30s");
}

#[test]
fn test_format_duration_seconds_only() {
    assert_eq!(format_duration(45), "45s");
}

#[test]
fn test_format_duration_zero() {
    assert_eq!(format_duration(0), "");
}

#[test]
fn test_format_duration_negative() {
    assert_eq!(format_duration(-5), "");
}

#[test]
fn test_format_duration_exactly_one_hour() {
    assert_eq!(format_duration(3600), "1h 00m");
}

#[test]
fn test_format_duration_exactly_one_minute() {
    assert_eq!(format_duration(60), "1m 00s");
}

// ============================================================
// Test: url_encode
// ============================================================

#[test]
fn test_url_encode_simple() {
    assert_eq!(url_encode("hello"), "hello");
}

#[test]
fn test_url_encode_spaces() {
    assert_eq!(url_encode("hello world"), "hello%20world");
}

#[test]
fn test_url_encode_special_chars() {
    assert_eq!(url_encode("a+b&c=d"), "a%2Bb%26c%3Dd");
}

#[test]
fn test_url_encode_preserves_safe_chars() {
    assert_eq!(url_encode("a-b_c.d~e"), "a-b_c.d~e");
}

// ============================================================
// Test: parse_search_response
// ============================================================

#[test]
fn test_parse_search_response_basic() {
    let json = r#"{
        "total": 2,
        "data": [
            {
                "uuid": "aaaa-bbbb",
                "name": "Test Video 1",
                "url": "https://tube.example.org/videos/watch/aaaa-bbbb",
                "description": "A test video",
                "duration": 120,
                "thumbnailUrl": "https://tube.example.org/static/thumbnails/thumb1.jpg",
                "nsfw": false,
                "category": {"label": "Science & Technology"},
                "channel": {"displayName": "TestChannel"}
            },
            {
                "uuid": "cccc-dddd",
                "name": "Test Video 2",
                "url": "https://other.instance.com/videos/watch/cccc-dddd",
                "description": "",
                "duration": 3600,
                "thumbnailUrl": "",
                "nsfw": false,
                "category": null,
                "channel": null
            }
        ]
    }"#;

    let resp = parse_search_response(json.as_bytes()).unwrap();
    assert_eq!(resp.total, 2);
    assert_eq!(resp.data.len(), 2);

    assert_eq!(resp.data[0].uuid, "aaaa-bbbb");
    assert_eq!(resp.data[0].name, "Test Video 1");
    assert_eq!(resp.data[0].duration, 120);
    assert!(!resp.data[0].nsfw);
    assert_eq!(
        resp.data[0].category.as_ref().unwrap().label,
        "Science & Technology"
    );
    assert_eq!(
        resp.data[0].channel.as_ref().unwrap().displayName,
        "TestChannel"
    );

    assert_eq!(resp.data[1].uuid, "cccc-dddd");
    assert!(resp.data[1].category.is_none());
    assert!(resp.data[1].channel.is_none());
}

#[test]
fn test_parse_search_response_empty() {
    let json = r#"{"total": 0, "data": []}"#;
    let resp = parse_search_response(json.as_bytes()).unwrap();
    assert_eq!(resp.total, 0);
    assert_eq!(resp.data.len(), 0);
}

#[test]
fn test_parse_search_response_nsfw_flagged() {
    let json = r#"{
        "total": 1,
        "data": [
            {
                "uuid": "nsfw-uuid",
                "name": "NSFW Video",
                "url": "https://tube.example.org/videos/watch/nsfw-uuid",
                "nsfw": true
            }
        ]
    }"#;

    let resp = parse_search_response(json.as_bytes()).unwrap();
    assert!(resp.data[0].nsfw);
}

#[test]
fn test_parse_search_response_invalid_json() {
    let json = b"not json at all";
    assert!(parse_search_response(json).is_none());
}

// ============================================================
// Test: parse_video_detail and extract_playable_url
// ============================================================

#[test]
fn test_parse_video_detail_with_hls() {
    let json = r#"{
        "uuid": "aaaa-bbbb",
        "files": [
            {
                "fileUrl": "https://tube.example.org/static/webseed/video-480.mp4",
                "fileDownloadUrl": "",
                "resolution": {"id": 480, "label": "480p"}
            }
        ],
        "streamingPlaylists": [
            {
                "playlistUrl": "https://tube.example.org/static/streaming-playlists/hls/master.m3u8",
                "files": []
            }
        ]
    }"#;

    let detail = parse_video_detail(json.as_bytes()).unwrap();
    let url = extract_playable_url(&detail).unwrap();
    assert_eq!(
        url,
        "https://tube.example.org/static/streaming-playlists/hls/master.m3u8"
    );
}

#[test]
fn test_parse_video_detail_fallback_to_mp4() {
    let json = r#"{
        "uuid": "aaaa-bbbb",
        "files": [
            {
                "fileUrl": "https://tube.example.org/static/webseed/video-360.mp4",
                "fileDownloadUrl": "",
                "resolution": {"id": 360, "label": "360p"}
            },
            {
                "fileUrl": "https://tube.example.org/static/webseed/video-1080.mp4",
                "fileDownloadUrl": "",
                "resolution": {"id": 1080, "label": "1080p"}
            },
            {
                "fileUrl": "https://tube.example.org/static/webseed/video-480.mp4",
                "fileDownloadUrl": "",
                "resolution": {"id": 480, "label": "480p"}
            }
        ],
        "streamingPlaylists": []
    }"#;

    let detail = parse_video_detail(json.as_bytes()).unwrap();
    let url = extract_playable_url(&detail).unwrap();
    // Should pick the 1080p file (highest resolution)
    assert_eq!(
        url,
        "https://tube.example.org/static/webseed/video-1080.mp4"
    );
}

#[test]
fn test_parse_video_detail_no_files() {
    let json = r#"{
        "uuid": "aaaa-bbbb",
        "files": [],
        "streamingPlaylists": []
    }"#;

    let detail = parse_video_detail(json.as_bytes()).unwrap();
    assert!(extract_playable_url(&detail).is_none());
}

#[test]
fn test_parse_video_detail_uses_download_url_fallback() {
    let json = r#"{
        "uuid": "aaaa-bbbb",
        "files": [
            {
                "fileUrl": "",
                "fileDownloadUrl": "https://tube.example.org/download/video-720.mp4",
                "resolution": {"id": 720, "label": "720p"}
            }
        ],
        "streamingPlaylists": []
    }"#;

    let detail = parse_video_detail(json.as_bytes()).unwrap();
    let url = extract_playable_url(&detail).unwrap();
    assert_eq!(
        url,
        "https://tube.example.org/download/video-720.mp4"
    );
}

#[test]
fn test_parse_video_detail_invalid_json() {
    assert!(parse_video_detail(b"garbage").is_none());
}

// ============================================================
// Test: sepia_video_to_stream
// ============================================================

#[test]
fn test_sepia_video_to_stream_with_category() {
    let video = SepiaVideo {
        uuid: "test-uuid-1234".to_string(),
        name: "Intro to Rust".to_string(),
        url: "https://tube.example.org/videos/watch/test-uuid-1234".to_string(),
        description: "A great tutorial".to_string(),
        duration: 600,
        thumbnailUrl: "https://tube.example.org/thumb.jpg".to_string(),
        nsfw: false,
        category: Some(SepiaCategory {
            label: "Education".to_string(),
        }),
        channel: Some(SepiaChannel {
            displayName: "RustChan".to_string(),
        }),
    };

    let stream = sepia_video_to_stream(&video, "https://tube.example.org/stream/master.m3u8");
    assert_eq!(stream.id, "test-uuid-1234");
    assert_eq!(stream.name, "Intro to Rust");
    assert_eq!(stream.url, "https://tube.example.org/stream/master.m3u8");
    assert_eq!(stream.group, "Education");
    assert_eq!(stream.vod_type, "movie");
    assert_eq!(stream.logo, Some("https://tube.example.org/thumb.jpg".to_string()));
    assert_eq!(stream.tags, Some(vec!["RustChan".to_string()]));
    assert_eq!(stream.description, Some("A great tutorial".to_string()));
    assert_eq!(stream.duration, Some("10m 00s".to_string()));
}

#[test]
fn test_sepia_video_to_stream_no_category_uses_instance() {
    let video = SepiaVideo {
        uuid: "uuid-5678".to_string(),
        name: "Some Video".to_string(),
        url: "https://framatube.org/videos/watch/uuid-5678".to_string(),
        description: "".to_string(),
        duration: 0,
        thumbnailUrl: "".to_string(),
        nsfw: false,
        category: None,
        channel: None,
    };

    let stream = sepia_video_to_stream(&video, "https://framatube.org/watch.mp4");
    assert_eq!(stream.group, "framatube.org");
    assert!(stream.logo.is_none());
    assert!(stream.tags.is_none());
    assert!(stream.description.is_none());
    assert!(stream.duration.is_none());
}

#[test]
fn test_sepia_video_to_stream_relative_thumbnail() {
    let video = SepiaVideo {
        uuid: "uuid-rel".to_string(),
        name: "Relative Thumb".to_string(),
        url: "https://tube.example.org/videos/watch/uuid-rel".to_string(),
        description: "".to_string(),
        duration: 30,
        thumbnailUrl: "/static/thumbnails/thumb.jpg".to_string(),
        nsfw: false,
        category: None,
        channel: None,
    };

    let stream = sepia_video_to_stream(&video, "https://tube.example.org/file.mp4");
    assert_eq!(
        stream.logo,
        Some("https://tube.example.org/static/thumbnails/thumb.jpg".to_string())
    );
}

#[test]
fn test_sepia_video_to_stream_long_description_truncated() {
    let long_desc = "A".repeat(300);
    let video = SepiaVideo {
        uuid: "uuid-long".to_string(),
        name: "Long Desc".to_string(),
        url: "https://tube.example.org/videos/watch/uuid-long".to_string(),
        description: long_desc,
        duration: 60,
        thumbnailUrl: "".to_string(),
        nsfw: false,
        category: None,
        channel: None,
    };

    let stream = sepia_video_to_stream(&video, "https://tube.example.org/file.mp4");
    let desc = stream.description.unwrap();
    assert!(desc.len() <= 203); // 197 chars + "..."
    assert!(desc.ends_with("..."));
}

// ============================================================
// Test: extract_playable_url prefers HLS over MP4
// ============================================================

#[test]
fn test_extract_playable_url_hls_preferred_over_high_res_mp4() {
    let detail = VideoDetail {
        uuid: "test".to_string(),
        files: vec![VideoFile {
            fileUrl: "https://example.org/video-2160.mp4".to_string(),
            fileDownloadUrl: "".to_string(),
            resolution: Some(VideoResolution {
                id: 2160,
                label: "2160p".to_string(),
            }),
        }],
        streamingPlaylists: vec![StreamingPlaylist {
            playlistUrl: "https://example.org/hls/master.m3u8".to_string(),
            files: vec![],
        }],
    };

    let url = extract_playable_url(&detail).unwrap();
    assert_eq!(url, "https://example.org/hls/master.m3u8");
}

// ============================================================
// Test: multiple streaming playlists picks first non-empty
// ============================================================

#[test]
fn test_extract_playable_url_skips_empty_playlist_url() {
    let detail = VideoDetail {
        uuid: "test".to_string(),
        files: vec![],
        streamingPlaylists: vec![
            StreamingPlaylist {
                playlistUrl: "".to_string(),
                files: vec![],
            },
            StreamingPlaylist {
                playlistUrl: "https://example.org/hls/master.m3u8".to_string(),
                files: vec![],
            },
        ],
    };

    let url = extract_playable_url(&detail).unwrap();
    assert_eq!(url, "https://example.org/hls/master.m3u8");
}
