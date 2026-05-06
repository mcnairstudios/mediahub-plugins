use super::*;
use serde_json::json;

// ============================================================
// district_api_url tests
// ============================================================

#[test]
fn test_district_api_url_single_digit() {
    let url = district_api_url(3);
    assert_eq!(
        url,
        "https://cwwp2.dot.ca.gov/data/d3/cctv/cctvStatusD03.json"
    );
}

#[test]
fn test_district_api_url_double_digit() {
    let url = district_api_url(11);
    assert_eq!(
        url,
        "https://cwwp2.dot.ca.gov/data/d11/cctv/cctvStatusD11.json"
    );
}

#[test]
fn test_district_api_url_all_districts() {
    for d in DISTRICTS {
        let url = district_api_url(d.number);
        assert!(url.contains(&format!("/d{}/", d.number)));
        assert!(url.contains(&format!("D{:02}.json", d.number)));
    }
}

// ============================================================
// district_label tests
// ============================================================

#[test]
fn test_district_label_known() {
    assert_eq!(district_label(7), "D7 - Los Angeles");
    assert_eq!(district_label(4), "D4 - Bay Area");
    assert_eq!(district_label(12), "D12 - Orange County");
}

#[test]
fn test_district_label_unknown() {
    assert_eq!(district_label(99), "Unknown District");
}

// ============================================================
// build_camera_name tests
// ============================================================

#[test]
fn test_build_camera_name_full() {
    let name = build_camera_name("I-5", "Burbank Blvd", "Burbank", "NB");
    assert_eq!(name, "I-5 at Burbank Blvd (NB)");
}

#[test]
fn test_build_camera_name_no_location_uses_nearby() {
    let name = build_camera_name("SR-99", "", "Fresno", "SB");
    assert_eq!(name, "SR-99 near Fresno (SB)");
}

#[test]
fn test_build_camera_name_route_only() {
    let name = build_camera_name("US-101", "", "", "");
    assert_eq!(name, "US-101");
}

#[test]
fn test_build_camera_name_empty() {
    let name = build_camera_name("", "", "", "");
    assert_eq!(name, "Unknown Camera");
}

#[test]
fn test_build_camera_name_location_no_route() {
    let name = build_camera_name("", "Main Street", "", "EB");
    assert_eq!(name, "at Main Street (EB)");
}

// ============================================================
// parse_camera tests
// ============================================================

fn make_camera_json(
    index: &str,
    in_service: &str,
    streaming_url: &str,
    route: &str,
    location_name: &str,
    nearby_place: &str,
    county: &str,
    direction: &str,
) -> Value {
    json!({
        "index": index,
        "inService": in_service,
        "streamingVideoURL": streaming_url,
        "location": {
            "route": route,
            "locationName": location_name,
            "nearbyPlace": nearby_place,
            "county": county,
            "direction": direction
        }
    })
}

#[test]
fn test_parse_camera_valid() {
    let cam = make_camera_json(
        "CCTV-196",
        "TRUE",
        "https://wzmedia.dot.ca.gov/D7/CCTV-196.stream/playlist.m3u8",
        "I-5",
        "Burbank Blvd",
        "Burbank",
        "Los Angeles",
        "NB",
    );

    let result = parse_camera(&cam, 7);
    assert!(result.is_some());

    let stream = result.unwrap();
    assert_eq!(stream.id, "caltrans-d7-CCTV-196");
    assert_eq!(stream.name, "I-5 at Burbank Blvd (NB)");
    assert_eq!(
        stream.url,
        "https://wzmedia.dot.ca.gov/D7/CCTV-196.stream/playlist.m3u8"
    );
    assert_eq!(stream.group, "D7 - Los Angeles");
    assert!(stream.tags.contains(&"california".to_string()));
    assert!(stream.tags.contains(&"i-5".to_string()));
    assert!(stream.tags.contains(&"los angeles".to_string()));
    assert!(stream.tags.contains(&"nb".to_string()));
}

#[test]
fn test_parse_camera_not_in_service() {
    let cam = make_camera_json(
        "CCTV-100",
        "FALSE",
        "https://wzmedia.dot.ca.gov/D7/CCTV-100.stream/playlist.m3u8",
        "I-5",
        "Test",
        "",
        "LA",
        "SB",
    );

    let result = parse_camera(&cam, 7);
    assert!(result.is_none());
}

#[test]
fn test_parse_camera_no_streaming_url() {
    let cam = make_camera_json(
        "CCTV-200",
        "TRUE",
        "",
        "US-101",
        "Test",
        "",
        "Ventura",
        "NB",
    );

    let result = parse_camera(&cam, 7);
    assert!(result.is_none());
}

#[test]
fn test_parse_camera_missing_location_fields() {
    let cam = json!({
        "index": "CCTV-300",
        "inService": "TRUE",
        "streamingVideoURL": "https://wzmedia.dot.ca.gov/D4/CCTV-300.stream/playlist.m3u8",
        "location": {}
    });

    let result = parse_camera(&cam, 4);
    assert!(result.is_some());

    let stream = result.unwrap();
    assert_eq!(stream.name, "Unknown Camera");
    assert_eq!(stream.group, "D4 - Bay Area");
}

// ============================================================
// parse_district_cameras tests
// ============================================================

#[test]
fn test_parse_district_cameras_with_data_wrapper() {
    let json_data = json!({
        "data": [
            {
                "cctv": {
                    "index": "CAM-1",
                    "inService": "TRUE",
                    "streamingVideoURL": "https://wzmedia.dot.ca.gov/D7/CAM-1.stream/playlist.m3u8",
                    "location": {
                        "route": "I-405",
                        "locationName": "Wilshire Blvd",
                        "nearbyPlace": "Westwood",
                        "county": "Los Angeles",
                        "direction": "NB"
                    }
                }
            },
            {
                "cctv": {
                    "index": "CAM-2",
                    "inService": "FALSE",
                    "streamingVideoURL": "https://wzmedia.dot.ca.gov/D7/CAM-2.stream/playlist.m3u8",
                    "location": {
                        "route": "I-10",
                        "locationName": "La Brea",
                        "nearbyPlace": "",
                        "county": "Los Angeles",
                        "direction": "EB"
                    }
                }
            },
            {
                "cctv": {
                    "index": "CAM-3",
                    "inService": "TRUE",
                    "streamingVideoURL": "https://wzmedia.dot.ca.gov/D7/CAM-3.stream/playlist.m3u8",
                    "location": {
                        "route": "SR-110",
                        "locationName": "Figueroa St",
                        "nearbyPlace": "Downtown LA",
                        "county": "Los Angeles",
                        "direction": "SB"
                    }
                }
            }
        ]
    });

    let body = serde_json::to_vec(&json_data).unwrap();
    let streams = parse_district_cameras(&body, 7);

    // CAM-2 is not in service, so only 2 results
    assert_eq!(streams.len(), 2);
    assert_eq!(streams[0].id, "caltrans-d7-CAM-1");
    assert_eq!(streams[1].id, "caltrans-d7-CAM-3");
}

#[test]
fn test_parse_district_cameras_direct_array() {
    let json_data = json!([
        {
            "index": "CAM-10",
            "inService": "TRUE",
            "streamingVideoURL": "https://wzmedia.dot.ca.gov/D3/CAM-10.stream/playlist.m3u8",
            "location": {
                "route": "I-80",
                "locationName": "Capital City Freeway",
                "nearbyPlace": "Sacramento",
                "county": "Sacramento",
                "direction": "EB"
            }
        }
    ]);

    let body = serde_json::to_vec(&json_data).unwrap();
    let streams = parse_district_cameras(&body, 3);

    assert_eq!(streams.len(), 1);
    assert_eq!(streams[0].id, "caltrans-d3-CAM-10");
    assert_eq!(streams[0].group, "D3 - Sacramento");
}

#[test]
fn test_parse_district_cameras_invalid_json() {
    let body = b"this is not json";
    let streams = parse_district_cameras(body, 7);
    assert!(streams.is_empty());
}

#[test]
fn test_parse_district_cameras_empty_data() {
    let json_data = json!({ "data": [] });
    let body = serde_json::to_vec(&json_data).unwrap();
    let streams = parse_district_cameras(&body, 7);
    assert!(streams.is_empty());
}

// ============================================================
// HLS URL construction tests
// ============================================================

#[test]
fn test_hls_url_format() {
    // Verify the expected HLS URL pattern from Caltrans
    let district = 7;
    let camera_id = "CCTV-196";
    let expected = format!(
        "https://wzmedia.dot.ca.gov/D{}/{}.stream/playlist.m3u8",
        district, camera_id
    );
    assert_eq!(
        expected,
        "https://wzmedia.dot.ca.gov/D7/CCTV-196.stream/playlist.m3u8"
    );
}

#[test]
fn test_parsed_camera_preserves_hls_url() {
    let hls_url = "https://wzmedia.dot.ca.gov/D11/CCTV-500.stream/playlist.m3u8";
    let cam = make_camera_json(
        "CCTV-500",
        "TRUE",
        hls_url,
        "I-5",
        "Border",
        "San Ysidro",
        "San Diego",
        "NB",
    );

    let stream = parse_camera(&cam, 11).unwrap();
    assert_eq!(stream.url, hls_url);
    assert!(stream.url.ends_with(".m3u8"));
    assert!(stream.url.contains("/D11/"));
}

// ============================================================
// District/county grouping tests
// ============================================================

#[test]
fn test_cameras_grouped_by_district() {
    let d7_cam = make_camera_json(
        "CAM-A",
        "TRUE",
        "https://wzmedia.dot.ca.gov/D7/CAM-A.stream/playlist.m3u8",
        "I-5",
        "Downtown",
        "",
        "Los Angeles",
        "NB",
    );
    let d4_cam = make_camera_json(
        "CAM-B",
        "TRUE",
        "https://wzmedia.dot.ca.gov/D4/CAM-B.stream/playlist.m3u8",
        "I-880",
        "Hegenberger",
        "Oakland",
        "Alameda",
        "SB",
    );

    let s7 = parse_camera(&d7_cam, 7).unwrap();
    let s4 = parse_camera(&d4_cam, 4).unwrap();

    assert_eq!(s7.group, "D7 - Los Angeles");
    assert_eq!(s4.group, "D4 - Bay Area");
    assert_ne!(s7.group, s4.group);
}

#[test]
fn test_county_in_tags() {
    let cam = make_camera_json(
        "CAM-C",
        "TRUE",
        "https://wzmedia.dot.ca.gov/D11/CAM-C.stream/playlist.m3u8",
        "I-15",
        "Escondido",
        "",
        "San Diego",
        "NB",
    );

    let stream = parse_camera(&cam, 11).unwrap();
    assert!(stream.tags.contains(&"san diego".to_string()));
}

// ============================================================
// Config parsing tests
// ============================================================

#[test]
fn test_get_selected_districts_default_all() {
    let config = serde_json::Map::new();
    let selected = get_selected_districts(&config);
    assert_eq!(selected.len(), DISTRICTS.len());
}

#[test]
fn test_get_selected_districts_from_array() {
    let mut config = serde_json::Map::new();
    config.insert(
        "districts".to_string(),
        json!(["7", "4", "11"]),
    );
    let selected = get_selected_districts(&config);
    assert_eq!(selected.len(), 3);
    assert!(selected.contains(&7));
    assert!(selected.contains(&4));
    assert!(selected.contains(&11));
}

#[test]
fn test_get_selected_districts_from_string() {
    let mut config = serde_json::Map::new();
    config.insert(
        "districts".to_string(),
        json!("[\"3\", \"8\"]"),
    );
    let selected = get_selected_districts(&config);
    assert_eq!(selected.len(), 2);
    assert!(selected.contains(&3));
    assert!(selected.contains(&8));
}

#[test]
fn test_get_selected_districts_filters_invalid() {
    let mut config = serde_json::Map::new();
    config.insert(
        "districts".to_string(),
        json!(["7", "99", "abc", "4"]),
    );
    let selected = get_selected_districts(&config);
    assert_eq!(selected.len(), 2);
    assert!(selected.contains(&7));
    assert!(selected.contains(&4));
}

// ============================================================
// Tag search matching tests
// ============================================================

#[test]
fn test_stream_tags_searchable() {
    let cam = make_camera_json(
        "CAM-D",
        "TRUE",
        "https://wzmedia.dot.ca.gov/D7/CAM-D.stream/playlist.m3u8",
        "I-405",
        "Getty Center Dr",
        "Bel Air",
        "Los Angeles",
        "NB",
    );

    let stream = parse_camera(&cam, 7).unwrap();

    // Simulate search matching logic
    let query = "i-405";
    let query_lower = query.to_lowercase();
    let name_match = stream.name.to_lowercase().contains(&query_lower);
    let tags_match = stream.tags.iter().any(|t| t.contains(&query_lower));
    assert!(name_match || tags_match);

    // Search by county
    let query2 = "los angeles";
    let query2_lower = query2.to_lowercase();
    let tags_match2 = stream.tags.iter().any(|t| t.contains(&query2_lower));
    assert!(tags_match2);
}

// ============================================================
// Edge cases
// ============================================================

#[test]
fn test_parse_camera_with_cctv_id_field() {
    // Some cameras use "cctv-id" instead of "index"
    let cam = json!({
        "cctv-id": "CCTV-999",
        "inService": "TRUE",
        "streamingVideoURL": "https://wzmedia.dot.ca.gov/D1/CCTV-999.stream/playlist.m3u8",
        "location": {
            "route": "US-101",
            "locationName": "",
            "nearbyPlace": "Eureka",
            "county": "Humboldt",
            "direction": "NB"
        }
    });

    let stream = parse_camera(&cam, 1).unwrap();
    assert_eq!(stream.id, "caltrans-d1-CCTV-999");
    assert_eq!(stream.name, "US-101 near Eureka (NB)");
    assert_eq!(stream.group, "D1 - Northwest");
}

#[test]
fn test_all_districts_have_labels() {
    assert_eq!(DISTRICTS.len(), 12);
    for d in DISTRICTS {
        assert!(!d.label.is_empty());
        assert!(d.number >= 1 && d.number <= 12);
    }
}

#[test]
fn test_stream_serialization() {
    let stream = Stream {
        id: "test-1".to_string(),
        name: "Test Camera".to_string(),
        url: "https://example.com/stream.m3u8".to_string(),
        group: "D7 - Los Angeles".to_string(),
        logo: String::new(),
        vod_type: String::new(),
        tags: vec!["california".to_string(), "i-5".to_string()],
    };

    let json_str = serde_json::to_string(&stream).unwrap();
    let parsed: Value = serde_json::from_str(&json_str).unwrap();

    assert_eq!(parsed["id"], "test-1");
    assert_eq!(parsed["name"], "Test Camera");
    assert_eq!(parsed["url"], "https://example.com/stream.m3u8");
    assert_eq!(parsed["group"], "D7 - Los Angeles");
    assert_eq!(parsed["tags"][0], "california");
    assert_eq!(parsed["tags"][1], "i-5");
}
