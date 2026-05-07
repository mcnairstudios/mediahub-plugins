use super::*;

// ============================================================
// Title parsing tests
// ============================================================

#[test]
fn test_parse_title_en_dash() {
    let (name, group) = parse_title("La Traviata Verdi \u{2013} Royal Opera House");
    assert_eq!(name, "La Traviata Verdi");
    assert_eq!(group, "Royal Opera House");
}

#[test]
fn test_parse_title_double_hyphen() {
    let (name, group) = parse_title("The Corsair -- Estonian National Ballet");
    assert_eq!(name, "The Corsair");
    assert_eq!(group, "Estonian National Ballet");
}

#[test]
fn test_parse_title_single_hyphen_fallback() {
    let (name, group) = parse_title("Tosca Puccini - Wiener Staatsoper");
    assert_eq!(name, "Tosca Puccini");
    assert_eq!(group, "Wiener Staatsoper");
}

#[test]
fn test_parse_title_no_delimiter() {
    let (name, group) = parse_title("OperaVision Gala 2024");
    assert_eq!(name, "OperaVision Gala 2024");
    assert_eq!(group, "OperaVision");
}

#[test]
fn test_parse_title_empty() {
    let (name, group) = parse_title("");
    assert_eq!(name, "");
    assert_eq!(group, "OperaVision");
}

#[test]
fn test_parse_title_multiple_hyphens_uses_last() {
    let (name, group) = parse_title("Carmen - Suite No. 1 - Opera de Paris");
    assert_eq!(name, "Carmen - Suite No. 1");
    assert_eq!(group, "Opera de Paris");
}

#[test]
fn test_parse_title_en_dash_takes_priority_over_hyphen() {
    let (name, group) = parse_title("Swan Lake - Act II Tchaikovsky \u{2013} Bolshoi Theatre");
    assert_eq!(name, "Swan Lake - Act II Tchaikovsky");
    assert_eq!(group, "Bolshoi Theatre");
}

#[test]
fn test_parse_title_trims_whitespace() {
    let (name, group) = parse_title("  Aida Verdi  \u{2013}  Teatro alla Scala  ");
    assert_eq!(name, "Aida Verdi");
    assert_eq!(group, "Teatro alla Scala");
}

// ============================================================
// Tag detection tests
// ============================================================

#[test]
fn test_detect_tags_ballet() {
    let tags = detect_tags("The Corsair -- Estonian National Ballet");
    assert!(tags.contains(&"ballet".to_string()));
}

#[test]
fn test_detect_tags_opera() {
    let tags = detect_tags("La Traviata -- Royal Opera House");
    assert!(tags.contains(&"opera".to_string()));
}

#[test]
fn test_detect_tags_concert() {
    let tags = detect_tags("New Year's Concert 2024 -- Vienna Philharmonic");
    assert!(tags.contains(&"concert".to_string()));
}

#[test]
fn test_detect_tags_default_performance() {
    let tags = detect_tags("Something Unusual -- Some Theatre");
    assert_eq!(tags, vec!["performance".to_string()]);
}

#[test]
fn test_detect_tags_multiple() {
    let tags = detect_tags("Opera Ballet Gala Concert");
    assert!(tags.contains(&"ballet".to_string()));
    assert!(tags.contains(&"opera".to_string()));
    assert!(tags.contains(&"concert".to_string()));
}

// ============================================================
// YouTube URL construction tests
// ============================================================

#[test]
fn test_build_youtube_url() {
    assert_eq!(
        build_youtube_url("LMn6rDScs_Y"),
        "https://www.youtube.com/watch?v=LMn6rDScs_Y"
    );
}

#[test]
fn test_build_thumbnail_url() {
    assert_eq!(
        build_thumbnail_url("LMn6rDScs_Y"),
        "https://i.ytimg.com/vi/LMn6rDScs_Y/hqdefault.jpg"
    );
}

// ============================================================
// Video ID extraction from Piped URL path
// ============================================================

#[test]
fn test_extract_video_id_basic() {
    let id = extract_video_id_from_path("/watch?v=LMn6rDScs_Y");
    assert_eq!(id, Some("LMn6rDScs_Y".to_string()));
}

#[test]
fn test_extract_video_id_with_extra_params() {
    let id = extract_video_id_from_path("/watch?v=abc123&list=PLxyz");
    assert_eq!(id, Some("abc123".to_string()));
}

#[test]
fn test_extract_video_id_missing() {
    let id = extract_video_id_from_path("/channel/UCxyz");
    assert_eq!(id, None);
}

#[test]
fn test_extract_video_id_empty_value() {
    let id = extract_video_id_from_path("/watch?v=");
    assert_eq!(id, None);
}

// ============================================================
// Piped JSON parsing tests
// ============================================================

fn sample_piped_json() -> String {
    r#"{
        "name": "OperaVision",
        "relatedStreams": [
            {
                "url": "/watch?v=abc123DEF_0",
                "title": "La Traviata Verdi \u2013 Royal Opera House",
                "thumbnail": "https://proxy.example/vi/abc123DEF_0/hqdefault.jpg",
                "duration": 5400
            },
            {
                "url": "/watch?v=xyz789GHI_1",
                "title": "Swan Lake Tchaikovsky \u2013 Bolshoi Theatre",
                "thumbnail": "https://proxy.example/vi/xyz789GHI_1/hqdefault.jpg",
                "duration": 7200
            },
            {
                "url": "/watch?v=qrs456JKL_2",
                "title": "Some Trailer",
                "thumbnail": "https://proxy.example/vi/qrs456JKL_2/hqdefault.jpg",
                "duration": 120
            }
        ]
    }"#.to_string()
}

#[test]
fn test_parse_piped_json_count() {
    let json = sample_piped_json();
    let items = parse_piped_json(&json);
    assert_eq!(items.len(), 3);
}

#[test]
fn test_parse_piped_json_first_entry() {
    let json = sample_piped_json();
    let items = parse_piped_json(&json);
    assert_eq!(items[0].0, "abc123DEF_0");
    assert!(items[0].1.contains("La Traviata"));
}

#[test]
fn test_parse_piped_json_second_entry() {
    let json = sample_piped_json();
    let items = parse_piped_json(&json);
    assert_eq!(items[1].0, "xyz789GHI_1");
    assert!(items[1].1.contains("Swan Lake"));
}

#[test]
fn test_parse_piped_json_empty_streams() {
    let json = r#"{"name": "OperaVision", "relatedStreams": []}"#;
    let items = parse_piped_json(json);
    assert!(items.is_empty());
}

#[test]
fn test_parse_piped_json_missing_related_streams() {
    let json = r#"{"name": "OperaVision"}"#;
    let items = parse_piped_json(json);
    assert!(items.is_empty());
}

#[test]
fn test_parse_piped_json_invalid_json() {
    let items = parse_piped_json("not json at all");
    assert!(items.is_empty());
}

#[test]
fn test_parse_piped_json_deduplicates() {
    let json = r#"{
        "name": "OperaVision",
        "relatedStreams": [
            {"url": "/watch?v=abc123", "title": "Title A"},
            {"url": "/watch?v=abc123", "title": "Title A Again"},
            {"url": "/watch?v=def456", "title": "Title B"}
        ]
    }"#;
    let items = parse_piped_json(json);
    assert_eq!(items.len(), 2);
    assert_eq!(items[0].0, "abc123");
    assert_eq!(items[1].0, "def456");
}

#[test]
fn test_parse_piped_json_missing_url_skips_entry() {
    let json = r#"{
        "name": "OperaVision",
        "relatedStreams": [
            {"title": "No URL here"},
            {"url": "/watch?v=valid1", "title": "Has URL"}
        ]
    }"#;
    let items = parse_piped_json(json);
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].0, "valid1");
}

#[test]
fn test_parse_piped_json_missing_title_defaults_empty() {
    let json = r#"{
        "name": "OperaVision",
        "relatedStreams": [
            {"url": "/watch?v=notitle1"}
        ]
    }"#;
    let items = parse_piped_json(json);
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].0, "notitle1");
    assert_eq!(items[0].1, "");
}

// ============================================================
// Stream building tests
// ============================================================

#[test]
fn test_build_streams_basic() {
    let items = vec![
        ("LMn6rDScs_Y".to_string(), "The Corsair -- Estonian National Ballet".to_string()),
    ];
    let streams = build_streams(&items);
    assert_eq!(streams.len(), 1);

    let s = &streams[0];
    assert_eq!(s.id, "LMn6rDScs_Y");
    assert_eq!(s.name, "The Corsair");
    assert_eq!(s.group, "Estonian National Ballet");
    assert_eq!(s.url, "https://www.youtube.com/watch?v=LMn6rDScs_Y");
    assert_eq!(s.logo, Some("https://i.ytimg.com/vi/LMn6rDScs_Y/hqdefault.jpg".to_string()));
    assert_eq!(s.vod_type, "youtube");
    assert!(s.tags.as_ref().unwrap().contains(&"ballet".to_string()));
}

#[test]
fn test_build_streams_empty_title_uses_video_id() {
    let items = vec![
        ("abc123".to_string(), "".to_string()),
    ];
    let streams = build_streams(&items);
    assert_eq!(streams[0].name, "abc123");
    assert_eq!(streams[0].group, "OperaVision");
}

#[test]
fn test_build_streams_multiple() {
    let items = vec![
        ("vid1".to_string(), "Opera A Verdi \u{2013} Theatre X".to_string()),
        ("vid2".to_string(), "Ballet B Tchaikovsky \u{2013} Theatre Y".to_string()),
    ];
    let streams = build_streams(&items);
    assert_eq!(streams.len(), 2);
    assert_eq!(streams[0].group, "Theatre X");
    assert_eq!(streams[1].group, "Theatre Y");
}

// ============================================================
// End-to-end: Piped JSON to streams
// ============================================================

#[test]
fn test_piped_to_streams_integration() {
    let json = sample_piped_json();
    let items = parse_piped_json(&json);
    let streams = build_streams(&items);

    assert_eq!(streams.len(), 3);
    assert_eq!(streams[0].url, "https://www.youtube.com/watch?v=abc123DEF_0");
    assert_eq!(streams[0].vod_type, "youtube");
    assert!(streams[0].logo.as_ref().unwrap().contains("abc123DEF_0"));

    // Title parsing still works correctly through the pipeline
    assert!(!streams[0].name.is_empty());
}

#[test]
fn test_piped_to_streams_title_parsing() {
    let json = sample_piped_json();
    let items = parse_piped_json(&json);
    let streams = build_streams(&items);

    // First stream: La Traviata (en-dash split)
    assert_eq!(streams[0].name, "La Traviata Verdi");
    assert_eq!(streams[0].group, "Royal Opera House");

    // Second stream: Swan Lake (en-dash split)
    assert_eq!(streams[1].name, "Swan Lake Tchaikovsky");
    assert_eq!(streams[1].group, "Bolshoi Theatre");

    // Third stream: no delimiter, falls back to OperaVision group
    assert_eq!(streams[2].name, "Some Trailer");
    assert_eq!(streams[2].group, "OperaVision");
}
