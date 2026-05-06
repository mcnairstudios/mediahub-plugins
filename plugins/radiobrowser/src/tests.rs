use super::*;
use std::collections::HashSet;

// ============================================================
// JSON parsing tests
// ============================================================

#[test]
fn test_parse_stations_basic() {
    let json = r#"[
        {
            "stationuuid": "abc-123",
            "name": "Jazz FM",
            "url_resolved": "http://stream.jazzfm.com/live",
            "url": "http://fallback.jazzfm.com/live",
            "favicon": "http://jazzfm.com/logo.png",
            "tags": "jazz,smooth jazz,instrumental",
            "country": "United Kingdom",
            "codec": "MP3",
            "bitrate": 128
        },
        {
            "stationuuid": "def-456",
            "name": "Rock Radio",
            "url_resolved": "http://stream.rockradio.com/live",
            "url": "",
            "favicon": "",
            "tags": "rock,classic rock",
            "country": "Germany",
            "codec": "AAC",
            "bitrate": 192
        }
    ]"#;

    let stations = parse_stations(json.as_bytes());
    assert_eq!(stations.len(), 2);

    assert_eq!(stations[0].stationuuid.as_deref(), Some("abc-123"));
    assert_eq!(stations[0].name.as_deref(), Some("Jazz FM"));
    assert_eq!(
        stations[0].url_resolved.as_deref(),
        Some("http://stream.jazzfm.com/live")
    );
    assert_eq!(stations[0].favicon.as_deref(), Some("http://jazzfm.com/logo.png"));
    assert_eq!(stations[0].bitrate, Some(128));

    assert_eq!(stations[1].stationuuid.as_deref(), Some("def-456"));
    assert_eq!(stations[1].name.as_deref(), Some("Rock Radio"));
}

#[test]
fn test_parse_stations_empty_array() {
    let json = b"[]";
    let stations = parse_stations(json);
    assert!(stations.is_empty());
}

#[test]
fn test_parse_stations_invalid_json() {
    let json = b"not json at all";
    let stations = parse_stations(json);
    assert!(stations.is_empty());
}

#[test]
fn test_parse_stations_missing_optional_fields() {
    let json = r#"[
        {
            "stationuuid": "xyz-789",
            "name": "Minimal Station"
        }
    ]"#;

    let stations = parse_stations(json.as_bytes());
    assert_eq!(stations.len(), 1);
    assert_eq!(stations[0].stationuuid.as_deref(), Some("xyz-789"));
    assert!(stations[0].url_resolved.is_none());
    assert!(stations[0].favicon.is_none());
    assert!(stations[0].tags.is_none());
}

// ============================================================
// Tag splitting tests
// ============================================================

#[test]
fn test_split_tags_basic() {
    let tags = split_tags("jazz,rock,classical");
    assert_eq!(tags, vec!["jazz", "rock", "classical"]);
}

#[test]
fn test_split_tags_with_spaces() {
    let tags = split_tags("jazz , rock , classic rock");
    assert_eq!(tags, vec!["jazz", "rock", "classic rock"]);
}

#[test]
fn test_split_tags_empty_string() {
    let tags = split_tags("");
    assert!(tags.is_empty());
}

#[test]
fn test_split_tags_deduplication() {
    let tags = split_tags("rock,Rock,ROCK,jazz,Jazz");
    assert_eq!(tags, vec!["rock", "jazz"]);
}

#[test]
fn test_split_tags_trailing_comma() {
    let tags = split_tags("jazz,rock,");
    assert_eq!(tags, vec!["jazz", "rock"]);
}

#[test]
fn test_split_tags_empty_segments() {
    let tags = split_tags("jazz,,rock,,,classical");
    assert_eq!(tags, vec!["jazz", "rock", "classical"]);
}

// ============================================================
// Station-to-stream mapping tests
// ============================================================

#[test]
fn test_station_to_stream_basic() {
    let station = RadioStation {
        stationuuid: Some("abc-123".to_string()),
        name: Some("Jazz FM".to_string()),
        url_resolved: Some("http://stream.jazzfm.com/live".to_string()),
        url: Some("http://fallback.jazzfm.com/live".to_string()),
        favicon: Some("http://jazzfm.com/logo.png".to_string()),
        tags: Some("jazz,smooth jazz".to_string()),
        country: Some("United Kingdom".to_string()),
        codec: Some("MP3".to_string()),
        bitrate: Some(128),
    };

    let stream = station_to_stream(&station, "jazz").unwrap();
    assert_eq!(stream.id, "abc-123");
    assert_eq!(stream.name, "Jazz FM");
    assert_eq!(stream.url, "http://stream.jazzfm.com/live");
    assert_eq!(stream.group, "jazz");
    assert_eq!(stream.logo, "http://jazzfm.com/logo.png");
    assert_eq!(stream.tags, vec!["jazz", "smooth jazz"]);
}

#[test]
fn test_station_to_stream_falls_back_to_url() {
    let station = RadioStation {
        stationuuid: Some("abc-123".to_string()),
        name: Some("Test Station".to_string()),
        url_resolved: Some(String::new()),
        url: Some("http://fallback.com/stream".to_string()),
        favicon: None,
        tags: None,
        country: None,
        codec: None,
        bitrate: None,
    };

    let stream = station_to_stream(&station, "test").unwrap();
    assert_eq!(stream.url, "http://fallback.com/stream");
}

#[test]
fn test_station_to_stream_no_url_returns_none() {
    let station = RadioStation {
        stationuuid: Some("abc-123".to_string()),
        name: Some("Dead Station".to_string()),
        url_resolved: None,
        url: None,
        favicon: None,
        tags: None,
        country: None,
        codec: None,
        bitrate: None,
    };

    assert!(station_to_stream(&station, "test").is_none());
}

#[test]
fn test_station_to_stream_no_uuid_returns_none() {
    let station = RadioStation {
        stationuuid: None,
        name: Some("No ID Station".to_string()),
        url_resolved: Some("http://stream.com/live".to_string()),
        url: None,
        favicon: None,
        tags: None,
        country: None,
        codec: None,
        bitrate: None,
    };

    assert!(station_to_stream(&station, "test").is_none());
}

#[test]
fn test_station_to_stream_empty_uuid_returns_none() {
    let station = RadioStation {
        stationuuid: Some(String::new()),
        name: Some("Empty ID Station".to_string()),
        url_resolved: Some("http://stream.com/live".to_string()),
        url: None,
        favicon: None,
        tags: None,
        country: None,
        codec: None,
        bitrate: None,
    };

    assert!(station_to_stream(&station, "test").is_none());
}

#[test]
fn test_station_to_stream_missing_name_uses_default() {
    let station = RadioStation {
        stationuuid: Some("abc-123".to_string()),
        name: None,
        url_resolved: Some("http://stream.com/live".to_string()),
        url: None,
        favicon: None,
        tags: None,
        country: None,
        codec: None,
        bitrate: None,
    };

    let stream = station_to_stream(&station, "test").unwrap();
    assert_eq!(stream.name, "Unknown Station");
}

// ============================================================
// Deduplication tests
// ============================================================

#[test]
fn test_collect_streams_deduplication() {
    let stations = vec![
        RadioStation {
            stationuuid: Some("abc-123".to_string()),
            name: Some("Station A".to_string()),
            url_resolved: Some("http://a.com/stream".to_string()),
            url: None,
            favicon: None,
            tags: Some("rock".to_string()),
            country: None,
            codec: None,
            bitrate: None,
        },
        RadioStation {
            stationuuid: Some("abc-123".to_string()), // duplicate UUID
            name: Some("Station A (Duplicate)".to_string()),
            url_resolved: Some("http://a2.com/stream".to_string()),
            url: None,
            favicon: None,
            tags: Some("rock".to_string()),
            country: None,
            codec: None,
            bitrate: None,
        },
        RadioStation {
            stationuuid: Some("def-456".to_string()),
            name: Some("Station B".to_string()),
            url_resolved: Some("http://b.com/stream".to_string()),
            url: None,
            favicon: None,
            tags: Some("jazz".to_string()),
            country: None,
            codec: None,
            bitrate: None,
        },
    ];

    let mut seen = HashSet::new();
    let streams = collect_streams(&stations, "rock", &mut seen);
    assert_eq!(streams.len(), 2);
    assert_eq!(streams[0].id, "abc-123");
    assert_eq!(streams[1].id, "def-456");
}

#[test]
fn test_collect_streams_cross_group_deduplication() {
    let stations_a = vec![RadioStation {
        stationuuid: Some("shared-id".to_string()),
        name: Some("Shared Station".to_string()),
        url_resolved: Some("http://shared.com/stream".to_string()),
        url: None,
        favicon: None,
        tags: Some("rock,jazz".to_string()),
        country: None,
        codec: None,
        bitrate: None,
    }];

    let stations_b = vec![RadioStation {
        stationuuid: Some("shared-id".to_string()), // same station in different group
        name: Some("Shared Station".to_string()),
        url_resolved: Some("http://shared.com/stream".to_string()),
        url: None,
        favicon: None,
        tags: Some("rock,jazz".to_string()),
        country: None,
        codec: None,
        bitrate: None,
    }];

    let mut seen = HashSet::new();
    let streams_a = collect_streams(&stations_a, "rock", &mut seen);
    let streams_b = collect_streams(&stations_b, "jazz", &mut seen);

    assert_eq!(streams_a.len(), 1);
    assert_eq!(streams_b.len(), 0); // deduplicated across groups
}

#[test]
fn test_collect_streams_skips_invalid_stations() {
    let stations = vec![
        RadioStation {
            stationuuid: None, // no UUID
            name: Some("No UUID".to_string()),
            url_resolved: Some("http://a.com/stream".to_string()),
            url: None,
            favicon: None,
            tags: None,
            country: None,
            codec: None,
            bitrate: None,
        },
        RadioStation {
            stationuuid: Some("valid-id".to_string()),
            name: Some("Valid".to_string()),
            url_resolved: None, // no URL
            url: None,
            favicon: None,
            tags: None,
            country: None,
            codec: None,
            bitrate: None,
        },
        RadioStation {
            stationuuid: Some("good-id".to_string()),
            name: Some("Good Station".to_string()),
            url_resolved: Some("http://good.com/stream".to_string()),
            url: None,
            favicon: None,
            tags: Some("pop".to_string()),
            country: None,
            codec: None,
            bitrate: None,
        },
    ];

    let mut seen = HashSet::new();
    let streams = collect_streams(&stations, "test", &mut seen);
    assert_eq!(streams.len(), 1);
    assert_eq!(streams[0].id, "good-id");
}

// ============================================================
// URL encoding tests
// ============================================================

#[test]
fn test_url_encode_simple() {
    assert_eq!(url_encode("jazz"), "jazz");
    assert_eq!(url_encode("rock"), "rock");
}

#[test]
fn test_url_encode_spaces() {
    assert_eq!(url_encode("classic rock"), "classic%20rock");
}

#[test]
fn test_url_encode_special_chars() {
    assert_eq!(url_encode("r&b"), "r%26b");
    assert_eq!(url_encode("hip-hop"), "hip-hop");
    assert_eq!(url_encode("drum_and_bass"), "drum_and_bass");
}

// ============================================================
// Describe output tests
// ============================================================

#[test]
fn test_describe_json_structure() {
    let desc = Descriptor {
        r#type: "radiobrowser",
        label: "Radio Browser",
        short_label: "RADIO",
        color: "#ff9800",
        version: "1.0.0",
        description: "Browse 90,000+ internet radio stations",
        config_fields: vec![
            serde_json::json!({
                "key": "mode",
                "label": "Browse by",
                "type": "select",
                "required": true,
                "default": "tag"
            }),
            serde_json::json!({
                "key": "tags",
                "label": "Genres",
                "type": "text"
            }),
            serde_json::json!({
                "key": "countries",
                "label": "Countries",
                "type": "text"
            }),
        ],
        view: View {
            layout: "grouped_list",
            group_by: "group",
            searchable: true,
            sortable: true,
        },
        interactions: vec![serde_json::json!({
            "id": "search_stations",
            "label": "Search Stations",
            "type": "search"
        })],
    };

    let json_str = serde_json::to_string(&desc).unwrap();
    let parsed: Value = serde_json::from_str(&json_str).unwrap();

    assert_eq!(parsed["type"], "radiobrowser");
    assert_eq!(parsed["label"], "Radio Browser");
    assert_eq!(parsed["short_label"], "RADIO");
    assert_eq!(parsed["color"], "#ff9800");
    assert_eq!(parsed["view"]["layout"], "grouped_list");
    assert_eq!(parsed["view"]["group_by"], "group");
    assert_eq!(parsed["view"]["searchable"], true);
    assert_eq!(parsed["config_fields"].as_array().unwrap().len(), 3);
    assert_eq!(parsed["interactions"].as_array().unwrap().len(), 1);
    assert_eq!(parsed["interactions"][0]["id"], "search_stations");
}

// ============================================================
// Full response parsing (simulated API response)
// ============================================================

#[test]
fn test_parse_realistic_api_response() {
    let json = r#"[
        {
            "changeuuid": "change-1",
            "stationuuid": "uuid-001",
            "serveruuid": null,
            "name": "BBC Radio 1",
            "url": "http://bbcmedia.ic.llnwd.net/stream/bbcmedia_radio1_mf_p",
            "url_resolved": "http://stream.live.vc.bbcmedia.co.uk/bbc_radio_one",
            "homepage": "https://www.bbc.co.uk/radio1",
            "favicon": "https://cdn-radiotime-logos.tunein.com/s24939q.png",
            "tags": "bbc,pop,dance,electronic,uk",
            "country": "United Kingdom",
            "countrycode": "GB",
            "state": "",
            "language": "english",
            "languagecodes": "en",
            "votes": 12345,
            "lastchangetime": "2024-01-01 00:00:00",
            "lastchangetime_iso8601": "2024-01-01T00:00:00Z",
            "codec": "MP3",
            "bitrate": 128,
            "hls": 0,
            "lastcheckok": 1,
            "lastchecktime": "2024-01-02 00:00:00",
            "lastchecktime_iso8601": "2024-01-02T00:00:00Z",
            "clicktimestamp": "",
            "clicktimestamp_iso8601": null,
            "clickcount": 500,
            "clicktrend": 10,
            "ssl_error": 0,
            "geo_lat": null,
            "geo_long": null,
            "has_extended_info": false
        }
    ]"#;

    let stations = parse_stations(json.as_bytes());
    assert_eq!(stations.len(), 1);

    let stream = station_to_stream(&stations[0], "pop").unwrap();
    assert_eq!(stream.id, "uuid-001");
    assert_eq!(stream.name, "BBC Radio 1");
    assert_eq!(
        stream.url,
        "http://stream.live.vc.bbcmedia.co.uk/bbc_radio_one"
    );
    assert_eq!(stream.group, "pop");
    assert_eq!(
        stream.logo,
        "https://cdn-radiotime-logos.tunein.com/s24939q.png"
    );
    assert_eq!(stream.tags, vec!["bbc", "pop", "dance", "electronic", "uk"]);
}
