use super::*;
use serde_json::json;

// ============================================================
// Year extraction tests
// ============================================================

#[test]
fn test_extract_year_from_number() {
    let val = Some(json!(1931));
    assert_eq!(extract_year(&val), Some(1931));
}

#[test]
fn test_extract_year_from_string() {
    let val = Some(json!("1942"));
    assert_eq!(extract_year(&val), Some(1942));
}

#[test]
fn test_extract_year_from_date_string() {
    let val = Some(json!("1955-01-01T00:00:00Z"));
    assert_eq!(extract_year(&val), Some(1955));
}

#[test]
fn test_extract_year_none() {
    assert_eq!(extract_year(&None), None);
}

#[test]
fn test_extract_year_null_value() {
    let val = Some(json!(null));
    assert_eq!(extract_year(&val), None);
}

// ============================================================
// Decade grouping tests
// ============================================================

#[test]
fn test_year_to_decade_1920s() {
    assert_eq!(year_to_decade(1920), "1920s");
    assert_eq!(year_to_decade(1929), "1920s");
}

#[test]
fn test_year_to_decade_1930s() {
    assert_eq!(year_to_decade(1931), "1930s");
    assert_eq!(year_to_decade(1939), "1930s");
}

#[test]
fn test_year_to_decade_1950s() {
    assert_eq!(year_to_decade(1955), "1950s");
}

#[test]
fn test_year_to_decade_2000s() {
    assert_eq!(year_to_decade(2005), "2000s");
}

// ============================================================
// Year from title tests
// ============================================================

#[test]
fn test_year_from_title_with_parens() {
    assert_eq!(year_from_title("Millie (1931)"), Some(1931));
}

#[test]
fn test_year_from_title_no_year() {
    assert_eq!(year_from_title("Some Movie"), None);
}

#[test]
fn test_year_from_title_invalid_year() {
    assert_eq!(year_from_title("Movie (abcd)"), None);
}

#[test]
fn test_year_from_title_with_extra_text() {
    assert_eq!(year_from_title("Night of the Living Dead (1968) restored"), Some(1968));
}

#[test]
fn test_year_from_title_multiple_parens() {
    assert_eq!(year_from_title("Movie (drama) (1945)"), Some(1945));
}

// ============================================================
// Heuristic URL tests
// ============================================================

#[test]
fn test_heuristic_video_url() {
    assert_eq!(
        heuristic_video_url("night-of-living-dead"),
        "https://archive.org/download/night-of-living-dead/night-of-living-dead.mp4"
    );
}

#[test]
fn test_heuristic_video_url_complex_id() {
    assert_eq!(
        heuristic_video_url("charade_1963"),
        "https://archive.org/download/charade_1963/charade_1963.mp4"
    );
}

// ============================================================
// Search response parsing tests
// ============================================================

#[test]
fn test_parse_search_response_valid() {
    let data = json!({
        "responseHeader": {"status": 0},
        "response": {
            "numFound": 2,
            "start": 0,
            "docs": [
                {
                    "identifier": "night-of-living-dead",
                    "title": "Night of the Living Dead",
                    "description": "A horror classic",
                    "year": "1968"
                },
                {
                    "identifier": "charade-1963",
                    "title": "Charade",
                    "year": 1963
                }
            ]
        }
    });
    let bytes = serde_json::to_vec(&data).unwrap();
    let results = parse_search_response(&bytes);
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].identifier, "night-of-living-dead");
    assert_eq!(results[0].title, Some("Night of the Living Dead".to_string()));
    assert_eq!(results[1].identifier, "charade-1963");
    assert_eq!(results[1].year, Some(json!(1963)));
}

#[test]
fn test_parse_search_response_empty() {
    let data = json!({
        "response": {
            "numFound": 0,
            "start": 0,
            "docs": []
        }
    });
    let bytes = serde_json::to_vec(&data).unwrap();
    let results = parse_search_response(&bytes);
    assert_eq!(results.len(), 0);
}

#[test]
fn test_parse_search_response_invalid_json() {
    let bytes = b"not json at all";
    let results = parse_search_response(bytes);
    assert_eq!(results.len(), 0);
}

#[test]
fn test_parse_search_response_missing_optional_fields() {
    let data = json!({
        "response": {
            "numFound": 1,
            "start": 0,
            "docs": [
                {
                    "identifier": "some-movie"
                }
            ]
        }
    });
    let bytes = serde_json::to_vec(&data).unwrap();
    let results = parse_search_response(&bytes);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].identifier, "some-movie");
    assert_eq!(results[0].title, None);
    assert_eq!(results[0].year, None);
    assert_eq!(results[0].description, None);
}

// ============================================================
// Stream building tests (now using heuristic URLs)
// ============================================================

#[test]
fn test_build_stream_from_search_with_year() {
    let item = SearchResult {
        identifier: "test-film".to_string(),
        title: Some("Test Film".to_string()),
        description: None,
        year: Some(json!(1955)),
        creator: None,
    };

    let stream = build_stream_from_search(&item, "Unknown Decade");
    assert_eq!(stream.id, "test-film");
    assert_eq!(stream.name, "Test Film (1955)");
    assert_eq!(stream.group, "1950s");
    assert_eq!(stream.year, Some("1955".to_string()));
    assert_eq!(stream.vod_type, "movie");
    assert!(stream.url.contains("test-film.mp4"));
    assert_eq!(stream.logo, Some("https://archive.org/services/img/test-film".to_string()));
}

#[test]
fn test_build_stream_from_search_no_year() {
    let item = SearchResult {
        identifier: "test-film".to_string(),
        title: Some("Test Film".to_string()),
        description: None,
        year: None,
        creator: None,
    };

    let stream = build_stream_from_search(&item, "Unknown Decade");
    assert_eq!(stream.group, "Unknown Decade");
    assert_eq!(stream.year, None);
}

#[test]
fn test_build_stream_from_search_year_from_title_fallback() {
    let item = SearchResult {
        identifier: "test-film".to_string(),
        title: Some("Test Film (1945)".to_string()),
        description: None,
        year: None,
        creator: None,
    };

    let stream = build_stream_from_search(&item, "Unknown Decade");
    assert_eq!(stream.group, "1940s");
    assert_eq!(stream.year, Some("1945".to_string()));
    // Title already has the year, so it should not be duplicated
    assert_eq!(stream.name, "Test Film (1945)");
}

#[test]
fn test_build_stream_from_search_no_title() {
    let item = SearchResult {
        identifier: "mystery-film-123".to_string(),
        title: None,
        description: None,
        year: None,
        creator: None,
    };

    let stream = build_stream_from_search(&item, "Other");
    assert_eq!(stream.name, "mystery-film-123");
    assert_eq!(stream.id, "mystery-film-123");
}

// ============================================================
// URL encoding tests
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
fn test_url_encode_parens() {
    assert_eq!(url_encode("Movie (1931).mp4"), "Movie%20%281931%29.mp4");
}

#[test]
fn test_url_encode_preserves_safe_chars() {
    assert_eq!(url_encode("file-name_v2.mp4"), "file-name_v2.mp4");
}

// ============================================================
// Config parsing tests
// ============================================================

#[test]
fn test_parse_max_films_default() {
    let config = serde_json::Map::new();
    assert_eq!(parse_max_films(&config), 300);
}

#[test]
fn test_parse_max_films_number() {
    let mut config = serde_json::Map::new();
    config.insert("max_films".to_string(), json!(100));
    assert_eq!(parse_max_films(&config), 100);
}

#[test]
fn test_parse_max_films_string() {
    let mut config = serde_json::Map::new();
    config.insert("max_films".to_string(), json!("200"));
    assert_eq!(parse_max_films(&config), 200);
}

// ============================================================
// Decade grouping integration test
// ============================================================

#[test]
fn test_streams_grouped_by_decade() {
    let years_and_expected = vec![
        (1922, "1920s"),
        (1935, "1930s"),
        (1941, "1940s"),
        (1957, "1950s"),
        (1963, "1960s"),
    ];

    for (year, expected_decade) in years_and_expected {
        let item = SearchResult {
            identifier: "film".to_string(),
            title: Some("Film".to_string()),
            description: None,
            year: Some(json!(year)),
            creator: None,
        };
        let stream = build_stream_from_search(&item, "Other");
        assert_eq!(stream.group, expected_decade, "year {} should be in {}", year, expected_decade);
    }
}

// ============================================================
// Decade queries coverage test
// ============================================================

#[test]
fn test_decade_queries_has_many_entries() {
    assert!(
        DECADE_QUERIES.len() >= 5,
        "should have 5+ decade queries for variety, got {}",
        DECADE_QUERIES.len()
    );
}

#[test]
fn test_decade_queries_total_rows_reasonable() {
    let total: u32 = DECADE_QUERIES.iter().map(|&(_, _, rows)| rows).sum();
    assert!(
        total >= 200,
        "total rows across queries should be >= 200 for good coverage, got {}",
        total
    );
    assert!(
        total <= 600,
        "total rows across queries should be <= 600 to stay efficient, got {}",
        total
    );
}
