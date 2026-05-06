use super::*;

// ============================================================
// File selection tests
// ============================================================

fn make_file(name: &str, format: &str, source: &str) -> ArchiveFile {
    ArchiveFile {
        name: name.to_string(),
        format: format.to_string(),
        source: source.to_string(),
    }
}

#[test]
fn test_pick_best_video_h264() {
    let files = vec![
        make_file("movie.ogv", "Ogg Video", "derivative"),
        make_file("movie.mp4", "h.264", "derivative"),
        make_file("movie_orig.avi", "AVI", "original"),
    ];
    let best = pick_best_video_file(&files).unwrap();
    assert_eq!(best.name, "movie.mp4");
    assert_eq!(best.format, "h.264");
}

#[test]
fn test_pick_best_video_mpeg4_fallback() {
    let files = vec![
        make_file("movie.ogv", "Ogg Video", "derivative"),
        make_file("movie.mp4", "MPEG4", "derivative"),
        make_file("movie_orig.avi", "AVI", "original"),
    ];
    let best = pick_best_video_file(&files).unwrap();
    assert_eq!(best.name, "movie.mp4");
    assert_eq!(best.format, "MPEG4");
}

#[test]
fn test_pick_best_video_ogv_fallback() {
    let files = vec![
        make_file("movie.ogv", "Ogg Video", "derivative"),
        make_file("movie_orig.avi", "AVI", "original"),
    ];
    let best = pick_best_video_file(&files).unwrap();
    assert_eq!(best.name, "movie.ogv");
}

#[test]
fn test_pick_best_video_any_mp4_fallback() {
    let files = vec![
        make_file("movie_orig.mp4", "Unknown Format", "original"),
        make_file("movie_orig.avi", "AVI", "original"),
    ];
    let best = pick_best_video_file(&files).unwrap();
    assert_eq!(best.name, "movie_orig.mp4");
}

#[test]
fn test_pick_best_video_none_when_no_video() {
    let files = vec![
        make_file("track01.mp3", "VBR MP3", "derivative"),
        make_file("metadata.xml", "Metadata", "original"),
    ];
    let result = pick_best_video_file(&files);
    assert!(result.is_none());
}

#[test]
fn test_pick_best_video_empty_files() {
    let files: Vec<ArchiveFile> = vec![];
    let result = pick_best_video_file(&files);
    assert!(result.is_none());
}

#[test]
fn test_pick_best_audio_vbr_mp3() {
    let files = vec![
        make_file("track.flac", "Flac", "original"),
        make_file("track.mp3", "VBR MP3", "derivative"),
        make_file("track.ogg", "Ogg Vorbis", "derivative"),
    ];
    let best = pick_best_audio_file(&files).unwrap();
    assert_eq!(best.name, "track.mp3");
    assert_eq!(best.format, "VBR MP3");
}

#[test]
fn test_pick_best_audio_mp3_derivative() {
    let files = vec![
        make_file("track.flac", "Flac", "original"),
        make_file("track.mp3", "128Kbps MP3", "derivative"),
        make_file("track.ogg", "Ogg Vorbis", "derivative"),
    ];
    let best = pick_best_audio_file(&files).unwrap();
    assert_eq!(best.name, "track.mp3");
}

#[test]
fn test_pick_best_audio_any_mp3_fallback() {
    let files = vec![
        make_file("track.flac", "Flac", "original"),
        make_file("track.mp3", "Unknown", "original"),
    ];
    let best = pick_best_audio_file(&files).unwrap();
    assert_eq!(best.name, "track.mp3");
}

#[test]
fn test_pick_best_audio_ogg_fallback() {
    let files = vec![
        make_file("track.flac", "Flac", "original"),
        make_file("track.ogg", "Ogg Vorbis", "derivative"),
    ];
    let best = pick_best_audio_file(&files).unwrap();
    assert_eq!(best.name, "track.ogg");
}

#[test]
fn test_pick_best_audio_flac_fallback() {
    let files = vec![
        make_file("track.flac", "Flac", "original"),
        make_file("metadata.xml", "Metadata", "original"),
    ];
    let best = pick_best_audio_file(&files).unwrap();
    assert_eq!(best.name, "track.flac");
}

#[test]
fn test_pick_best_audio_none_when_no_audio() {
    let files = vec![
        make_file("metadata.xml", "Metadata", "original"),
        make_file("thumb.jpg", "JPEG", "derivative"),
    ];
    let result = pick_best_audio_file(&files);
    assert!(result.is_none());
}

#[test]
fn test_pick_best_file_dispatches_to_video() {
    let files = vec![
        make_file("movie.mp4", "h.264", "derivative"),
        make_file("track.mp3", "VBR MP3", "derivative"),
    ];
    let best = pick_best_file(&files, MediaType::Video).unwrap();
    assert_eq!(best.name, "movie.mp4");
}

#[test]
fn test_pick_best_file_dispatches_to_audio() {
    let files = vec![
        make_file("movie.mp4", "h.264", "derivative"),
        make_file("track.mp3", "VBR MP3", "derivative"),
    ];
    let best = pick_best_file(&files, MediaType::Audio).unwrap();
    assert_eq!(best.name, "track.mp3");
}

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
                    "year": "1968"
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
// Metadata response parsing tests
// ============================================================

#[test]
fn test_parse_metadata_response_video() {
    let json = r#"{
        "metadata": {
            "title": "Night of the Living Dead",
            "year": "1968",
            "collection": "feature_films",
            "mediatype": "movies"
        },
        "files": [
            {"name": "night_of_the_living_dead.mp4", "format": "h.264", "source": "derivative"},
            {"name": "night_of_the_living_dead.ogv", "format": "Ogg Video", "source": "derivative"},
            {"name": "night_of_the_living_dead_512kb.mp4", "format": "MPEG4", "source": "derivative"},
            {"name": "night_of_the_living_dead.avi", "format": "AVI", "source": "original"}
        ]
    }"#;

    let meta = parse_metadata_response(json.as_bytes()).unwrap();
    assert_eq!(meta.metadata.mediatype, Some("movies".to_string()));
    assert_eq!(meta.files.len(), 4);

    let best = pick_best_video_file(&meta.files).unwrap();
    assert_eq!(best.name, "night_of_the_living_dead.mp4");
    assert_eq!(best.format, "h.264");
}

#[test]
fn test_parse_metadata_response_audio() {
    let json = r#"{
        "metadata": {
            "title": "Mercury Theater on the Air",
            "year": "1938",
            "collection": "oldtimeradio",
            "mediatype": "audio"
        },
        "files": [
            {"name": "show.flac", "format": "Flac", "source": "original"},
            {"name": "show.mp3", "format": "VBR MP3", "source": "derivative"},
            {"name": "show.ogg", "format": "Ogg Vorbis", "source": "derivative"},
            {"name": "__ia_thumb.jpg", "format": "JPEG Thumb", "source": "derivative"}
        ]
    }"#;

    let meta = parse_metadata_response(json.as_bytes()).unwrap();
    assert_eq!(meta.files.len(), 4);

    let best = pick_best_audio_file(&meta.files).unwrap();
    assert_eq!(best.name, "show.mp3");
    assert_eq!(best.format, "VBR MP3");
}

#[test]
fn test_parse_metadata_response_dark_item() {
    let json = r#"{
        "metadata": {
            "title": "Restricted Item"
        },
        "files": [],
        "is_dark": true
    }"#;

    let meta = parse_metadata_response(json.as_bytes()).unwrap();
    assert_eq!(meta.is_dark, Some(true));
}

#[test]
fn test_parse_metadata_response_invalid() {
    let result = parse_metadata_response(b"garbage");
    assert!(result.is_none());
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
// Title extraction tests
// ============================================================

#[test]
fn test_extract_title_string() {
    let v = Some(Value::String("My Movie".to_string()));
    assert_eq!(extract_title(&v), Some("My Movie".to_string()));
}

#[test]
fn test_extract_title_array() {
    let v = Some(Value::Array(vec![Value::String("First Title".to_string())]));
    assert_eq!(extract_title(&v), Some("First Title".to_string()));
}

#[test]
fn test_extract_title_none() {
    assert_eq!(extract_title(&None), None);
}

#[test]
fn test_extract_title_empty_string() {
    let v = Some(Value::String("".to_string()));
    assert_eq!(extract_title(&v), None);
}

// ============================================================
// Collection lookup tests
// ============================================================

#[test]
fn test_collection_display_name_known() {
    assert_eq!(collection_display_name("feature_films"), "Feature Films");
    assert_eq!(collection_display_name("GratefulDead"), "Grateful Dead");
    assert_eq!(collection_display_name("oldtimeradio"), "Old Time Radio");
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
    assert_eq!(parse_items_count(&v), 50); // default
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
}

#[test]
fn test_metadata_url() {
    assert_eq!(
        metadata_url("night-of-the-living-dead"),
        "https://archive.org/metadata/night-of-the-living-dead"
    );
}

#[test]
fn test_download_url() {
    assert_eq!(
        download_url("night-of-the-living-dead", "movie.mp4"),
        "https://archive.org/download/night-of-the-living-dead/movie.mp4"
    );
}

#[test]
fn test_thumbnail_url() {
    assert_eq!(
        thumbnail_url("night-of-the-living-dead"),
        "https://archive.org/services/img/night-of-the-living-dead"
    );
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
// Full metadata + file selection integration tests
// ============================================================

#[test]
fn test_metadata_to_stream_video_full() {
    let meta_json = r#"{
        "metadata": {
            "title": "The General",
            "year": "1926",
            "collection": ["feature_films", "silent_films"],
            "mediatype": "movies"
        },
        "files": [
            {"name": "__ia_thumb.jpg", "format": "JPEG Thumb", "source": "derivative"},
            {"name": "TheGeneral.mp4", "format": "h.264", "source": "derivative"},
            {"name": "TheGeneral.ogv", "format": "Ogg Video", "source": "derivative"},
            {"name": "TheGeneral_512kb.mp4", "format": "MPEG4", "source": "derivative"},
            {"name": "TheGeneral.avi", "format": "Cinepak", "source": "original"}
        ]
    }"#;

    let meta = parse_metadata_response(meta_json.as_bytes()).unwrap();
    let best = pick_best_file(&meta.files, MediaType::Video).unwrap();
    assert_eq!(best.name, "TheGeneral.mp4");

    let url = download_url("the-general-1926", &best.name);
    assert_eq!(url, "https://archive.org/download/the-general-1926/TheGeneral.mp4");
}

#[test]
fn test_metadata_to_stream_audio_concert() {
    let meta_json = r#"{
        "metadata": {
            "title": "Grateful Dead Live at Winterland Arena 1977-12-31",
            "year": "1977",
            "collection": "GratefulDead",
            "mediatype": "etree"
        },
        "files": [
            {"name": "gd1977-12-31d1t01.flac", "format": "Flac", "source": "original"},
            {"name": "gd1977-12-31d1t01.mp3", "format": "VBR MP3", "source": "derivative"},
            {"name": "gd1977-12-31d1t01.ogg", "format": "Ogg Vorbis", "source": "derivative"},
            {"name": "gd1977-12-31d1t02.flac", "format": "Flac", "source": "original"},
            {"name": "gd1977-12-31d1t02.mp3", "format": "VBR MP3", "source": "derivative"}
        ]
    }"#;

    let meta = parse_metadata_response(meta_json.as_bytes()).unwrap();
    let best = pick_best_file(&meta.files, MediaType::Audio).unwrap();
    assert_eq!(best.name, "gd1977-12-31d1t01.mp3");
    assert_eq!(best.format, "VBR MP3");
}

#[test]
fn test_metadata_no_playable_files() {
    let meta_json = r#"{
        "metadata": {
            "title": "Metadata Only Item",
            "mediatype": "texts"
        },
        "files": [
            {"name": "item.pdf", "format": "Text PDF", "source": "original"},
            {"name": "item_djvu.xml", "format": "Djvu XML", "source": "derivative"}
        ]
    }"#;

    let meta = parse_metadata_response(meta_json.as_bytes()).unwrap();
    assert!(pick_best_file(&meta.files, MediaType::Video).is_none());
    assert!(pick_best_file(&meta.files, MediaType::Audio).is_none());
}
