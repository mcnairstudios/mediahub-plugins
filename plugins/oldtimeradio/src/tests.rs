use super::*;

// ============================================================
// Search response parsing tests
// ============================================================

#[test]
fn test_parse_search_response_valid() {
    let json = r#"{
        "responseHeader": {"status": 0},
        "response": {
            "numFound": 3,
            "start": 0,
            "docs": [
                {
                    "identifier": "OTRR_Gunsmoke_Singles",
                    "title": "Gunsmoke",
                    "description": "Classic western radio drama",
                    "creator": "CBS Radio"
                },
                {
                    "identifier": "OTRR_Dragnet_Singles",
                    "title": "Dragnet",
                    "description": "Police procedural drama",
                    "creator": "NBC Radio"
                },
                {
                    "identifier": "LightsOut_Episode1",
                    "title": "Lights Out - Episode 1",
                    "description": "",
                    "creator": ""
                }
            ]
        }
    }"#;

    let docs = parse_search_response(json.as_bytes()).unwrap();
    assert_eq!(docs.len(), 3);
    assert_eq!(docs[0].identifier, "OTRR_Gunsmoke_Singles");
    assert_eq!(docs[0].title, "Gunsmoke");
    assert_eq!(docs[0].creator, "CBS Radio");
    assert_eq!(docs[1].identifier, "OTRR_Dragnet_Singles");
    assert_eq!(docs[2].creator, "");
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
                    "identifier": "test_item"
                }
            ]
        }
    }"#;

    let docs = parse_search_response(json.as_bytes()).unwrap();
    assert_eq!(docs.len(), 1);
    assert_eq!(docs[0].identifier, "test_item");
    assert_eq!(docs[0].title, "");
    assert_eq!(docs[0].description, "");
    assert_eq!(docs[0].creator, "");
}

// ============================================================
// Percent decode tests
// ============================================================

#[test]
fn test_percent_decode_spaces() {
    assert_eq!(percent_decode("hello%20world"), "hello world");
}

#[test]
fn test_percent_decode_multiple() {
    assert_eq!(percent_decode("a%20b%20c%20d"), "a b c d");
}

#[test]
fn test_percent_decode_no_encoding() {
    assert_eq!(percent_decode("plain text"), "plain text");
}

#[test]
fn test_percent_decode_special_chars() {
    assert_eq!(percent_decode("%28parentheses%29"), "(parentheses)");
}

#[test]
fn test_percent_decode_incomplete_sequence() {
    assert_eq!(percent_decode("abc%2"), "abc%2");
}

// ============================================================
// URL encoding tests
// ============================================================

#[test]
fn test_url_encode_path_spaces() {
    assert_eq!(
        url_encode_path("Billy the Kid.mp3"),
        "Billy%20the%20Kid.mp3"
    );
}

#[test]
fn test_url_encode_path_hash() {
    assert_eq!(url_encode_path("track#1.mp3"), "track%231.mp3");
}

#[test]
fn test_url_encode_path_no_encoding_needed() {
    assert_eq!(url_encode_path("simple.mp3"), "simple.mp3");
}

// ============================================================
// Group derivation tests -- now uses show-name matching
// ============================================================

#[test]
fn test_derive_group_matches_gunsmoke_in_identifier() {
    let doc = IASearchDoc {
        identifier: "OTRR_Gunsmoke_Singles".to_string(),
        title: "Some Title".to_string(),
        description: String::new(),
        creator: "CBS Radio".to_string(),
    };
    assert_eq!(derive_group(&doc), "Gunsmoke");
}

#[test]
fn test_derive_group_matches_dragnet_in_identifier() {
    let doc = IASearchDoc {
        identifier: "OTRR_Dragnet_Singles".to_string(),
        title: "Dragnet Collection".to_string(),
        description: String::new(),
        creator: "NBC Radio".to_string(),
    };
    assert_eq!(derive_group(&doc), "Dragnet");
}

#[test]
fn test_derive_group_matches_suspense_in_title() {
    let doc = IASearchDoc {
        identifier: "some_random_id".to_string(),
        title: "Suspense - The Killer".to_string(),
        description: String::new(),
        creator: String::new(),
    };
    assert_eq!(derive_group(&doc), "Suspense");
}

#[test]
fn test_derive_group_matches_shadow() {
    let doc = IASearchDoc {
        identifier: "TheShadow_1938_collection".to_string(),
        title: "The Shadow Collection".to_string(),
        description: String::new(),
        creator: String::new(),
    };
    assert_eq!(derive_group(&doc), "The Shadow");
}

#[test]
fn test_derive_group_falls_back_to_creator() {
    let doc = IASearchDoc {
        identifier: "unknown_show_123".to_string(),
        title: "Some Random Title".to_string(),
        description: String::new(),
        creator: "ABC Radio".to_string(),
    };
    assert_eq!(derive_group(&doc), "ABC Radio");
}

#[test]
fn test_derive_group_falls_back_to_title() {
    let doc = IASearchDoc {
        identifier: "unknown_show_123".to_string(),
        title: "My Special Show".to_string(),
        description: String::new(),
        creator: String::new(),
    };
    assert_eq!(derive_group(&doc), "My Special Show");
}

#[test]
fn test_derive_group_no_info() {
    let doc = IASearchDoc {
        identifier: "unknown_123".to_string(),
        title: String::new(),
        description: String::new(),
        creator: String::new(),
    };
    assert_eq!(derive_group(&doc), "Miscellaneous");
}

// ============================================================
// Stream URL tests
// ============================================================

#[test]
fn test_make_stream_url() {
    assert_eq!(
        make_stream_url("OTRR_Gunsmoke_Singles"),
        "https://archive.org/download/OTRR_Gunsmoke_Singles/OTRR_Gunsmoke_Singles_vbr.mp3"
    );
}

// ============================================================
// doc_to_stream tests
// ============================================================

#[test]
fn test_doc_to_stream_basic() {
    let doc = IASearchDoc {
        identifier: "OTRR_Gunsmoke_Singles".to_string(),
        title: "Gunsmoke Collection".to_string(),
        description: "Classic western".to_string(),
        creator: "CBS Radio".to_string(),
    };

    let stream = doc_to_stream(&doc);
    assert_eq!(stream.id, "OTRR_Gunsmoke_Singles");
    assert_eq!(stream.name, "Gunsmoke Collection");
    assert_eq!(stream.group, "Gunsmoke"); // matched by show name
    assert_eq!(stream.vod_type, "movie");
    assert!(stream.url.contains("OTRR_Gunsmoke_Singles"));
    assert!(stream.url.contains("_vbr.mp3"));
    assert_eq!(
        stream.logo.as_deref(),
        Some("https://archive.org/services/img/OTRR_Gunsmoke_Singles")
    );
    let tags = stream.tags.as_ref().unwrap();
    assert!(tags.contains(&"radio".to_string()));
    assert!(tags.contains(&"classic".to_string()));
}

#[test]
fn test_doc_to_stream_uses_identifier_when_no_title() {
    let doc = IASearchDoc {
        identifier: "mystery_item".to_string(),
        title: String::new(),
        description: String::new(),
        creator: String::new(),
    };

    let stream = doc_to_stream(&doc);
    assert_eq!(stream.name, "mystery_item");
}

#[test]
fn test_doc_to_stream_unmatched_show_uses_creator() {
    let doc = IASearchDoc {
        identifier: "random_item_123".to_string(),
        title: "Random Show".to_string(),
        description: String::new(),
        creator: "Some Network".to_string(),
    };

    let stream = doc_to_stream(&doc);
    assert_eq!(stream.group, "Some Network");
}

// ============================================================
// Known shows coverage test
// ============================================================

#[test]
fn test_known_shows_has_many_entries() {
    assert!(
        KNOWN_SHOWS.len() >= 20,
        "should have 20+ known show patterns, got {}",
        KNOWN_SHOWS.len()
    );
}

#[test]
fn test_search_queries_defined() {
    assert!(
        SEARCH_QUERIES.len() >= 4,
        "should have 4+ search queries for variety, got {}",
        SEARCH_QUERIES.len()
    );
}
