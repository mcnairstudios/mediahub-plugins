use super::*;

// ============================================================
// Search response parsing tests
// ============================================================

#[test]
fn test_parse_search_response_valid() {
    let json = r#"{
        "responseHeader": {"status": 0},
        "response": {
            "numFound": 2,
            "start": 0,
            "docs": [
                {
                    "identifier": "night-of-the-living-dead",
                    "title": "Night of the Living Dead",
                    "description": "A classic horror film",
                    "year": "1968",
                    "creator": "George A. Romero"
                },
                {
                    "identifier": "plan-9-from-outer-space",
                    "title": "Plan 9 from Outer Space",
                    "year": 1957
                }
            ]
        }
    }"#;

    let docs = parse_search_response(json.as_bytes()).unwrap();
    assert_eq!(docs.len(), 2);
    assert_eq!(docs[0].identifier, "night-of-the-living-dead");
    assert_eq!(docs[0].title, "Night of the Living Dead");
    assert_eq!(docs[0].description, Some("A classic horror film".to_string()));
    assert_eq!(docs[1].identifier, "plan-9-from-outer-space");
    assert_eq!(docs[1].title, "Plan 9 from Outer Space");
}

#[test]
fn test_parse_search_response_empty_docs() {
    let json = r#"{
        "responseHeader": {"status": 0},
        "response": {
            "numFound": 0,
            "start": 0,
            "docs": []
        }
    }"#;

    let docs = parse_search_response(json.as_bytes()).unwrap();
    assert!(docs.is_empty());
}

#[test]
fn test_parse_search_response_invalid_json() {
    let result = parse_search_response(b"not json");
    assert!(result.is_none());
}

#[test]
fn test_parse_search_response_missing_fields() {
    let json = r#"{
        "response": {
            "docs": [
                {
                    "identifier": "some-item"
                }
            ]
        }
    }"#;

    let docs = parse_search_response(json.as_bytes()).unwrap();
    assert_eq!(docs.len(), 1);
    assert_eq!(docs[0].identifier, "some-item");
    assert_eq!(docs[0].title, "");
    assert!(docs[0].description.is_none());
    assert!(docs[0].year.is_none());
}

// ============================================================
// Year extraction tests
// ============================================================

#[test]
fn test_extract_year_string() {
    let v = Some(Value::String("1968".to_string()));
    assert_eq!(extract_year(&v), Some("1968".to_string()));
}

#[test]
fn test_extract_year_long_string() {
    let v = Some(Value::String("1968-01-01".to_string()));
    assert_eq!(extract_year(&v), Some("1968".to_string()));
}

#[test]
fn test_extract_year_number() {
    let v = Some(Value::Number(serde_json::Number::from(1957)));
    assert_eq!(extract_year(&v), Some("1957".to_string()));
}

#[test]
fn test_extract_year_none() {
    assert_eq!(extract_year(&None), None);
}

#[test]
fn test_extract_year_empty_string() {
    let v = Some(Value::String("".to_string()));
    assert_eq!(extract_year(&v), None);
}

// ============================================================
// Creator extraction tests
// ============================================================

#[test]
fn test_extract_creator_string() {
    let v = Some(Value::String("George A. Romero".to_string()));
    assert_eq!(extract_creator(&v), Some("George A. Romero".to_string()));
}

#[test]
fn test_extract_creator_array() {
    let v = Some(Value::Array(vec![Value::String("Director Name".to_string())]));
    assert_eq!(extract_creator(&v), Some("Director Name".to_string()));
}

#[test]
fn test_extract_creator_none() {
    assert_eq!(extract_creator(&None), None);
}

#[test]
fn test_extract_creator_empty_string() {
    let v = Some(Value::String("".to_string()));
    assert_eq!(extract_creator(&v), None);
}

// ============================================================
// Heuristic URL tests
// ============================================================

#[test]
fn test_heuristic_video_url() {
    assert_eq!(
        heuristic_video_url("night-of-the-living-dead"),
        "https://archive.org/download/night-of-the-living-dead/night-of-the-living-dead.mp4"
    );
}

#[test]
fn test_heuristic_audio_url() {
    assert_eq!(
        heuristic_audio_url("gd1977-12-31"),
        "https://archive.org/download/gd1977-12-31/gd1977-12-31_vbr.mp3"
    );
}

#[test]
fn test_thumbnail_url() {
    assert_eq!(
        thumbnail_url("night-of-the-living-dead"),
        "https://archive.org/services/img/night-of-the-living-dead"
    );
}

// ============================================================
// Collection lookup tests
// ============================================================

#[test]
fn test_collection_display_name_known() {
    assert_eq!(collection_display_name("feature_films"), "Feature Films");
    assert_eq!(collection_display_name("GratefulDead"), "Grateful Dead");
    assert_eq!(collection_display_name("oldtimeradio"), "Old Time Radio");
    assert_eq!(collection_display_name("comedy_films"), "Comedy Films");
    assert_eq!(collection_display_name("horror_movies"), "Horror Movies");
}

#[test]
fn test_collection_display_name_unknown() {
    assert_eq!(collection_display_name("some_custom_collection"), "some_custom_collection");
}

#[test]
fn test_collection_media_type() {
    assert_eq!(collection_media_type("feature_films"), MediaType::Video);
    assert_eq!(collection_media_type("oldtimeradio"), MediaType::Audio);
    assert_eq!(collection_media_type("GratefulDead"), MediaType::Audio);
    assert_eq!(collection_media_type("comedy_films"), MediaType::Video);
    assert_eq!(collection_media_type("unknown"), MediaType::Video); // default
}

// ============================================================
// Config parsing tests
// ============================================================

#[test]
fn test_parse_collection_list() {
    let result = parse_collection_list("feature_films,prelinger,oldtimeradio");
    assert_eq!(result, vec!["feature_films", "prelinger", "oldtimeradio"]);
}

#[test]
fn test_parse_collection_list_with_spaces() {
    let result = parse_collection_list("feature_films , prelinger , oldtimeradio");
    assert_eq!(result, vec!["feature_films", "prelinger", "oldtimeradio"]);
}

#[test]
fn test_parse_collection_list_empty() {
    let result = parse_collection_list("");
    assert!(result.is_empty());
}

#[test]
fn test_parse_items_count_number() {
    let v = Value::Number(serde_json::Number::from(100));
    assert_eq!(parse_items_count(&v), 100);
}

#[test]
fn test_parse_items_count_string() {
    let v = Value::String("75".to_string());
    assert_eq!(parse_items_count(&v), 75);
}

#[test]
fn test_parse_items_count_invalid() {
    let v = Value::Null;
    assert_eq!(parse_items_count(&v), 40); // default
}

// ============================================================
// URL construction tests
// ============================================================

#[test]
fn test_search_url() {
    let url = search_url("feature_films", "downloads desc", 50);
    assert!(url.contains("collection:feature_films"));
    assert!(url.contains("rows=50"));
    assert!(url.contains("sort=downloads+desc"));
    assert!(url.contains("output=json"));
    assert!(url.contains("fl[]=identifier"));
    assert!(url.contains("fl[]=title"));
    assert!(url.contains("fl[]=year"));
    assert!(url.contains("fl[]=creator"));
}

#[test]
fn test_search_query_url() {
    let collections = vec!["feature_films".to_string(), "prelinger".to_string()];
    let url = search_query_url("horror", &collections, "downloads desc", 30);
    assert!(url.contains("q=horror"));
    assert!(url.contains("collection:feature_films"));
    assert!(url.contains("collection:prelinger"));
    assert!(url.contains("rows=30"));
}

#[test]
fn test_search_query_url_no_collections() {
    let url = search_query_url("horror", &[], "downloads desc", 30);
    assert!(url.contains("q=horror"));
    assert!(!url.contains("collection:"));
}

// ============================================================
// doc_to_stream tests (replaces metadata-based tests)
// ============================================================

#[test]
fn test_doc_to_stream_video() {
    let doc = SearchDoc {
        identifier: "night-of-the-living-dead".to_string(),
        title: "Night of the Living Dead".to_string(),
        description: Some("A classic horror film".to_string()),
        year: Some(Value::String("1968".to_string())),
        creator: Some(Value::String("George A. Romero".to_string())),
    };

    let stream = doc_to_stream(&doc, "Feature Films", MediaType::Video);
    assert_eq!(stream.id, "night-of-the-living-dead");
    assert_eq!(stream.name, "Night of the Living Dead");
    assert_eq!(stream.group, "Feature Films");
    assert_eq!(stream.year, Some("1968".to_string()));
    assert_eq!(stream.vod_type, Some("movie".to_string()));
    assert!(stream.url.contains("night-of-the-living-dead.mp4"));
    assert!(stream.logo.unwrap().contains("night-of-the-living-dead"));
    assert_eq!(stream.tags, Some(vec!["video".to_string()]));
}

#[test]
fn test_doc_to_stream_audio() {
    let doc = SearchDoc {
        identifier: "gd1977-12-31".to_string(),
        title: "Grateful Dead Live 1977-12-31".to_string(),
        description: None,
        year: Some(Value::Number(serde_json::Number::from(1977))),
        creator: None,
    };

    let stream = doc_to_stream(&doc, "Grateful Dead", MediaType::Audio);
    assert_eq!(stream.id, "gd1977-12-31");
    assert_eq!(stream.group, "Grateful Dead");
    assert_eq!(stream.year, Some("1977".to_string()));
    assert_eq!(stream.vod_type, None);
    assert!(stream.url.contains("_vbr.mp3"));
    assert_eq!(stream.tags, Some(vec!["audio".to_string()]));
}

#[test]
fn test_doc_to_stream_uses_identifier_when_no_title() {
    let doc = SearchDoc {
        identifier: "some-item".to_string(),
        title: "".to_string(),
        description: None,
        year: None,
        creator: None,
    };

    let stream = doc_to_stream(&doc, "Test Group", MediaType::Video);
    assert_eq!(stream.name, "some-item");
}

#[test]
fn test_doc_to_stream_no_year() {
    let doc = SearchDoc {
        identifier: "mystery-film".to_string(),
        title: "Mystery Film".to_string(),
        description: None,
        year: None,
        creator: None,
    };

    let stream = doc_to_stream(&doc, "Feature Films", MediaType::Video);
    assert_eq!(stream.year, None);
}

// ============================================================
// Default collections have enough variety
// ============================================================

#[test]
fn test_default_collections_has_many_groups() {
    let defaults = parse_collection_list(&default_collections());
    assert!(
        defaults.len() >= 10,
        "default collections should have 10+ groups, got {}",
        defaults.len()
    );
}

#[test]
fn test_known_collections_has_many_entries() {
    assert!(
        KNOWN_COLLECTIONS.len() >= 10,
        "should have 10+ known collections, got {}",
        KNOWN_COLLECTIONS.len()
    );
}
