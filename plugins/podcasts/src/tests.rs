use super::*;

// ============================================================
// Sample iTunes API responses for testing
// ============================================================

fn sample_search_response() -> &'static str {
    r#"{
        "resultCount": 3,
        "results": [
            {
                "wrapperType": "podcastEpisode",
                "kind": "podcast-episode",
                "trackId": 1000001,
                "trackName": "Episode 1: The Beginning",
                "collectionName": "True Crime Weekly",
                "collectionId": 555001,
                "episodeUrl": "https://traffic.megaphone.fm/episode1.mp3",
                "artworkUrl600": "https://is1-ssl.mzstatic.com/image/thumb/Podcasts/artwork600.jpg",
                "artworkUrl160": "https://is1-ssl.mzstatic.com/image/thumb/Podcasts/artwork160.jpg",
                "shortDescription": "The story begins here",
                "genres": ["True Crime", "Society & Culture"]
            },
            {
                "wrapperType": "podcastEpisode",
                "kind": "podcast-episode",
                "trackId": 1000002,
                "trackName": "Episode 2: The Investigation",
                "collectionName": "True Crime Weekly",
                "collectionId": 555001,
                "episodeUrl": "https://traffic.megaphone.fm/episode2.mp3",
                "artworkUrl600": "https://is1-ssl.mzstatic.com/image/thumb/Podcasts/artwork600.jpg",
                "shortDescription": "",
                "genres": ["True Crime"]
            },
            {
                "wrapperType": "podcastEpisode",
                "kind": "podcast-episode",
                "trackId": 1000003,
                "trackName": "Tech News Roundup",
                "collectionName": "Daily Tech",
                "collectionId": 555002,
                "episodeUrl": "https://dts.podtrac.com/technews.mp3",
                "artworkUrl600": "https://is1-ssl.mzstatic.com/image/thumb/Podcasts/tech600.jpg",
                "shortDescription": "Weekly tech roundup",
                "genres": ["Technology", "News"]
            }
        ]
    }"#
}

fn sample_lookup_response() -> &'static str {
    r#"{
        "resultCount": 3,
        "results": [
            {
                "wrapperType": "track",
                "kind": "podcast",
                "collectionId": 555001,
                "collectionName": "True Crime Weekly",
                "artworkUrl600": "https://is1-ssl.mzstatic.com/image/thumb/Podcasts/artwork600.jpg"
            },
            {
                "wrapperType": "podcastEpisode",
                "kind": "podcast-episode",
                "trackId": 1000010,
                "trackName": "Episode 10: The Verdict",
                "collectionName": "True Crime Weekly",
                "collectionId": 555001,
                "episodeUrl": "https://traffic.megaphone.fm/episode10.mp3",
                "artworkUrl600": "https://is1-ssl.mzstatic.com/image/thumb/Podcasts/artwork600.jpg",
                "shortDescription": "Final verdict",
                "genres": ["True Crime", "Society & Culture"]
            },
            {
                "wrapperType": "podcastEpisode",
                "kind": "podcast-episode",
                "trackId": 1000011,
                "trackName": "Episode 11: Aftermath",
                "collectionName": "True Crime Weekly",
                "collectionId": 555001,
                "episodeUrl": "https://traffic.megaphone.fm/episode11.mp3",
                "artworkUrl600": "https://is1-ssl.mzstatic.com/image/thumb/Podcasts/artwork600.jpg",
                "shortDescription": "",
                "genres": ["True Crime"]
            }
        ]
    }"#
}

// ============================================================
// Tests: Episode parsing
// ============================================================

#[test]
fn test_parse_episodes_basic() {
    let streams = parse_episodes(sample_search_response().as_bytes());
    assert_eq!(streams.len(), 3);

    assert_eq!(streams[0].id, "1000001");
    assert_eq!(streams[0].name, "Episode 1: The Beginning");
    assert_eq!(streams[0].url, "https://traffic.megaphone.fm/episode1.mp3");
    assert_eq!(streams[0].group, "True Crime Weekly");
    assert_eq!(streams[0].vod_type, "podcast");
    assert_eq!(
        streams[0].logo.as_deref(),
        Some("https://is1-ssl.mzstatic.com/image/thumb/Podcasts/artwork600.jpg")
    );
    assert_eq!(
        streams[0].episode_name.as_deref(),
        Some("The story begins here")
    );
}

#[test]
fn test_parse_episodes_genres() {
    let streams = parse_episodes(sample_search_response().as_bytes());

    let tags = streams[0].tags.as_ref().unwrap();
    assert_eq!(tags.len(), 2);
    assert_eq!(tags[0], "True Crime");
    assert_eq!(tags[1], "Society & Culture");
}

#[test]
fn test_parse_episodes_empty_short_description_becomes_none() {
    let streams = parse_episodes(sample_search_response().as_bytes());
    // Episode 2 has shortDescription: "" -- should be None
    assert!(streams[1].episode_name.is_none());
}

#[test]
fn test_parse_episodes_grouping() {
    let streams = parse_episodes(sample_search_response().as_bytes());

    // First two episodes belong to "True Crime Weekly"
    assert_eq!(streams[0].group, "True Crime Weekly");
    assert_eq!(streams[1].group, "True Crime Weekly");
    // Third episode belongs to "Daily Tech"
    assert_eq!(streams[2].group, "Daily Tech");
}

#[test]
fn test_parse_episodes_url_extraction() {
    let streams = parse_episodes(sample_search_response().as_bytes());

    assert_eq!(streams[0].url, "https://traffic.megaphone.fm/episode1.mp3");
    assert_eq!(streams[1].url, "https://traffic.megaphone.fm/episode2.mp3");
    assert_eq!(streams[2].url, "https://dts.podtrac.com/technews.mp3");
}

// ============================================================
// Tests: Lookup response (skips collection wrapper)
// ============================================================

#[test]
fn test_parse_lookup_skips_collection_wrapper() {
    let streams = parse_episodes(sample_lookup_response().as_bytes());
    // The first result is wrapperType "track" (the podcast itself), should be skipped
    assert_eq!(streams.len(), 2);
    assert_eq!(streams[0].id, "1000010");
    assert_eq!(streams[0].name, "Episode 10: The Verdict");
    assert_eq!(streams[1].id, "1000011");
    assert_eq!(streams[1].name, "Episode 11: Aftermath");
}

// ============================================================
// Tests: Episodes without episodeUrl are filtered out
// ============================================================

#[test]
fn test_parse_episodes_filters_missing_url() {
    let data = r#"{
        "resultCount": 2,
        "results": [
            {
                "wrapperType": "podcastEpisode",
                "trackId": 9001,
                "trackName": "Good Episode",
                "collectionName": "A Podcast",
                "episodeUrl": "https://example.com/good.mp3",
                "artworkUrl600": "https://example.com/art.jpg"
            },
            {
                "wrapperType": "podcastEpisode",
                "trackId": 9002,
                "trackName": "Bad Episode No URL",
                "collectionName": "A Podcast"
            }
        ]
    }"#;

    let streams = parse_episodes(data.as_bytes());
    assert_eq!(streams.len(), 1);
    assert_eq!(streams[0].id, "9001");
}

#[test]
fn test_parse_episodes_filters_empty_url() {
    let data = r#"{
        "resultCount": 1,
        "results": [
            {
                "wrapperType": "podcastEpisode",
                "trackId": 9003,
                "trackName": "Empty URL Episode",
                "collectionName": "A Podcast",
                "episodeUrl": ""
            }
        ]
    }"#;

    let streams = parse_episodes(data.as_bytes());
    assert_eq!(streams.len(), 0);
}

// ============================================================
// Tests: Deduplication
// ============================================================

#[test]
fn test_dedup_streams() {
    let streams = vec![
        Stream {
            id: "100".to_string(),
            name: "Ep A".to_string(),
            url: "https://a.mp3".to_string(),
            group: "Show".to_string(),
            logo: None,
            vod_type: "podcast".to_string(),
            tags: None,
            episode_name: None,
        },
        Stream {
            id: "200".to_string(),
            name: "Ep B".to_string(),
            url: "https://b.mp3".to_string(),
            group: "Show".to_string(),
            logo: None,
            vod_type: "podcast".to_string(),
            tags: None,
            episode_name: None,
        },
        Stream {
            id: "100".to_string(),
            name: "Ep A (duplicate)".to_string(),
            url: "https://a-dup.mp3".to_string(),
            group: "Show".to_string(),
            logo: None,
            vod_type: "podcast".to_string(),
            tags: None,
            episode_name: None,
        },
    ];

    let deduped = dedup_streams(streams);
    assert_eq!(deduped.len(), 2);
    assert_eq!(deduped[0].id, "100");
    assert_eq!(deduped[0].name, "Ep A"); // keeps first occurrence
    assert_eq!(deduped[1].id, "200");
}

#[test]
fn test_dedup_streams_empty() {
    let deduped = dedup_streams(vec![]);
    assert!(deduped.is_empty());
}

// ============================================================
// Tests: URL encoding
// ============================================================

#[test]
fn test_url_encode_simple() {
    assert_eq!(url_encode("hello"), "hello");
}

#[test]
fn test_url_encode_spaces() {
    assert_eq!(url_encode("true crime"), "true%20crime");
}

#[test]
fn test_url_encode_special_chars() {
    assert_eq!(url_encode("news & politics"), "news%20%26%20politics");
}

#[test]
fn test_url_encode_preserves_safe_chars() {
    assert_eq!(url_encode("a-b_c.d~e"), "a-b_c.d~e");
}

// ============================================================
// Tests: Config parsing
// ============================================================

#[test]
fn test_parse_comma_list() {
    let mut config = serde_json::Map::new();
    config.insert(
        "searches".to_string(),
        Value::String("true crime, tech news, comedy".to_string()),
    );

    let result = parse_comma_list(&config, "searches");
    assert_eq!(result, vec!["true crime", "tech news", "comedy"]);
}

#[test]
fn test_parse_comma_list_empty() {
    let config = serde_json::Map::new();
    let result = parse_comma_list(&config, "searches");
    assert!(result.is_empty());
}

#[test]
fn test_parse_comma_list_trims_whitespace() {
    let mut config = serde_json::Map::new();
    config.insert(
        "ids".to_string(),
        Value::String("  123 , 456 ,789  ".to_string()),
    );

    let result = parse_comma_list(&config, "ids");
    assert_eq!(result, vec!["123", "456", "789"]);
}

#[test]
fn test_parse_comma_list_filters_empty() {
    let mut config = serde_json::Map::new();
    config.insert(
        "ids".to_string(),
        Value::String("a,,b,  , c".to_string()),
    );

    let result = parse_comma_list(&config, "ids");
    assert_eq!(result, vec!["a", "b", "c"]);
}

#[test]
fn test_parse_limit_default() {
    let config = serde_json::Map::new();
    assert_eq!(parse_limit(&config), 25);
}

#[test]
fn test_parse_limit_string_value() {
    let mut config = serde_json::Map::new();
    config.insert("limit".to_string(), Value::String("50".to_string()));
    assert_eq!(parse_limit(&config), 50);
}

#[test]
fn test_parse_limit_number_value() {
    let mut config = serde_json::Map::new();
    config.insert("limit".to_string(), serde_json::json!(100));
    assert_eq!(parse_limit(&config), 100);
}

// ============================================================
// Tests: Artwork fallback
// ============================================================

#[test]
fn test_artwork_fallback_to_160() {
    let data = r#"{
        "resultCount": 1,
        "results": [
            {
                "wrapperType": "podcastEpisode",
                "trackId": 7001,
                "trackName": "Fallback Art",
                "collectionName": "Art Test",
                "episodeUrl": "https://example.com/ep.mp3",
                "artworkUrl160": "https://example.com/art160.jpg"
            }
        ]
    }"#;

    let streams = parse_episodes(data.as_bytes());
    assert_eq!(streams.len(), 1);
    assert_eq!(
        streams[0].logo.as_deref(),
        Some("https://example.com/art160.jpg")
    );
}

#[test]
fn test_artwork_fallback_to_100() {
    let data = r#"{
        "resultCount": 1,
        "results": [
            {
                "wrapperType": "podcastEpisode",
                "trackId": 7002,
                "trackName": "Fallback Art 100",
                "collectionName": "Art Test",
                "episodeUrl": "https://example.com/ep.mp3",
                "artworkUrl100": "https://example.com/art100.jpg"
            }
        ]
    }"#;

    let streams = parse_episodes(data.as_bytes());
    assert_eq!(streams.len(), 1);
    assert_eq!(
        streams[0].logo.as_deref(),
        Some("https://example.com/art100.jpg")
    );
}

#[test]
fn test_no_artwork_gives_none() {
    let data = r#"{
        "resultCount": 1,
        "results": [
            {
                "wrapperType": "podcastEpisode",
                "trackId": 7003,
                "trackName": "No Art",
                "collectionName": "Art Test",
                "episodeUrl": "https://example.com/ep.mp3"
            }
        ]
    }"#;

    let streams = parse_episodes(data.as_bytes());
    assert_eq!(streams.len(), 1);
    assert!(streams[0].logo.is_none());
}

// ============================================================
// Tests: Invalid / edge-case JSON
// ============================================================

#[test]
fn test_parse_episodes_invalid_json() {
    let streams = parse_episodes(b"not json at all");
    assert!(streams.is_empty());
}

#[test]
fn test_parse_episodes_empty_results() {
    let data = r#"{"resultCount": 0, "results": []}"#;
    let streams = parse_episodes(data.as_bytes());
    assert!(streams.is_empty());
}

#[test]
fn test_parse_episodes_missing_results_key() {
    let data = r#"{"data": []}"#;
    let streams = parse_episodes(data.as_bytes());
    assert!(streams.is_empty());
}

// ============================================================
// Tests: Grouping correctness across multiple sources
// ============================================================

#[test]
fn test_grouping_across_sources() {
    // Simulate combining results from two different search terms
    let mut all = parse_episodes(sample_search_response().as_bytes());
    all.extend(parse_episodes(sample_lookup_response().as_bytes()));

    let deduped = dedup_streams(all);

    // Count episodes per group
    let tcw_count = deduped.iter().filter(|s| s.group == "True Crime Weekly").count();
    let dt_count = deduped.iter().filter(|s| s.group == "Daily Tech").count();

    // 2 from search + 2 from lookup = 4 True Crime Weekly episodes (all unique IDs)
    assert_eq!(tcw_count, 4);
    // 1 from search
    assert_eq!(dt_count, 1);
    assert_eq!(deduped.len(), 5);
}
