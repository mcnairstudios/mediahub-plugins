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
    // rfind picks the last parenthesized group
    assert_eq!(year_from_title("Movie (drama) (1945)"), Some(1945));
}

// ============================================================
// MP4 file selection tests
// ============================================================

#[test]
fn test_select_best_mp4_prefers_h264() {
    let files = vec![
        json!({
            "name": "movie.mp4",
            "format": "MPEG4",
            "size": "500000000"
        }),
        json!({
            "name": "movie_h264.mp4",
            "format": "h.264",
            "size": "300000000"
        }),
    ];
    assert_eq!(select_best_mp4(&files), Some("movie_h264.mp4".to_string()));
}

#[test]
fn test_select_best_mp4_falls_back_to_mpeg4() {
    let files = vec![
        json!({
            "name": "movie.mp4",
            "format": "MPEG4",
            "size": "500000000"
        }),
        json!({
            "name": "thumbnail.jpg",
            "format": "JPEG",
            "size": "50000"
        }),
    ];
    assert_eq!(select_best_mp4(&files), Some("movie.mp4".to_string()));
}

#[test]
fn test_select_best_mp4_no_video() {
    let files = vec![
        json!({
            "name": "metadata.xml",
            "format": "Metadata",
            "size": "1000"
        }),
        json!({
            "name": "thumbnail.jpg",
            "format": "JPEG",
            "size": "50000"
        }),
    ];
    assert_eq!(select_best_mp4(&files), None);
}

#[test]
fn test_select_best_mp4_picks_largest_h264() {
    let files = vec![
        json!({
            "name": "small.mp4",
            "format": "h.264",
            "size": "100000"
        }),
        json!({
            "name": "large.mp4",
            "format": "h.264",
            "size": "900000000"
        }),
    ];
    assert_eq!(select_best_mp4(&files), Some("large.mp4".to_string()));
}

#[test]
fn test_select_best_mp4_size_as_number() {
    let files = vec![
        json!({
            "name": "movie.mp4",
            "format": "h.264",
            "size": 500000000_u64
        }),
    ];
    assert_eq!(select_best_mp4(&files), Some("movie.mp4".to_string()));
}

#[test]
fn test_select_best_mp4_h264_ia_variant() {
    let files = vec![
        json!({
            "name": "movie.mp4",
            "format": "h.264 IA",
            "size": "300000000"
        }),
    ];
    assert_eq!(select_best_mp4(&files), Some("movie.mp4".to_string()));
}

#[test]
fn test_select_best_mp4_empty_files() {
    let files: Vec<Value> = vec![];
    assert_eq!(select_best_mp4(&files), None);
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
// Stream building tests
// ============================================================

#[test]
fn test_build_stream_from_metadata_success() {
    let files = vec![
        json!({"name": "movie.mp4", "format": "h.264", "size": "500000000"}),
    ];
    let year = Some(json!(1955));

    let stream = build_stream_from_metadata("test-film", "Test Film", &year, &files);
    assert!(stream.is_some());

    let s = stream.unwrap();
    assert_eq!(s.id, "test-film");
    assert_eq!(s.name, "Test Film (1955)");
    assert_eq!(s.url, "https://archive.org/download/test-film/movie.mp4");
    assert_eq!(s.group, "1950s");
    assert_eq!(s.year, Some("1955".to_string()));
    assert_eq!(s.vod_type, "movie");
    assert_eq!(s.logo, Some("https://archive.org/services/img/test-film".to_string()));
}

#[test]
fn test_build_stream_from_metadata_no_mp4() {
    let files = vec![
        json!({"name": "meta.xml", "format": "Metadata", "size": "1000"}),
    ];
    let year = Some(json!(1940));

    let stream = build_stream_from_metadata("test-film", "Test Film", &year, &files);
    assert!(stream.is_none());
}

#[test]
fn test_build_stream_from_metadata_no_year() {
    let files = vec![
        json!({"name": "movie.mp4", "format": "h.264", "size": "500000000"}),
    ];

    let stream = build_stream_from_metadata("test-film", "Test Film", &None, &files);
    assert!(stream.is_some());

    let s = stream.unwrap();
    assert_eq!(s.group, "Unknown Decade");
    assert_eq!(s.year, None);
}

#[test]
fn test_build_stream_from_metadata_year_from_title_fallback() {
    let files = vec![
        json!({"name": "movie.mp4", "format": "h.264", "size": "500000000"}),
    ];

    let stream = build_stream_from_metadata("test-film", "Test Film (1945)", &None, &files);
    assert!(stream.is_some());

    let s = stream.unwrap();
    assert_eq!(s.group, "1940s");
    assert_eq!(s.year, Some("1945".to_string()));
    // Title already has the year, so it should not be duplicated
    assert_eq!(s.name, "Test Film (1945)");
}

#[test]
fn test_build_stream_url_encoding() {
    let files = vec![
        json!({"name": "The Great Film (1940).mp4", "format": "h.264", "size": "500000000"}),
    ];
    let year = Some(json!(1940));

    let stream = build_stream_from_metadata("great-film", "The Great Film", &year, &files);
    assert!(stream.is_some());

    let s = stream.unwrap();
    assert!(s.url.contains("The%20Great%20Film%20%281940%29.mp4"));
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
    assert_eq!(parse_max_films(&config), 50);
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
    let files = vec![
        json!({"name": "m.mp4", "format": "h.264", "size": "100"}),
    ];

    let years_and_expected = vec![
        (1922, "1920s"),
        (1935, "1930s"),
        (1941, "1940s"),
        (1957, "1950s"),
        (1963, "1960s"),
    ];

    for (year, expected_decade) in years_and_expected {
        let stream = build_stream_from_metadata(
            "film",
            "Film",
            &Some(json!(year)),
            &files,
        )
        .unwrap();
        assert_eq!(stream.group, expected_decade, "year {} should be in {}", year, expected_decade);
    }
}
