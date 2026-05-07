use super::*;

// ============================================================
// URL construction tests
// ============================================================

#[test]
fn test_channel_api_url_primary() {
    assert_eq!(
        channel_api_url("https://api.piped.private.coffee", "UC7_gcs09iThXybpVgjHZ_7g"),
        "https://api.piped.private.coffee/channel/UC7_gcs09iThXybpVgjHZ_7g"
    );
}

#[test]
fn test_channel_api_url_backup() {
    assert_eq!(
        channel_api_url("https://pipedapi.in.projectsegfau.lt", "UCHnyfMqiRRG1u-2MsSQLbXA"),
        "https://pipedapi.in.projectsegfau.lt/channel/UCHnyfMqiRRG1u-2MsSQLbXA"
    );
}

#[test]
fn test_video_url() {
    assert_eq!(
        video_url("dQw4w9WgXcQ"),
        "https://www.youtube.com/watch?v=dQw4w9WgXcQ"
    );
}

#[test]
fn test_thumbnail_url() {
    assert_eq!(
        thumbnail_url("dQw4w9WgXcQ"),
        "https://i.ytimg.com/vi/dQw4w9WgXcQ/hqdefault.jpg"
    );
}

// ============================================================
// Video ID extraction tests
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
fn test_extract_video_id_no_match() {
    assert_eq!(extract_video_id("/shorts/abc123"), None);
}

#[test]
fn test_extract_video_id_missing_prefix() {
    assert_eq!(extract_video_id(""), None);
}

// ============================================================
// Piped JSON parsing tests
// ============================================================

#[test]
fn test_parse_piped_response_full() {
    let json = r#"{
        "name": "Veritasium",
        "relatedStreams": [
            {
                "url": "/watch?v=dQw4w9WgXcQ",
                "title": "Never Gonna Give You Up",
                "thumbnail": "https://i.ytimg.com/vi/dQw4w9WgXcQ/hqdefault.jpg",
                "uploaderName": "Veritasium",
                "uploadedDate": "2 days ago",
                "duration": 1234,
                "views": 5000000,
                "isShort": false
            }
        ]
    }"#;

    let streams = parse_piped_response(json, "Veritasium");
    assert_eq!(streams.len(), 1);
    assert_eq!(streams[0].id, "yt-dQw4w9WgXcQ");
    assert_eq!(streams[0].name, "Never Gonna Give You Up");
    assert_eq!(streams[0].url, "https://www.youtube.com/watch?v=dQw4w9WgXcQ");
    assert_eq!(streams[0].group, "Veritasium");
    assert_eq!(
        streams[0].logo,
        Some("https://i.ytimg.com/vi/dQw4w9WgXcQ/hqdefault.jpg".to_string())
    );
    assert_eq!(streams[0].vod_type, "movie");
    assert_eq!(streams[0].tags, Some(vec!["youtube".to_string()]));
    assert_eq!(streams[0].episode_name, Some("2 days ago".to_string()));
}

#[test]
fn test_parse_piped_response_filters_shorts() {
    let json = r#"{
        "name": "TestChannel",
        "relatedStreams": [
            {
                "url": "/watch?v=regular1",
                "title": "Regular Video",
                "thumbnail": "https://example.com/thumb.jpg",
                "uploadedDate": "1 day ago",
                "isShort": false
            },
            {
                "url": "/watch?v=short1",
                "title": "Short Video",
                "thumbnail": "https://example.com/thumb2.jpg",
                "uploadedDate": "3 hours ago",
                "isShort": true
            },
            {
                "url": "/watch?v=regular2",
                "title": "Another Regular Video",
                "thumbnail": "https://example.com/thumb3.jpg",
                "uploadedDate": "5 days ago",
                "isShort": false
            }
        ]
    }"#;

    let streams = parse_piped_response(json, "TestChannel");
    assert_eq!(streams.len(), 2);
    assert_eq!(streams[0].id, "yt-regular1");
    assert_eq!(streams[1].id, "yt-regular2");
}

#[test]
fn test_parse_piped_response_uses_api_channel_name() {
    let json = r#"{
        "name": "API Channel Name",
        "relatedStreams": [
            {
                "url": "/watch?v=vid1",
                "title": "A Video",
                "isShort": false
            }
        ]
    }"#;

    let streams = parse_piped_response(json, "Fallback Name");
    assert_eq!(streams.len(), 1);
    assert_eq!(streams[0].group, "API Channel Name");
}

#[test]
fn test_parse_piped_response_fallback_channel_name() {
    let json = r#"{
        "relatedStreams": [
            {
                "url": "/watch?v=vid1",
                "title": "A Video",
                "isShort": false
            }
        ]
    }"#;

    let streams = parse_piped_response(json, "Fallback Name");
    assert_eq!(streams.len(), 1);
    assert_eq!(streams[0].group, "Fallback Name");
}

#[test]
fn test_parse_piped_response_empty_streams() {
    let json = r#"{
        "name": "Empty Channel",
        "relatedStreams": []
    }"#;

    let streams = parse_piped_response(json, "Empty");
    assert!(streams.is_empty());
}

#[test]
fn test_parse_piped_response_no_related_streams() {
    let json = r#"{"name": "Broken Channel"}"#;

    let streams = parse_piped_response(json, "Broken");
    assert!(streams.is_empty());
}

#[test]
fn test_parse_piped_response_invalid_json() {
    let streams = parse_piped_response("not json at all", "Channel");
    assert!(streams.is_empty());
}

#[test]
fn test_parse_piped_response_missing_url() {
    let json = r#"{
        "name": "TestChannel",
        "relatedStreams": [
            {
                "title": "No URL Video",
                "isShort": false
            }
        ]
    }"#;

    let streams = parse_piped_response(json, "TestChannel");
    assert!(streams.is_empty());
}

#[test]
fn test_parse_piped_response_fallback_thumbnail() {
    let json = r#"{
        "name": "TestChannel",
        "relatedStreams": [
            {
                "url": "/watch?v=xyz789",
                "title": "No Thumbnail Video",
                "isShort": false
            }
        ]
    }"#;

    let streams = parse_piped_response(json, "TestChannel");
    assert_eq!(streams.len(), 1);
    assert_eq!(
        streams[0].logo,
        Some("https://i.ytimg.com/vi/xyz789/hqdefault.jpg".to_string())
    );
}

#[test]
fn test_parse_piped_response_multiple_videos() {
    let json = r#"{
        "name": "Veritasium",
        "relatedStreams": [
            {
                "url": "/watch?v=vid001",
                "title": "Video One",
                "thumbnail": "https://i.ytimg.com/vi/vid001/hqdefault.jpg",
                "uploadedDate": "1 day ago",
                "isShort": false
            },
            {
                "url": "/watch?v=vid002",
                "title": "Video Two",
                "thumbnail": "https://i.ytimg.com/vi/vid002/hqdefault.jpg",
                "uploadedDate": "3 days ago",
                "isShort": false
            }
        ]
    }"#;

    let streams = parse_piped_response(json, "Veritasium");
    assert_eq!(streams.len(), 2);
    assert_eq!(streams[0].id, "yt-vid001");
    assert_eq!(streams[0].name, "Video One");
    assert_eq!(streams[0].group, "Veritasium");
    assert_eq!(streams[1].id, "yt-vid002");
    assert_eq!(streams[1].name, "Video Two");
}

#[test]
fn test_parse_piped_response_title_fallback_to_id() {
    let json = r#"{
        "name": "TestChannel",
        "relatedStreams": [
            {
                "url": "/watch?v=abc123",
                "isShort": false
            }
        ]
    }"#;

    let streams = parse_piped_response(json, "TestChannel");
    assert_eq!(streams.len(), 1);
    assert_eq!(streams[0].name, "abc123");
}

#[test]
fn test_parse_piped_response_no_episode_name_without_date() {
    let json = r#"{
        "name": "TestChannel",
        "relatedStreams": [
            {
                "url": "/watch?v=abc123",
                "title": "A Video",
                "isShort": false
            }
        ]
    }"#;

    let streams = parse_piped_response(json, "TestChannel");
    assert_eq!(streams.len(), 1);
    assert_eq!(streams[0].episode_name, None);
}

// ============================================================
// Channel definitions tests
// ============================================================

#[test]
fn test_channel_count() {
    assert_eq!(CHANNELS.len(), 14);
}

#[test]
fn test_channel_ids_unique() {
    let mut ids: Vec<&str> = CHANNELS.iter().map(|c| c.id).collect();
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), CHANNELS.len());
}

#[test]
fn test_channel_names_unique() {
    let mut names: Vec<&str> = CHANNELS.iter().map(|c| c.name).collect();
    names.sort();
    names.dedup();
    assert_eq!(names.len(), CHANNELS.len());
}

#[test]
fn test_known_channels_present() {
    let ids: Vec<&str> = CHANNELS.iter().map(|c| c.id).collect();
    assert!(ids.contains(&"UC7_gcs09iThXybpVgjHZ_7g")); // PBS Space Time
    assert!(ids.contains(&"UCHnyfMqiRRG1u-2MsSQLbXA")); // Veritasium
    assert!(ids.contains(&"UCYO_jab_esuFRV4b17AJtAw")); // 3Blue1Brown
    assert!(ids.contains(&"UCsXVk37bltHxD1rDPwtNM8Q")); // Kurzgesagt
}
