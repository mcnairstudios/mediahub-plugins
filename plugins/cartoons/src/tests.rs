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
                    "identifier": "popeye-the-sailor",
                    "title": "Popeye the Sailor Meets Sinbad",
                    "description": "A classic Popeye cartoon",
                    "year": "1936",
                    "creator": "Fleischer Studios"
                },
                {
                    "identifier": "superman-mechanical-monsters",
                    "title": "Superman: The Mechanical Monsters",
                    "year": 1941,
                    "creator": "Fleischer Studios"
                }
            ]
        }
    }"#;

    let docs = parse_search_response(json.as_bytes()).unwrap();
    assert_eq!(docs.len(), 2);
    assert_eq!(docs[0].identifier, "popeye-the-sailor");
    assert_eq!(docs[0].title, "Popeye the Sailor Meets Sinbad");
    assert_eq!(docs[0].description, Some("A classic Popeye cartoon".to_string()));
    assert_eq!(docs[1].identifier, "superman-mechanical-monsters");
}

#[test]
fn test_parse_search_response_empty() {
    let json = r#"{
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
                    "identifier": "some-cartoon"
                }
            ]
        }
    }"#;

    let docs = parse_search_response(json.as_bytes()).unwrap();
    assert_eq!(docs.len(), 1);
    assert_eq!(docs[0].identifier, "some-cartoon");
    assert_eq!(docs[0].title, "");
    assert!(docs[0].description.is_none());
    assert!(docs[0].year.is_none());
}

// ============================================================
// Year extraction tests
// ============================================================

#[test]
fn test_extract_year_string() {
    let v = Some(Value::String("1936".to_string()));
    assert_eq!(extract_year(&v), Some("1936".to_string()));
}

#[test]
fn test_extract_year_long_string() {
    let v = Some(Value::String("1936-01-01".to_string()));
    assert_eq!(extract_year(&v), Some("1936".to_string()));
}

#[test]
fn test_extract_year_number() {
    let v = Some(Value::Number(serde_json::Number::from(1941)));
    assert_eq!(extract_year(&v), Some("1941".to_string()));
}

#[test]
fn test_extract_year_none() {
    assert_eq!(extract_year(&None), None);
}

#[test]
fn test_extract_year_empty() {
    let v = Some(Value::String("".to_string()));
    assert_eq!(extract_year(&v), None);
}

// ============================================================
// Creator extraction tests
// ============================================================

#[test]
fn test_extract_creator_string() {
    let v = Some(Value::String("Fleischer Studios".to_string()));
    assert_eq!(extract_creator(&v), Some("Fleischer Studios".to_string()));
}

#[test]
fn test_extract_creator_array() {
    let v = Some(Value::Array(vec![Value::String("Max Fleischer".to_string())]));
    assert_eq!(extract_creator(&v), Some("Max Fleischer".to_string()));
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
// Group determination tests
// ============================================================

#[test]
fn test_determine_group_by_title() {
    assert_eq!(
        determine_group("Popeye the Sailor Meets Sinbad", &None, &None),
        "Popeye"
    );
    assert_eq!(
        determine_group("Superman: The Mechanical Monsters", &None, &None),
        "Superman"
    );
    assert_eq!(
        determine_group("Betty Boop in Snow White", &None, &None),
        "Betty Boop"
    );
    assert_eq!(
        determine_group("Felix the Cat Woos Whoopee", &None, &None),
        "Felix the Cat"
    );
}

#[test]
fn test_determine_group_by_description() {
    assert_eq!(
        determine_group(
            "Some Cartoon",
            &Some("A Popeye cartoon from 1935".to_string()),
            &None
        ),
        "Popeye"
    );
}

#[test]
fn test_determine_group_by_creator() {
    assert_eq!(
        determine_group(
            "Some Cartoon",
            &None,
            &Some("Fleischer Studios".to_string())
        ),
        "Fleischer Studios"
    );
}

#[test]
fn test_determine_group_creator_keyword_match() {
    assert_eq!(
        determine_group(
            "Some Cartoon",
            &None,
            &Some("fleischer".to_string())
        ),
        "Fleischer Studios"
    );
}

#[test]
fn test_determine_group_fallback_to_creator() {
    assert_eq!(
        determine_group(
            "Unknown Cartoon",
            &None,
            &Some("Walter Lantz".to_string())
        ),
        "Walter Lantz"
    );
}

#[test]
fn test_determine_group_fallback_default() {
    assert_eq!(
        determine_group("Unknown Cartoon", &None, &None),
        "Classic Cartoons"
    );
}

#[test]
fn test_determine_group_case_insensitive() {
    assert_eq!(
        determine_group("POPEYE THE SAILOR", &None, &None),
        "Popeye"
    );
    assert_eq!(
        determine_group("superman flies again", &None, &None),
        "Superman"
    );
}

// ============================================================
// Decade extraction tests
// ============================================================

#[test]
fn test_decade_from_year() {
    assert_eq!(decade_from_year(&Some("1936".to_string())), Some("1930s".to_string()));
    assert_eq!(decade_from_year(&Some("1941".to_string())), Some("1940s".to_string()));
    assert_eq!(decade_from_year(&Some("1929".to_string())), Some("1920s".to_string()));
}

#[test]
fn test_decade_from_year_none() {
    assert_eq!(decade_from_year(&None), None);
}

#[test]
fn test_decade_from_year_short() {
    assert_eq!(decade_from_year(&Some("19".to_string())), None);
}

// ============================================================
// URL construction tests
// ============================================================

#[test]
fn test_search_url() {
    let url = search_url();
    assert!(url.contains("collection:pdcartooncollection"));
    assert!(url.contains("rows=200"));
    assert!(url.contains("sort=downloads+desc"));
    assert!(url.contains("output=json"));
}

#[test]
fn test_video_url() {
    assert_eq!(
        video_url("popeye-the-sailor"),
        "https://archive.org/download/popeye-the-sailor/popeye-the-sailor.mp4"
    );
}

#[test]
fn test_thumbnail_url() {
    assert_eq!(
        thumbnail_url("popeye-the-sailor"),
        "https://archive.org/services/img/popeye-the-sailor"
    );
}

// ============================================================
// Doc to stream conversion tests
// ============================================================

#[test]
fn test_doc_to_stream_full() {
    let doc = SearchDoc {
        identifier: "popeye-the-sailor".to_string(),
        title: "Popeye the Sailor Meets Sinbad".to_string(),
        description: Some("A classic cartoon".to_string()),
        year: Some(Value::String("1936".to_string())),
        creator: Some(Value::String("Fleischer Studios".to_string())),
    };

    let stream = doc_to_stream(&doc);
    assert_eq!(stream.id, "popeye-the-sailor");
    assert_eq!(stream.name, "Popeye the Sailor Meets Sinbad");
    assert_eq!(stream.url, "https://archive.org/download/popeye-the-sailor/popeye-the-sailor.mp4");
    assert_eq!(stream.group, "Popeye");
    assert_eq!(stream.logo, Some("https://archive.org/services/img/popeye-the-sailor".to_string()));
    assert_eq!(stream.vod_type, "movie");
    assert_eq!(stream.year, Some("1936".to_string()));
    assert_eq!(stream.tags, Some(vec!["cartoon".to_string()]));
}

#[test]
fn test_doc_to_stream_minimal() {
    let doc = SearchDoc {
        identifier: "unknown-cartoon".to_string(),
        title: "".to_string(),
        description: None,
        year: None,
        creator: None,
    };

    let stream = doc_to_stream(&doc);
    assert_eq!(stream.id, "unknown-cartoon");
    assert_eq!(stream.name, "unknown-cartoon"); // fallback to identifier
    assert_eq!(stream.group, "Classic Cartoons"); // fallback
    assert_eq!(stream.year, None);
}

#[test]
fn test_doc_to_stream_superman() {
    let doc = SearchDoc {
        identifier: "superman-mechanical".to_string(),
        title: "Superman: The Mechanical Monsters".to_string(),
        description: None,
        year: Some(Value::Number(serde_json::Number::from(1941))),
        creator: Some(Value::String("Fleischer Studios".to_string())),
    };

    let stream = doc_to_stream(&doc);
    assert_eq!(stream.group, "Superman");
    assert_eq!(stream.year, Some("1941".to_string()));
}
