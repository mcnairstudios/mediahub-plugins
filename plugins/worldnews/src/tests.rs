use super::*;

#[test]
fn test_build_streams_returns_all() {
    let streams = build_streams();
    assert_eq!(streams.len(), stream_count());
    assert!(
        streams.len() >= 25,
        "expected at least 25 streams, got {}",
        streams.len()
    );
}

#[test]
fn test_all_urls_are_valid() {
    let streams = build_streams();
    for s in &streams {
        assert!(
            s.url.starts_with("https://"),
            "stream '{}' URL must use HTTPS: {}",
            s.id,
            s.url
        );
        assert!(
            s.url.starts_with("https://www.youtube.com/watch?v=")
                || s.url.ends_with(".m3u8"),
            "stream '{}' has unexpected URL format (neither YouTube nor HLS): {}",
            s.id,
            s.url
        );
    }
}

#[test]
fn test_hls_urls_are_well_formed() {
    let streams = build_streams();
    let hls: Vec<_> = streams.iter().filter(|s| s.url.ends_with(".m3u8")).collect();
    assert!(
        hls.len() >= 8,
        "expected at least 8 HLS streams, got {}",
        hls.len()
    );
    for s in &hls {
        assert!(
            s.url.contains("://") && s.url.contains('/'),
            "HLS stream '{}' has malformed URL: {}",
            s.id,
            s.url
        );
    }
}

#[test]
fn test_youtube_urls_have_video_id() {
    let streams = build_streams();
    let yt: Vec<_> = streams
        .iter()
        .filter(|s| s.url.starts_with("https://www.youtube.com/watch?v="))
        .collect();
    assert!(
        yt.len() >= 10,
        "expected at least 10 YouTube streams, got {}",
        yt.len()
    );
    for s in &yt {
        let vid_id = s
            .url
            .strip_prefix("https://www.youtube.com/watch?v=")
            .unwrap();
        assert!(
            !vid_id.is_empty(),
            "stream '{}' has empty YouTube video ID",
            s.id
        );
    }
}

#[test]
fn test_groups_are_correct() {
    let groups = get_groups();
    assert_eq!(
        groups,
        vec![
            "International",
            "Europe",
            "Asia",
            "Americas",
            "Science & Space"
        ]
    );
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
fn test_international_streams_count() {
    let streams = build_streams();
    let intl: Vec<_> = streams
        .iter()
        .filter(|s| s.group == "International")
        .collect();
    assert!(
        intl.len() >= 5,
        "expected at least 5 International streams, got {}",
        intl.len()
    );
}

#[test]
fn test_europe_streams_count() {
    let streams = build_streams();
    let eu: Vec<_> = streams.iter().filter(|s| s.group == "Europe").collect();
    assert!(
        eu.len() >= 4,
        "expected at least 4 Europe streams, got {}",
        eu.len()
    );
}

#[test]
fn test_asia_streams_count() {
    let streams = build_streams();
    let asia: Vec<_> = streams.iter().filter(|s| s.group == "Asia").collect();
    assert!(
        asia.len() >= 4,
        "expected at least 4 Asia streams, got {}",
        asia.len()
    );
}

#[test]
fn test_americas_streams_count() {
    let streams = build_streams();
    let am: Vec<_> = streams.iter().filter(|s| s.group == "Americas").collect();
    assert!(
        am.len() >= 4,
        "expected at least 4 Americas streams, got {}",
        am.len()
    );
}

#[test]
fn test_science_space_streams_count() {
    let streams = build_streams();
    let sci: Vec<_> = streams
        .iter()
        .filter(|s| s.group == "Science & Space")
        .collect();
    assert!(
        sci.len() >= 2,
        "expected at least 2 Science & Space streams, got {}",
        sci.len()
    );
}

#[test]
fn test_all_streams_have_vod_type_movie() {
    let streams = build_streams();
    for s in &streams {
        assert_eq!(s.vod_type, "movie", "stream '{}' has wrong vod_type", s.id);
    }
}

#[test]
fn test_all_streams_have_tags() {
    let streams = build_streams();
    for s in &streams {
        assert!(s.tags.is_some(), "stream '{}' should have tags", s.id);
        let tags = s.tags.as_ref().unwrap();
        assert!(
            !tags.is_empty(),
            "stream '{}' should have at least one tag",
            s.id
        );
    }
}

#[test]
fn test_specific_hls_stream() {
    let streams = build_streams();
    let dw = streams.iter().find(|s| s.id == "dw-english").unwrap();
    assert_eq!(
        dw.url,
        "https://dwamdstream107.akamaized.net/hls/live/2017968/dwstream107/stream05/streamPlaylist.m3u8"
    );
    assert_eq!(dw.group, "International");
    assert_eq!(dw.name, "Deutsche Welle (English)");
}

#[test]
fn test_specific_youtube_stream() {
    let streams = build_streams();
    let abc = streams.iter().find(|s| s.id == "abc-news-live").unwrap();
    assert_eq!(abc.url, "https://www.youtube.com/watch?v=GUyXFaR0yqM");
    assert_eq!(abc.group, "Americas");
    assert_eq!(abc.name, "ABC News Live");
}

#[test]
fn test_specific_nasa_stream() {
    let streams = build_streams();
    let nasa = streams.iter().find(|s| s.id == "nasa-live").unwrap();
    assert_eq!(
        nasa.url,
        "https://ntv1.akamaized.net/hls/live/2014075/NASA-NTV1-HLS/master.m3u8"
    );
    assert_eq!(nasa.group, "Science & Space");
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
    assert_eq!(arr.len(), stream_count());

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

#[test]
fn test_aljazeera_hls_stream() {
    let streams = build_streams();
    let aj = streams.iter().find(|s| s.id == "aljazeera-english").unwrap();
    assert_eq!(aj.url, "https://live-hls-web-aje.getaj.net/AJE/index.m3u8");
    assert_eq!(aj.group, "International");
}

#[test]
fn test_kbs_world_stream() {
    let streams = build_streams();
    let kbs = streams.iter().find(|s| s.id == "kbs-world").unwrap();
    assert_eq!(
        kbs.url,
        "https://kbsworld-ott.akamaized.net/hls/live/2002341/kbsworld/master.m3u8"
    );
    assert_eq!(kbs.group, "Asia");
}
