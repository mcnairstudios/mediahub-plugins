use super::*;

// ============================================================
// Audiobook JSON parsing tests
// ============================================================

fn sample_audiobooks_json() -> &'static str {
    r#"{
        "books": [
            {
                "id": "123",
                "title": "A Tale of Two Cities",
                "description": "A novel by Charles Dickens",
                "language": "English",
                "num_sections": "45",
                "authors": [
                    {"first_name": "Charles", "last_name": "Dickens"}
                ],
                "totaltime": "14:42:03",
                "url_librivox": "https://librivox.org/a-tale-of-two-cities-by-charles-dickens/"
            },
            {
                "id": "456",
                "title": "Pride and Prejudice",
                "description": "A novel by Jane Austen",
                "language": "English",
                "num_sections": "61",
                "authors": [
                    {"first_name": "Jane", "last_name": "Austen"}
                ],
                "totaltime": "11:35:40",
                "url_librivox": "https://librivox.org/pride-and-prejudice-by-jane-austen/"
            }
        ]
    }"#
}

fn sample_audiotracks_json() -> &'static str {
    r#"{
        "sections": [
            {
                "id": "1001",
                "section_number": "1",
                "title": "Book the First - Recalled to Life - I. The Period",
                "listen_url": "https://www.archive.org/download/tale_two_cities_0711_librivox/taletwo_01_dickens_64kb.mp3",
                "language": "English",
                "playtime": "08:23"
            },
            {
                "id": "1002",
                "section_number": "2",
                "title": "Book the First - Recalled to Life - II. The Mail",
                "listen_url": "https://www.archive.org/download/tale_two_cities_0711_librivox/taletwo_02_dickens_64kb.mp3",
                "language": "English",
                "playtime": "15:47"
            },
            {
                "id": "1003",
                "section_number": "3",
                "title": "Book the First - Recalled to Life - III. The Night Shadows",
                "listen_url": "https://www.archive.org/download/tale_two_cities_0711_librivox/taletwo_03_dickens_64kb.mp3",
                "language": "",
                "playtime": "06:12"
            }
        ]
    }"#
}

#[test]
fn test_parse_audiobooks() {
    let data = sample_audiobooks_json().as_bytes();
    let books = parse_audiobooks(data).expect("should parse audiobooks");
    assert_eq!(books.len(), 2);

    assert_eq!(books[0].id, "123");
    assert_eq!(books[0].title, "A Tale of Two Cities");
    assert_eq!(books[0].language, "English");
    assert_eq!(books[0].num_sections, "45");
    assert_eq!(books[0].authors.len(), 1);
    assert_eq!(books[0].authors[0].first_name, "Charles");
    assert_eq!(books[0].authors[0].last_name, "Dickens");

    assert_eq!(books[1].id, "456");
    assert_eq!(books[1].title, "Pride and Prejudice");
}

#[test]
fn test_parse_audiobooks_invalid_json() {
    let data = b"not json at all";
    assert!(parse_audiobooks(data).is_none());
}

#[test]
fn test_parse_audiobooks_empty_list() {
    let data = br#"{"books": []}"#;
    let books = parse_audiobooks(data).expect("should parse empty books list");
    assert_eq!(books.len(), 0);
}

#[test]
fn test_parse_audiotracks() {
    let data = sample_audiotracks_json().as_bytes();
    let tracks = parse_audiotracks(data).expect("should parse audiotracks");
    assert_eq!(tracks.len(), 3);

    assert_eq!(tracks[0].id, "1001");
    assert_eq!(tracks[0].section_number, "1");
    assert_eq!(tracks[0].title, "Book the First - Recalled to Life - I. The Period");
    assert_eq!(
        tracks[0].listen_url,
        "https://www.archive.org/download/tale_two_cities_0711_librivox/taletwo_01_dickens_64kb.mp3"
    );
    assert_eq!(tracks[0].language, "English");
    assert_eq!(tracks[0].playtime, "08:23");

    assert_eq!(tracks[1].section_number, "2");
    assert_eq!(tracks[2].section_number, "3");
}

#[test]
fn test_parse_audiotracks_invalid_json() {
    let data = b"{{invalid}}";
    assert!(parse_audiotracks(data).is_none());
}

#[test]
fn test_parse_audiotracks_empty() {
    let data = br#"{"sections": []}"#;
    let tracks = parse_audiotracks(data).expect("should parse empty sections list");
    assert_eq!(tracks.len(), 0);
}

// ============================================================
// Author name tests
// ============================================================

#[test]
fn test_author_full_name() {
    let author = Author {
        first_name: "Charles".to_string(),
        last_name: "Dickens".to_string(),
    };
    assert_eq!(author.full_name(), "Charles Dickens");
}

#[test]
fn test_author_full_name_first_only() {
    let author = Author {
        first_name: "Homer".to_string(),
        last_name: String::new(),
    };
    assert_eq!(author.full_name(), "Homer");
}

#[test]
fn test_author_full_name_empty() {
    let author = Author {
        first_name: String::new(),
        last_name: String::new(),
    };
    assert_eq!(author.full_name(), "Unknown Author");
}

#[test]
fn test_audiobook_author_display_multiple() {
    let book = Audiobook {
        id: "1".to_string(),
        title: "Test".to_string(),
        description: String::new(),
        language: "English".to_string(),
        num_sections: "1".to_string(),
        authors: vec![
            Author {
                first_name: "Jane".to_string(),
                last_name: "Austen".to_string(),
            },
            Author {
                first_name: "Charlotte".to_string(),
                last_name: "Bronte".to_string(),
            },
        ],
        totaltime: String::new(),
        url_librivox: String::new(),
    };
    assert_eq!(book.author_display(), "Jane Austen, Charlotte Bronte");
}

#[test]
fn test_audiobook_author_display_no_authors() {
    let book = Audiobook {
        id: "1".to_string(),
        title: "Test".to_string(),
        description: String::new(),
        language: String::new(),
        num_sections: String::new(),
        authors: vec![],
        totaltime: String::new(),
        url_librivox: String::new(),
    };
    assert_eq!(book.author_display(), "Unknown Author");
}

#[test]
fn test_audiobook_group_label() {
    let book = Audiobook {
        id: "123".to_string(),
        title: "A Tale of Two Cities".to_string(),
        description: String::new(),
        language: "English".to_string(),
        num_sections: "45".to_string(),
        authors: vec![Author {
            first_name: "Charles".to_string(),
            last_name: "Dickens".to_string(),
        }],
        totaltime: String::new(),
        url_librivox: String::new(),
    };
    assert_eq!(
        book.group_label(),
        "A Tale of Two Cities - Charles Dickens"
    );
}

// ============================================================
// URL construction tests
// ============================================================

#[test]
fn test_audiobooks_url_with_language() {
    let url = audiobooks_url(25, "English");
    assert_eq!(
        url,
        "https://librivox.org/api/feed/audiobooks?format=json&limit=25&language=English"
    );
}

#[test]
fn test_audiobooks_url_all_languages() {
    let url = audiobooks_url(50, "all");
    assert_eq!(
        url,
        "https://librivox.org/api/feed/audiobooks?format=json&limit=50"
    );
}

#[test]
fn test_audiobooks_url_empty_language() {
    let url = audiobooks_url(10, "");
    assert_eq!(
        url,
        "https://librivox.org/api/feed/audiobooks?format=json&limit=10"
    );
}

#[test]
fn test_audiotracks_url() {
    let url = audiotracks_url("123");
    assert_eq!(
        url,
        "https://librivox.org/api/feed/audiotracks?project_id=123&format=json"
    );
}

#[test]
fn test_search_url() {
    let url = search_url("tale of two", 10);
    assert_eq!(
        url,
        "https://librivox.org/api/feed/audiobooks?format=json&limit=10&title=^tale+of+two"
    );
}

#[test]
fn test_search_url_single_word() {
    let url = search_url("pride", 5);
    assert_eq!(
        url,
        "https://librivox.org/api/feed/audiobooks?format=json&limit=5&title=^pride"
    );
}

// ============================================================
// Track-to-stream mapping tests
// ============================================================

#[test]
fn test_track_to_stream_basic() {
    let book = Audiobook {
        id: "123".to_string(),
        title: "A Tale of Two Cities".to_string(),
        description: String::new(),
        language: "English".to_string(),
        num_sections: "45".to_string(),
        authors: vec![Author {
            first_name: "Charles".to_string(),
            last_name: "Dickens".to_string(),
        }],
        totaltime: String::new(),
        url_librivox: String::new(),
    };

    let track = AudioTrack {
        id: "1001".to_string(),
        section_number: "1".to_string(),
        title: "The Period".to_string(),
        listen_url: "https://www.archive.org/download/tale_two_cities/ch01_64kb.mp3".to_string(),
        language: "English".to_string(),
        playtime: "08:23".to_string(),
    };

    let stream = track_to_stream(&book, &track);
    assert_eq!(stream.id, "lbv-123-1");
    assert_eq!(stream.name, "The Period");
    assert_eq!(
        stream.url,
        "https://www.archive.org/download/tale_two_cities/ch01_64kb.mp3"
    );
    assert_eq!(
        stream.group,
        "A Tale of Two Cities - Charles Dickens"
    );
    assert_eq!(stream.vod_type, "episode");
    assert_eq!(
        stream.episode_name,
        Some("Ch. 1: The Period".to_string())
    );
    assert!(stream.tags.contains(&"audiobook".to_string()));
    assert!(stream.tags.contains(&"English".to_string()));
}

#[test]
fn test_track_to_stream_falls_back_to_book_language() {
    let book = Audiobook {
        id: "456".to_string(),
        title: "Les Miserables".to_string(),
        description: String::new(),
        language: "French".to_string(),
        num_sections: "10".to_string(),
        authors: vec![Author {
            first_name: "Victor".to_string(),
            last_name: "Hugo".to_string(),
        }],
        totaltime: String::new(),
        url_librivox: String::new(),
    };

    let track = AudioTrack {
        id: "2001".to_string(),
        section_number: "5".to_string(),
        title: "Chapitre V".to_string(),
        listen_url: "https://www.archive.org/download/les_mis/ch05_64kb.mp3".to_string(),
        language: String::new(), // empty track language
        playtime: "12:00".to_string(),
    };

    let stream = track_to_stream(&book, &track);
    assert_eq!(stream.id, "lbv-456-5");
    assert_eq!(stream.group, "Les Miserables - Victor Hugo");
    assert!(stream.tags.contains(&"French".to_string()));
    assert!(stream.tags.contains(&"audiobook".to_string()));
}

#[test]
fn test_track_to_stream_episode_name_format() {
    let book = Audiobook {
        id: "789".to_string(),
        title: "Moby Dick".to_string(),
        description: String::new(),
        language: "English".to_string(),
        num_sections: "135".to_string(),
        authors: vec![Author {
            first_name: "Herman".to_string(),
            last_name: "Melville".to_string(),
        }],
        totaltime: String::new(),
        url_librivox: String::new(),
    };

    let track = AudioTrack {
        id: "3001".to_string(),
        section_number: "42".to_string(),
        title: "The Whiteness of the Whale".to_string(),
        listen_url: "https://www.archive.org/download/moby_dick/ch42_64kb.mp3".to_string(),
        language: "English".to_string(),
        playtime: "20:15".to_string(),
    };

    let stream = track_to_stream(&book, &track);
    assert_eq!(
        stream.episode_name,
        Some("Ch. 42: The Whiteness of the Whale".to_string())
    );
}

// ============================================================
// JSON round-trip test (serialization)
// ============================================================

#[test]
fn test_stream_serialization() {
    let stream = Stream {
        id: "lbv-1-1".to_string(),
        name: "Chapter One".to_string(),
        url: "https://archive.org/download/test/ch01.mp3".to_string(),
        group: "Test Book - Test Author".to_string(),
        logo: String::new(),
        vod_type: "episode".to_string(),
        tags: vec!["audiobook".to_string(), "English".to_string()],
        episode_name: Some("Ch. 1: Chapter One".to_string()),
    };

    let json = serde_json::to_value(&stream).expect("should serialize");
    assert_eq!(json["id"], "lbv-1-1");
    assert_eq!(json["name"], "Chapter One");
    assert_eq!(json["vod_type"], "episode");
    assert_eq!(json["episode_name"], "Ch. 1: Chapter One");
}

#[test]
fn test_stream_serialization_no_episode_name() {
    let stream = Stream {
        id: "lbv-1-1".to_string(),
        name: "Chapter One".to_string(),
        url: "https://archive.org/download/test/ch01.mp3".to_string(),
        group: "Test Book - Test Author".to_string(),
        logo: String::new(),
        vod_type: "episode".to_string(),
        tags: vec!["audiobook".to_string()],
        episode_name: None,
    };

    let json = serde_json::to_value(&stream).expect("should serialize");
    // episode_name should be absent when None due to skip_serializing_if
    assert!(json.get("episode_name").is_none());
}

// ============================================================
// Audiobooks response with missing/optional fields
// ============================================================

#[test]
fn test_parse_audiobook_with_missing_optional_fields() {
    let data = br#"{
        "books": [
            {
                "id": "999",
                "title": "Minimal Book",
                "authors": []
            }
        ]
    }"#;

    let books = parse_audiobooks(data).expect("should parse");
    assert_eq!(books.len(), 1);
    assert_eq!(books[0].id, "999");
    assert_eq!(books[0].title, "Minimal Book");
    assert_eq!(books[0].description, "");
    assert_eq!(books[0].language, "");
    assert_eq!(books[0].num_sections, "");
    assert_eq!(books[0].authors.len(), 0);
    assert_eq!(books[0].author_display(), "Unknown Author");
}

#[test]
fn test_parse_audiotrack_with_missing_fields() {
    let data = br#"{
        "sections": [
            {
                "id": "5001",
                "section_number": "1",
                "title": "Intro",
                "listen_url": "https://www.archive.org/download/test/intro.mp3"
            }
        ]
    }"#;

    let tracks = parse_audiotracks(data).expect("should parse");
    assert_eq!(tracks.len(), 1);
    assert_eq!(tracks[0].language, "");
    assert_eq!(tracks[0].playtime, "");
}
