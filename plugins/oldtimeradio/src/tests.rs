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
    // Missing optional fields should default to empty strings
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
// Metadata parsing tests
// ============================================================

#[test]
fn test_parse_metadata_mp3s_filters_correctly() {
    let json = r#"{
        "files": [
            {"name": "episode1.mp3", "format": "VBR MP3", "title": "Episode 1"},
            {"name": "episode2.mp3", "format": "VBR MP3", "title": "Episode 2"},
            {"name": "metadata.xml", "format": "Metadata", "title": ""},
            {"name": "episode1.ogg", "format": "Ogg Vorbis", "title": "Episode 1"},
            {"name": "cover.jpg", "format": "JPEG", "title": ""},
            {"name": "episode3.mp3", "format": "VBR MP3", "title": ""}
        ]
    }"#;

    let mp3s = parse_metadata_mp3s(json.as_bytes()).unwrap();
    assert_eq!(mp3s.len(), 3);
    assert_eq!(mp3s[0].name, "episode1.mp3");
    assert_eq!(mp3s[0].format, "VBR MP3");
    assert_eq!(mp3s[0].title, "Episode 1");
    assert_eq!(mp3s[1].name, "episode2.mp3");
    assert_eq!(mp3s[2].name, "episode3.mp3");
    assert_eq!(mp3s[2].title, "");
}

#[test]
fn test_parse_metadata_mp3s_no_mp3s() {
    let json = r#"{
        "files": [
            {"name": "episode.ra", "format": "RealAudio", "title": "Old format"},
            {"name": "metadata.xml", "format": "Metadata", "title": ""}
        ]
    }"#;

    let mp3s = parse_metadata_mp3s(json.as_bytes()).unwrap();
    assert!(mp3s.is_empty());
}

#[test]
fn test_parse_metadata_mp3s_empty_files() {
    let json = r#"{"files": []}"#;
    let mp3s = parse_metadata_mp3s(json.as_bytes()).unwrap();
    assert!(mp3s.is_empty());
}

#[test]
fn test_parse_metadata_mp3s_invalid_json() {
    let result = parse_metadata_mp3s(b"garbage");
    assert!(result.is_none());
}

#[test]
fn test_parse_metadata_missing_format_field() {
    // Files with missing format should default to empty string and be filtered out
    let json = r#"{
        "files": [
            {"name": "episode1.mp3", "title": "Episode 1"},
            {"name": "episode2.mp3", "format": "VBR MP3", "title": "Episode 2"}
        ]
    }"#;

    let mp3s = parse_metadata_mp3s(json.as_bytes()).unwrap();
    assert_eq!(mp3s.len(), 1);
    assert_eq!(mp3s[0].name, "episode2.mp3");
}

// ============================================================
// Episode name extraction tests
// ============================================================

#[test]
fn test_episode_name_simple() {
    assert_eq!(
        episode_name_from_filename("Billy the Kid.mp3"),
        "Billy the Kid"
    );
}

#[test]
fn test_episode_name_with_show_prefix() {
    assert_eq!(
        episode_name_from_filename("Gunsmoke 52-04-26 (001) Billy the Kid.mp3"),
        "Gunsmoke 52-04-26 (001) Billy the Kid"
    );
}

#[test]
fn test_episode_name_percent_encoded() {
    assert_eq!(
        episode_name_from_filename("Gunsmoke%2052-04-26%20(001)%20Billy%20the%20Kid.mp3"),
        "Gunsmoke 52-04-26 (001) Billy the Kid"
    );
}

#[test]
fn test_episode_name_uppercase_extension() {
    assert_eq!(
        episode_name_from_filename("Episode Title.MP3"),
        "Episode Title"
    );
}

#[test]
fn test_episode_name_no_extension() {
    assert_eq!(
        episode_name_from_filename("no_extension_here"),
        "no_extension_here"
    );
}

#[test]
fn test_episode_name_empty() {
    assert_eq!(episode_name_from_filename(""), "");
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
    // Incomplete percent sequence should be left as-is
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
// Group derivation tests
// ============================================================

#[test]
fn test_derive_group_with_creator() {
    let doc = IASearchDoc {
        identifier: "test".to_string(),
        title: "Some Title".to_string(),
        description: String::new(),
        creator: "CBS Radio".to_string(),
    };
    assert_eq!(derive_group(&doc), "CBS Radio");
}

#[test]
fn test_derive_group_no_creator_uses_title() {
    let doc = IASearchDoc {
        identifier: "test".to_string(),
        title: "Gunsmoke Collection".to_string(),
        description: String::new(),
        creator: String::new(),
    };
    assert_eq!(derive_group(&doc), "Gunsmoke Collection");
}

#[test]
fn test_derive_group_no_creator_no_title() {
    let doc = IASearchDoc {
        identifier: "test".to_string(),
        title: String::new(),
        description: String::new(),
        creator: String::new(),
    };
    assert_eq!(derive_group(&doc), "Miscellaneous");
}

// ============================================================
// Stream ID tests
// ============================================================

#[test]
fn test_make_stream_id() {
    assert_eq!(
        make_stream_id("OTRR_Gunsmoke_Singles", 0),
        "OTRR_Gunsmoke_Singles__0000"
    );
    assert_eq!(
        make_stream_id("OTRR_Gunsmoke_Singles", 42),
        "OTRR_Gunsmoke_Singles__0042"
    );
}

// ============================================================
// Download URL tests
// ============================================================

#[test]
fn test_make_download_url() {
    assert_eq!(
        make_download_url("OTRR_Gunsmoke_Singles", "Billy the Kid.mp3"),
        "https://archive.org/download/OTRR_Gunsmoke_Singles/Billy%20the%20Kid.mp3"
    );
}

#[test]
fn test_make_download_url_no_spaces() {
    assert_eq!(
        make_download_url("item123", "episode.mp3"),
        "https://archive.org/download/item123/episode.mp3"
    );
}

// ============================================================
// doc_to_streams integration test
// ============================================================

#[test]
fn test_doc_to_streams_basic() {
    let doc = IASearchDoc {
        identifier: "OTRR_Gunsmoke_Singles".to_string(),
        title: "Gunsmoke".to_string(),
        description: "Classic western".to_string(),
        creator: "CBS Radio".to_string(),
    };

    let files = vec![
        IAFile {
            name: "Gunsmoke 52-04-26 (001) Billy the Kid.mp3".to_string(),
            format: "VBR MP3".to_string(),
            title: "Billy the Kid".to_string(),
        },
        IAFile {
            name: "Gunsmoke 52-05-03 (002) Hot Rod.mp3".to_string(),
            format: "VBR MP3".to_string(),
            title: "".to_string(),
        },
    ];

    let streams = doc_to_streams(&doc, &files);
    assert_eq!(streams.len(), 2);

    // First stream uses the title from the file metadata
    assert_eq!(streams[0].id, "OTRR_Gunsmoke_Singles__0000");
    assert_eq!(streams[0].name, "Billy the Kid");
    assert_eq!(streams[0].group, "CBS Radio");
    assert_eq!(
        streams[0].url,
        "https://archive.org/download/OTRR_Gunsmoke_Singles/Gunsmoke%2052-04-26%20(001)%20Billy%20the%20Kid.mp3"
    );
    assert_eq!(
        streams[0].logo.as_deref(),
        Some("https://archive.org/services/img/OTRR_Gunsmoke_Singles")
    );
    assert_eq!(streams[0].episode_name.as_deref(), Some("Billy the Kid"));
    assert_eq!(streams[0].vod_type, "movie");

    // Second stream falls back to filename-derived episode name
    assert_eq!(streams[1].id, "OTRR_Gunsmoke_Singles__0001");
    assert_eq!(streams[1].name, "Gunsmoke 52-05-03 (002) Hot Rod");
    assert_eq!(
        streams[1].episode_name.as_deref(),
        Some("Gunsmoke 52-05-03 (002) Hot Rod")
    );
}

#[test]
fn test_doc_to_streams_empty_files() {
    let doc = IASearchDoc {
        identifier: "empty_item".to_string(),
        title: "Empty".to_string(),
        description: String::new(),
        creator: String::new(),
    };

    let streams = doc_to_streams(&doc, &[]);
    assert!(streams.is_empty());
}

#[test]
fn test_doc_to_streams_tags() {
    let doc = IASearchDoc {
        identifier: "test".to_string(),
        title: "Test".to_string(),
        description: String::new(),
        creator: String::new(),
    };

    let files = vec![IAFile {
        name: "episode.mp3".to_string(),
        format: "VBR MP3".to_string(),
        title: "Episode".to_string(),
    }];

    let streams = doc_to_streams(&doc, &files);
    assert_eq!(streams.len(), 1);
    let tags = streams[0].tags.as_ref().unwrap();
    assert!(tags.contains(&"radio".to_string()));
    assert!(tags.contains(&"classic".to_string()));
}
