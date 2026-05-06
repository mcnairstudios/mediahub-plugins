use super::*;
use serde_json::json;

// ============================================================
// Tests for find_audio_url
// ============================================================

#[test]
fn test_find_audio_url_mp3_preferred() {
    let files = vec![
        json!({
            "file_name": "track.flac",
            "file_format_info": { "mime_type": "audio/x-flac" },
            "download_url": "https://ccmixter.org/content/user/track.flac"
        }),
        json!({
            "file_name": "track.mp3",
            "file_format_info": { "mime_type": "audio/mpeg" },
            "download_url": "https://ccmixter.org/content/user/track.mp3"
        }),
    ];
    let url = find_audio_url(&files);
    assert_eq!(url, Some("https://ccmixter.org/content/user/track.mp3".to_string()));
}

#[test]
fn test_find_audio_url_fallback_to_flac() {
    let files = vec![
        json!({
            "file_name": "track.flac",
            "file_format_info": { "mime_type": "audio/x-flac" },
            "download_url": "https://ccmixter.org/content/user/track.flac"
        }),
        json!({
            "file_name": "stems.zip",
            "file_format_info": { "mime_type": "application/zip" },
            "download_url": "https://ccmixter.org/content/user/stems.zip"
        }),
    ];
    let url = find_audio_url(&files);
    assert_eq!(url, Some("https://ccmixter.org/content/user/track.flac".to_string()));
}

#[test]
fn test_find_audio_url_no_audio_files() {
    let files = vec![
        json!({
            "file_name": "stems.zip",
            "file_format_info": { "mime_type": "application/zip" },
            "download_url": "https://ccmixter.org/content/user/stems.zip"
        }),
    ];
    let url = find_audio_url(&files);
    assert!(url.is_none());
}

#[test]
fn test_find_audio_url_empty_files() {
    let files: Vec<Value> = vec![];
    let url = find_audio_url(&files);
    assert!(url.is_none());
}

#[test]
fn test_find_audio_url_skips_empty_download_url() {
    let files = vec![
        json!({
            "file_name": "track.mp3",
            "file_format_info": { "mime_type": "audio/mpeg" },
            "download_url": ""
        }),
        json!({
            "file_name": "track2.mp3",
            "file_format_info": { "mime_type": "audio/mpeg" },
            "download_url": "https://ccmixter.org/content/user/track2.mp3"
        }),
    ];
    let url = find_audio_url(&files);
    assert_eq!(url, Some("https://ccmixter.org/content/user/track2.mp3".to_string()));
}

// ============================================================
// Tests for extract_group
// ============================================================

#[test]
fn test_extract_group_from_ccud() {
    let upload = json!({
        "upload_extra": { "ccud": "remix" },
        "upload_tags": "electronic,beat"
    });
    assert_eq!(extract_group(&upload), "Remixes");
}

#[test]
fn test_extract_group_sample() {
    let upload = json!({
        "upload_extra": { "ccud": "sample" }
    });
    assert_eq!(extract_group(&upload), "Samples");
}

#[test]
fn test_extract_group_a_cappella() {
    let upload = json!({
        "upload_extra": { "ccud": "a_cappella" }
    });
    assert_eq!(extract_group(&upload), "A Cappellas");
}

#[test]
fn test_extract_group_fallback_to_upload_tags() {
    let upload = json!({
        "upload_extra": {},
        "upload_tags": "electronic,beat,chill"
    });
    assert_eq!(extract_group(&upload), "Electronic");
}

#[test]
fn test_extract_group_no_tags() {
    let upload = json!({
        "upload_extra": {}
    });
    assert_eq!(extract_group(&upload), "Other");
}

#[test]
fn test_extract_group_empty_ccud() {
    let upload = json!({
        "upload_extra": { "ccud": "" },
        "upload_tags": "hiphop,rap"
    });
    assert_eq!(extract_group(&upload), "Hiphop");
}

// ============================================================
// Tests for extract_tags
// ============================================================

#[test]
fn test_extract_tags_from_usertags() {
    let upload = json!({
        "upload_extra": { "usertags": "electronic,chill,ambient" },
        "upload_tags": "remix,beat"
    });
    let tags = extract_tags(&upload);
    assert_eq!(tags, vec!["electronic", "chill", "ambient"]);
}

#[test]
fn test_extract_tags_fallback_to_upload_tags() {
    let upload = json!({
        "upload_extra": {},
        "upload_tags": "remix,beat,electronic"
    });
    let tags = extract_tags(&upload);
    assert_eq!(tags, vec!["remix", "beat", "electronic"]);
}

#[test]
fn test_extract_tags_empty() {
    let upload = json!({
        "upload_extra": {},
        "upload_tags": ""
    });
    let tags = extract_tags(&upload);
    assert!(tags.is_empty());
}

#[test]
fn test_extract_tags_trims_whitespace() {
    let upload = json!({
        "upload_extra": { "usertags": " electronic , chill , ambient " }
    });
    let tags = extract_tags(&upload);
    assert_eq!(tags, vec!["electronic", "chill", "ambient"]);
}

// ============================================================
// Tests for format_group_name
// ============================================================

#[test]
fn test_format_group_name_known_types() {
    assert_eq!(format_group_name("remix"), "Remixes");
    assert_eq!(format_group_name("Remix"), "Remixes");
    assert_eq!(format_group_name("sample"), "Samples");
    assert_eq!(format_group_name("a_cappella"), "A Cappellas");
    assert_eq!(format_group_name("editorial_pick"), "Editorial Picks");
}

#[test]
fn test_format_group_name_unknown_type() {
    assert_eq!(format_group_name("electronic beats"), "Electronic Beats");
    assert_eq!(format_group_name("hip_hop"), "Hip Hop");
}

// ============================================================
// Tests for upload_to_stream
// ============================================================

#[test]
fn test_upload_to_stream_complete() {
    let upload = json!({
        "upload_id": 12345,
        "upload_name": "Cool Remix",
        "user_name": "djcool",
        "user_real_name": "DJ Cool",
        "upload_tags": "remix,electronic",
        "upload_extra": {
            "ccud": "remix",
            "usertags": "electronic,dance"
        },
        "license_name": "Attribution (4.0)",
        "license_logo_url": "https://i.creativecommons.org/l/by/4.0/88x31.png",
        "files": [
            {
                "file_name": "cool_remix.mp3",
                "file_format_info": { "mime_type": "audio/mpeg" },
                "download_url": "https://ccmixter.org/content/djcool/cool_remix.mp3"
            }
        ]
    });

    let stream = upload_to_stream(&upload).unwrap();
    assert_eq!(stream.id, "12345");
    assert_eq!(stream.name, "Cool Remix - DJ Cool");
    assert_eq!(stream.url, "https://ccmixter.org/content/djcool/cool_remix.mp3");
    assert_eq!(stream.group, "Remixes");
    assert_eq!(stream.logo, "https://i.creativecommons.org/l/by/4.0/88x31.png");
    assert_eq!(stream.vod_type, "");
    assert_eq!(stream.tags, vec!["electronic", "dance"]);
    assert!(stream.http_headers.is_some());
}

#[test]
fn test_upload_to_stream_no_audio_returns_none() {
    let upload = json!({
        "upload_id": 99999,
        "upload_name": "No Audio Track",
        "user_name": "someone",
        "files": [
            {
                "file_name": "stems.zip",
                "file_format_info": { "mime_type": "application/zip" },
                "download_url": "https://ccmixter.org/content/someone/stems.zip"
            }
        ]
    });

    let stream = upload_to_stream(&upload);
    assert!(stream.is_none());
}

#[test]
fn test_upload_to_stream_no_upload_id() {
    let upload = json!({
        "upload_name": "No ID Track",
        "user_name": "someone",
        "files": [
            {
                "file_name": "track.mp3",
                "file_format_info": { "mime_type": "audio/mpeg" },
                "download_url": "https://ccmixter.org/content/someone/track.mp3"
            }
        ]
    });

    let stream = upload_to_stream(&upload);
    assert!(stream.is_none());
}

#[test]
fn test_upload_to_stream_string_upload_id() {
    let upload = json!({
        "upload_id": "67890",
        "upload_name": "String ID Track",
        "user_name": "artist",
        "files": [
            {
                "file_name": "track.mp3",
                "file_format_info": { "mime_type": "audio/mpeg" },
                "download_url": "https://ccmixter.org/content/artist/track.mp3"
            }
        ]
    });

    let stream = upload_to_stream(&upload).unwrap();
    assert_eq!(stream.id, "67890");
}

#[test]
fn test_upload_to_stream_uses_user_name_fallback() {
    let upload = json!({
        "upload_id": 11111,
        "upload_name": "Test Track",
        "user_name": "fallback_user",
        "files": [
            {
                "file_name": "track.mp3",
                "file_format_info": { "mime_type": "audio/mpeg" },
                "download_url": "https://ccmixter.org/content/user/track.mp3"
            }
        ]
    });

    let stream = upload_to_stream(&upload).unwrap();
    assert_eq!(stream.name, "Test Track - fallback_user");
}

#[test]
fn test_upload_to_stream_no_files_array() {
    let upload = json!({
        "upload_id": 22222,
        "upload_name": "Missing Files",
        "user_name": "someone"
    });

    let stream = upload_to_stream(&upload);
    assert!(stream.is_none());
}

#[test]
fn test_upload_to_stream_http_headers_present() {
    let upload = json!({
        "upload_id": 33333,
        "upload_name": "Headers Test",
        "user_name": "artist",
        "files": [
            {
                "file_name": "track.mp3",
                "file_format_info": { "mime_type": "audio/mpeg" },
                "download_url": "https://ccmixter.org/content/artist/track.mp3"
            }
        ]
    });

    let stream = upload_to_stream(&upload).unwrap();
    let headers = stream.http_headers.unwrap();
    assert_eq!(headers["Referer"], "https://ccmixter.org/");
    assert_eq!(headers["User-Agent"], "Mozilla/5.0");
}

// ============================================================
// Tests for multiple uploads parsing
// ============================================================

#[test]
fn test_parse_multiple_uploads() {
    let uploads = vec![
        json!({
            "upload_id": 1,
            "upload_name": "Track One",
            "user_name": "artist1",
            "upload_extra": { "ccud": "remix" },
            "files": [{
                "file_format_info": { "mime_type": "audio/mpeg" },
                "download_url": "https://ccmixter.org/content/artist1/track1.mp3"
            }]
        }),
        json!({
            "upload_id": 2,
            "upload_name": "Track Two",
            "user_name": "artist2",
            "upload_extra": { "ccud": "sample" },
            "files": [{
                "file_format_info": { "mime_type": "audio/mpeg" },
                "download_url": "https://ccmixter.org/content/artist2/track2.mp3"
            }]
        }),
        json!({
            "upload_id": 3,
            "upload_name": "No Audio",
            "user_name": "artist3",
            "files": [{
                "file_format_info": { "mime_type": "application/zip" },
                "download_url": "https://ccmixter.org/content/artist3/stems.zip"
            }]
        }),
    ];

    let streams: Vec<Stream> = uploads
        .iter()
        .filter_map(|u| upload_to_stream(u))
        .collect();

    assert_eq!(streams.len(), 2);
    assert_eq!(streams[0].group, "Remixes");
    assert_eq!(streams[1].group, "Samples");
}
