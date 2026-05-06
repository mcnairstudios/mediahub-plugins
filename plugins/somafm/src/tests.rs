use super::*;
use serde_json::json;

// ============================================================
// Genre extraction tests
// ============================================================

#[test]
fn test_extract_first_genre_basic() {
    assert_eq!(extract_first_genre("ambient|electronic"), "ambient");
}

#[test]
fn test_extract_first_genre_single() {
    assert_eq!(extract_first_genre("jazz"), "jazz");
}

#[test]
fn test_extract_first_genre_empty() {
    assert_eq!(extract_first_genre(""), "other");
}

#[test]
fn test_extract_first_genre_whitespace() {
    assert_eq!(extract_first_genre("  "), "other");
}

#[test]
fn test_extract_first_genre_leading_pipe() {
    // "|electronic" should yield "other" since first segment is empty
    assert_eq!(extract_first_genre("|electronic"), "other");
}

#[test]
fn test_extract_first_genre_multiple_pipes() {
    assert_eq!(
        extract_first_genre("indie|rock|alternative"),
        "indie"
    );
}

#[test]
fn test_extract_first_genre_with_spaces() {
    assert_eq!(extract_first_genre(" ambient | electronic "), "ambient");
}

// ============================================================
// Genre tag splitting tests
// ============================================================

#[test]
fn test_split_genre_tags_multiple() {
    assert_eq!(
        split_genre_tags("ambient|electronic|downtempo"),
        vec!["ambient", "electronic", "downtempo"]
    );
}

#[test]
fn test_split_genre_tags_single() {
    assert_eq!(split_genre_tags("jazz"), vec!["jazz"]);
}

#[test]
fn test_split_genre_tags_empty() {
    let empty: Vec<String> = vec![];
    assert_eq!(split_genre_tags(""), empty);
}

#[test]
fn test_split_genre_tags_with_spaces() {
    assert_eq!(
        split_genre_tags(" rock | indie "),
        vec!["rock", "indie"]
    );
}

// ============================================================
// Playlist URL selection tests
// ============================================================

#[test]
fn test_pick_best_mp3_url_highest_quality() {
    let playlists = vec![
        json!({"url": "http://example.com/low.pls", "format": "mp3", "quality": "low"}),
        json!({"url": "http://example.com/highest.pls", "format": "mp3", "quality": "highest"}),
        json!({"url": "http://example.com/high.pls", "format": "mp3", "quality": "high"}),
    ];
    assert_eq!(
        pick_best_mp3_url(&playlists),
        "http://example.com/highest.pls"
    );
}

#[test]
fn test_pick_best_mp3_url_prefers_mp3_over_aac() {
    let playlists = vec![
        json!({"url": "http://example.com/aac.pls", "format": "aac", "quality": "highest"}),
        json!({"url": "http://example.com/mp3.pls", "format": "mp3", "quality": "high"}),
    ];
    assert_eq!(
        pick_best_mp3_url(&playlists),
        "http://example.com/mp3.pls"
    );
}

#[test]
fn test_pick_best_mp3_url_fallback_to_non_mp3() {
    let playlists = vec![
        json!({"url": "http://example.com/aac.pls", "format": "aac", "quality": "highest"}),
    ];
    assert_eq!(
        pick_best_mp3_url(&playlists),
        "http://example.com/aac.pls"
    );
}

#[test]
fn test_pick_best_mp3_url_empty_playlists() {
    let playlists: Vec<Value> = vec![];
    assert_eq!(pick_best_mp3_url(&playlists), "");
}

#[test]
fn test_pick_best_mp3_url_single_mp3() {
    let playlists = vec![
        json!({"url": "http://example.com/stream.pls", "format": "mp3", "quality": "high"}),
    ];
    assert_eq!(
        pick_best_mp3_url(&playlists),
        "http://example.com/stream.pls"
    );
}

#[test]
fn test_pick_best_mp3_url_varying_bitrates() {
    // Some channels may have different quality labels
    let playlists = vec![
        json!({"url": "http://example.com/low.pls", "format": "mp3", "quality": "low"}),
        json!({"url": "http://example.com/high.pls", "format": "mp3", "quality": "high"}),
        json!({"url": "http://example.com/aac-high.pls", "format": "aac", "quality": "highest"}),
    ];
    assert_eq!(
        pick_best_mp3_url(&playlists),
        "http://example.com/high.pls"
    );
}

#[test]
fn test_pick_best_mp3_url_skips_empty_urls() {
    let playlists = vec![
        json!({"url": "", "format": "mp3", "quality": "highest"}),
        json!({"url": "http://example.com/valid.pls", "format": "mp3", "quality": "high"}),
    ];
    assert_eq!(
        pick_best_mp3_url(&playlists),
        "http://example.com/valid.pls"
    );
}

// ============================================================
// Channel parsing tests
// ============================================================

#[test]
fn test_channel_to_stream_basic() {
    let channel = json!({
        "id": "groovesalad",
        "title": "Groove Salad",
        "genre": "ambient|electronic",
        "largeimage": "https://api.somafm.com/logos/256/groovesalad256.png",
        "playlists": [
            {"url": "http://somafm.com/groovesalad256.pls", "format": "mp3", "quality": "highest"},
            {"url": "http://somafm.com/groovesalad130.pls", "format": "mp3", "quality": "high"},
            {"url": "http://somafm.com/groovesalad64.pls", "format": "aac", "quality": "highest"}
        ]
    });

    let stream = channel_to_stream(&channel).unwrap();

    assert_eq!(stream.id, "groovesalad");
    assert_eq!(stream.name, "Groove Salad");
    assert_eq!(stream.url, "http://somafm.com/groovesalad256.pls");
    assert_eq!(stream.group, "ambient");
    assert_eq!(
        stream.logo,
        "https://api.somafm.com/logos/256/groovesalad256.png"
    );
    assert_eq!(stream.tags, vec!["ambient", "electronic"]);
}

#[test]
fn test_channel_to_stream_missing_id() {
    let channel = json!({
        "title": "No ID Channel",
        "genre": "rock"
    });
    assert!(channel_to_stream(&channel).is_none());
}

#[test]
fn test_channel_to_stream_no_playlists() {
    let channel = json!({
        "id": "test",
        "title": "Test Channel",
        "genre": "rock",
        "largeimage": "https://example.com/logo.png"
    });

    let stream = channel_to_stream(&channel).unwrap();
    assert_eq!(stream.id, "test");
    assert_eq!(stream.url, "");
    assert_eq!(stream.group, "rock");
}

#[test]
fn test_channel_to_stream_empty_genre() {
    let channel = json!({
        "id": "test",
        "title": "Test Channel",
        "genre": "",
        "largeimage": "",
        "playlists": []
    });

    let stream = channel_to_stream(&channel).unwrap();
    assert_eq!(stream.group, "other");
    assert!(stream.tags.is_empty());
}

// ============================================================
// Full response parsing tests
// ============================================================

#[test]
fn test_parse_channels_response_full() {
    let response = json!({
        "channels": [
            {
                "id": "groovesalad",
                "title": "Groove Salad",
                "genre": "ambient|electronic",
                "largeimage": "https://api.somafm.com/logos/256/groovesalad256.png",
                "playlists": [
                    {"url": "http://somafm.com/groovesalad256.pls", "format": "mp3", "quality": "highest"},
                    {"url": "http://somafm.com/groovesalad130.pls", "format": "mp3", "quality": "high"}
                ]
            },
            {
                "id": "dronezone",
                "title": "Drone Zone",
                "genre": "ambient|space",
                "largeimage": "https://api.somafm.com/logos/256/dronezone256.png",
                "playlists": [
                    {"url": "http://somafm.com/dronezone256.pls", "format": "mp3", "quality": "highest"}
                ]
            },
            {
                "id": "defcon",
                "title": "DEF CON Radio",
                "genre": "electronic|hacker",
                "largeimage": "https://api.somafm.com/logos/256/defcon256.png",
                "playlists": [
                    {"url": "http://somafm.com/defcon256.pls", "format": "mp3", "quality": "highest"},
                    {"url": "http://somafm.com/defcon64.pls", "format": "aac", "quality": "high"}
                ]
            }
        ]
    });

    let body = serde_json::to_vec(&response).unwrap();
    let streams = parse_channels_response(&body);

    assert_eq!(streams.len(), 3);

    assert_eq!(streams[0].id, "groovesalad");
    assert_eq!(streams[0].group, "ambient");
    assert_eq!(streams[0].url, "http://somafm.com/groovesalad256.pls");

    assert_eq!(streams[1].id, "dronezone");
    assert_eq!(streams[1].group, "ambient");

    assert_eq!(streams[2].id, "defcon");
    assert_eq!(streams[2].group, "electronic");
    assert_eq!(streams[2].tags, vec!["electronic", "hacker"]);
}

#[test]
fn test_parse_channels_response_empty_channels() {
    let response = json!({"channels": []});
    let body = serde_json::to_vec(&response).unwrap();
    assert!(parse_channels_response(&body).is_empty());
}

#[test]
fn test_parse_channels_response_missing_channels_key() {
    let response = json!({"something_else": []});
    let body = serde_json::to_vec(&response).unwrap();
    assert!(parse_channels_response(&body).is_empty());
}

#[test]
fn test_parse_channels_response_invalid_json() {
    let body = b"not valid json at all";
    assert!(parse_channels_response(body).is_empty());
}

#[test]
fn test_parse_channels_response_skips_channels_without_id() {
    let response = json!({
        "channels": [
            {
                "id": "good",
                "title": "Good Channel",
                "genre": "rock",
                "playlists": []
            },
            {
                "title": "No ID",
                "genre": "rock",
                "playlists": []
            }
        ]
    });

    let body = serde_json::to_vec(&response).unwrap();
    let streams = parse_channels_response(&body);
    assert_eq!(streams.len(), 1);
    assert_eq!(streams[0].id, "good");
}
