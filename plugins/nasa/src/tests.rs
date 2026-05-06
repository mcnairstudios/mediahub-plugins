use super::*;

// ============================================================
// URL construction tests
// ============================================================

#[test]
fn test_build_search_url_basic() {
    let url = build_search_url("Mars", 50);
    assert_eq!(
        url,
        "https://images-api.nasa.gov/search?media_type=video&q=Mars&page_size=50"
    );
}

#[test]
fn test_build_search_url_with_spaces() {
    let url = build_search_url("space station", 25);
    assert_eq!(
        url,
        "https://images-api.nasa.gov/search?media_type=video&q=space%20station&page_size=25"
    );
}

#[test]
fn test_build_mp4_url() {
    let url = build_mp4_url("NHQ_2019_0311_Go Forward to the Moon");
    assert_eq!(
        url,
        "https://images-assets.nasa.gov/video/NHQ_2019_0311_Go Forward to the Moon/NHQ_2019_0311_Go Forward to the Moon~mobile.mp4"
    );
}

#[test]
fn test_build_mp4_url_simple_id() {
    let url = build_mp4_url("KSC-20230226-PH-SPX001");
    assert_eq!(
        url,
        "https://images-assets.nasa.gov/video/KSC-20230226-PH-SPX001/KSC-20230226-PH-SPX001~mobile.mp4"
    );
}

#[test]
fn test_build_thumb_url() {
    let url = build_thumb_url("iss070m261493940");
    assert_eq!(
        url,
        "https://images-assets.nasa.gov/video/iss070m261493940/iss070m261493940~thumb.jpg"
    );
}

// ============================================================
// Year extraction tests
// ============================================================

#[test]
fn test_extract_year_full_date() {
    assert_eq!(extract_year("2023-05-14T00:00:00Z"), Some("2023".to_string()));
}

#[test]
fn test_extract_year_just_year() {
    assert_eq!(extract_year("2019"), Some("2019".to_string()));
}

#[test]
fn test_extract_year_empty() {
    assert_eq!(extract_year(""), None);
}

#[test]
fn test_extract_year_short() {
    assert_eq!(extract_year("20"), None);
}

#[test]
fn test_extract_year_non_numeric() {
    assert_eq!(extract_year("abcd-01-01"), None);
}

// ============================================================
// Topics parsing tests
// ============================================================

#[test]
fn test_parse_topics_basic() {
    let topics = parse_topics("launch,ISS,Mars,Moon");
    assert_eq!(topics, vec!["launch", "ISS", "Mars", "Moon"]);
}

#[test]
fn test_parse_topics_with_whitespace() {
    let topics = parse_topics("  launch , ISS , Mars ");
    assert_eq!(topics, vec!["launch", "ISS", "Mars"]);
}

#[test]
fn test_parse_topics_empty_string() {
    let topics = parse_topics("");
    assert!(topics.is_empty());
}

#[test]
fn test_parse_topics_trailing_comma() {
    let topics = parse_topics("launch,ISS,");
    assert_eq!(topics, vec!["launch", "ISS"]);
}

#[test]
fn test_parse_topics_single() {
    let topics = parse_topics("Hubble");
    assert_eq!(topics, vec!["Hubble"]);
}

// ============================================================
// Search response parsing tests
// ============================================================

fn sample_search_response() -> Vec<u8> {
    serde_json::to_vec(&serde_json::json!({
        "collection": {
            "version": "1.0",
            "href": "https://images-api.nasa.gov/search?q=launch&media_type=video",
            "items": [
                {
                    "href": "https://images-api.nasa.gov/asset/KSC-20230226-MH-SPX001",
                    "data": [
                        {
                            "nasa_id": "KSC-20230226-MH-SPX001",
                            "title": "SpaceX Crew-6 Launch",
                            "description": "NASA SpaceX Crew-6 launched to the International Space Station.",
                            "date_created": "2023-02-26T00:00:00Z",
                            "center": "KSC",
                            "media_type": "video",
                            "keywords": ["SpaceX", "Crew-6", "ISS", "Launch"]
                        }
                    ],
                    "links": [
                        {
                            "href": "https://images-assets.nasa.gov/video/KSC-20230226-MH-SPX001/KSC-20230226-MH-SPX001~thumb.jpg",
                            "rel": "preview",
                            "render": "image"
                        }
                    ]
                },
                {
                    "href": "https://images-api.nasa.gov/asset/NHQ_2019_0311_Artemis",
                    "data": [
                        {
                            "nasa_id": "NHQ_2019_0311_Artemis",
                            "title": "Artemis Program Overview",
                            "description": "An overview of NASA's Artemis program to return humans to the Moon.",
                            "date_created": "2019-03-11T00:00:00Z",
                            "center": "HQ",
                            "media_type": "video",
                            "keywords": ["Artemis", "Moon", "SLS"]
                        }
                    ],
                    "links": [
                        {
                            "href": "https://images-assets.nasa.gov/video/NHQ_2019_0311_Artemis/NHQ_2019_0311_Artemis~thumb.jpg",
                            "rel": "preview",
                            "render": "image"
                        }
                    ]
                }
            ],
            "metadata": {
                "total_hits": 3310
            }
        }
    }))
    .unwrap()
}

#[test]
fn test_parse_search_response_count() {
    let body = sample_search_response();
    let streams = parse_search_response(&body, "launch");
    assert_eq!(streams.len(), 2);
}

#[test]
fn test_parse_search_response_first_item() {
    let body = sample_search_response();
    let streams = parse_search_response(&body, "launch");
    let first = &streams[0];

    assert_eq!(first.id, "KSC-20230226-MH-SPX001");
    assert_eq!(first.name, "SpaceX Crew-6 Launch");
    assert_eq!(first.group, "launch");
    assert_eq!(first.vod_type, "vod");
    assert_eq!(first.year, Some("2023".to_string()));
    assert_eq!(
        first.url,
        "https://images-assets.nasa.gov/video/KSC-20230226-MH-SPX001/KSC-20230226-MH-SPX001~mobile.mp4"
    );
    assert_eq!(
        first.logo,
        Some("https://images-assets.nasa.gov/video/KSC-20230226-MH-SPX001/KSC-20230226-MH-SPX001~thumb.jpg".to_string())
    );
}

#[test]
fn test_parse_search_response_second_item() {
    let body = sample_search_response();
    let streams = parse_search_response(&body, "launch");
    let second = &streams[1];

    assert_eq!(second.id, "NHQ_2019_0311_Artemis");
    assert_eq!(second.name, "Artemis Program Overview");
    assert_eq!(second.year, Some("2019".to_string()));
}

#[test]
fn test_parse_search_response_tags() {
    let body = sample_search_response();
    let streams = parse_search_response(&body, "launch");
    let first = &streams[0];

    let tags = first.tags.as_ref().expect("should have tags");
    assert_eq!(tags.len(), 4);
    assert!(tags.contains(&"spacex".to_string()));
    assert!(tags.contains(&"launch".to_string()));
    assert!(tags.contains(&"iss".to_string()));
}

#[test]
fn test_parse_search_response_description_truncation() {
    let long_desc = "A".repeat(250);
    let body = serde_json::to_vec(&serde_json::json!({
        "collection": {
            "items": [
                {
                    "data": [
                        {
                            "nasa_id": "TEST001",
                            "title": "Test Video",
                            "description": long_desc,
                            "date_created": "2024-01-01T00:00:00Z",
                            "media_type": "video"
                        }
                    ]
                }
            ]
        }
    }))
    .unwrap();

    let streams = parse_search_response(&body, "test");
    assert_eq!(streams.len(), 1);
    let desc = streams[0].description.as_ref().unwrap();
    assert_eq!(desc.len(), 200); // 197 chars + "..."
    assert!(desc.ends_with("..."));
}

#[test]
fn test_parse_search_response_short_description_not_truncated() {
    let body = serde_json::to_vec(&serde_json::json!({
        "collection": {
            "items": [
                {
                    "data": [
                        {
                            "nasa_id": "TEST002",
                            "title": "Short Desc Video",
                            "description": "A short description.",
                            "date_created": "2024-06-15T00:00:00Z",
                            "media_type": "video"
                        }
                    ]
                }
            ]
        }
    }))
    .unwrap();

    let streams = parse_search_response(&body, "test");
    assert_eq!(
        streams[0].description,
        Some("A short description.".to_string())
    );
}

// ============================================================
// Edge cases
// ============================================================

#[test]
fn test_parse_search_response_empty_collection() {
    let body = serde_json::to_vec(&serde_json::json!({
        "collection": {
            "items": []
        }
    }))
    .unwrap();

    let streams = parse_search_response(&body, "empty");
    assert!(streams.is_empty());
}

#[test]
fn test_parse_search_response_invalid_json() {
    let body = b"not valid json";
    let streams = parse_search_response(body, "bad");
    assert!(streams.is_empty());
}

#[test]
fn test_parse_search_response_missing_collection() {
    let body = serde_json::to_vec(&serde_json::json!({
        "error": "something went wrong"
    }))
    .unwrap();

    let streams = parse_search_response(&body, "error");
    assert!(streams.is_empty());
}

#[test]
fn test_parse_nasa_item_missing_nasa_id() {
    let item = serde_json::json!({
        "data": [
            {
                "title": "No ID Video",
                "description": "Missing nasa_id field"
            }
        ]
    });

    let result = parse_nasa_item(&item, "test");
    assert!(result.is_none());
}

#[test]
fn test_parse_nasa_item_empty_data_array() {
    let item = serde_json::json!({
        "data": []
    });

    let result = parse_nasa_item(&item, "test");
    assert!(result.is_none());
}

#[test]
fn test_parse_nasa_item_no_keywords() {
    let item = serde_json::json!({
        "data": [
            {
                "nasa_id": "TEST003",
                "title": "No Keywords Video",
                "date_created": "2022-08-01T00:00:00Z",
                "media_type": "video"
            }
        ]
    });

    let result = parse_nasa_item(&item, "misc").unwrap();
    assert_eq!(result.id, "TEST003");
    assert!(result.tags.is_none());
    assert!(result.description.is_none());
}

#[test]
fn test_parse_nasa_item_no_description() {
    let item = serde_json::json!({
        "data": [
            {
                "nasa_id": "TEST004",
                "title": "Video Without Description",
                "date_created": "2021-12-25T00:00:00Z",
                "media_type": "video",
                "keywords": ["Christmas"]
            }
        ]
    });

    let result = parse_nasa_item(&item, "holiday").unwrap();
    assert_eq!(result.name, "Video Without Description");
    assert!(result.description.is_none());
    assert_eq!(result.tags, Some(vec!["christmas".to_string()]));
}

// ============================================================
// Metadata extraction tests
// ============================================================

#[test]
fn test_stream_group_matches_topic() {
    let body = sample_search_response();
    let streams = parse_search_response(&body, "Moon");
    for s in &streams {
        assert_eq!(s.group, "Moon");
    }
}

#[test]
fn test_vod_type_is_always_vod() {
    let body = sample_search_response();
    let streams = parse_search_response(&body, "launch");
    for s in &streams {
        assert_eq!(s.vod_type, "vod");
    }
}

#[test]
fn test_logo_url_uses_thumb_pattern() {
    let body = sample_search_response();
    let streams = parse_search_response(&body, "launch");
    for s in &streams {
        let logo = s.logo.as_ref().unwrap();
        assert!(logo.contains("~thumb.jpg"), "logo should use thumb pattern");
        assert!(
            logo.contains(&s.id),
            "logo URL should contain the nasa_id"
        );
    }
}

#[test]
fn test_mp4_url_uses_mobile_pattern() {
    let body = sample_search_response();
    let streams = parse_search_response(&body, "launch");
    for s in &streams {
        assert!(
            s.url.contains("~mobile.mp4"),
            "url should use mobile.mp4 pattern"
        );
        assert!(
            s.url.contains(&s.id),
            "mp4 URL should contain the nasa_id"
        );
    }
}
