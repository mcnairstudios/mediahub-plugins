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
    // "Carmen - Suite No. 1 - Opéra de Paris" should split on the last " - "
    let (name, group) = parse_title("Carmen - Suite No. 1 - Opera de Paris");
    assert_eq!(name, "Carmen - Suite No. 1");
    assert_eq!(group, "Opera de Paris");
}

#[test]
fn test_parse_title_en_dash_takes_priority_over_hyphen() {
    // If both en-dash and hyphen exist, en-dash wins
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
// RSS XML parsing tests
// ============================================================

fn sample_rss_xml() -> String {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<feed xmlns:yt="http://www.youtube.com/xml/schemas/2015"
      xmlns:media="http://search.yahoo.com/mrss/"
      xmlns="http://www.w3.org/2005/Atom">
 <title>OperaVision</title>
 <entry>
  <yt:videoId>abc123DEF_0</yt:videoId>
  <title>La Traviata Verdi &#8211; Royal Opera House</title>
  <link rel="alternate" href="https://www.youtube.com/watch?v=abc123DEF_0"/>
 </entry>
 <entry>
  <yt:videoId>xyz789GHI_1</yt:videoId>
  <title>Swan Lake Tchaikovsky &#8211; Bolshoi Theatre</title>
  <link rel="alternate" href="https://www.youtube.com/watch?v=xyz789GHI_1"/>
 </entry>
 <entry>
  <yt:videoId>qrs456JKL_2</yt:videoId>
  <title>Some Trailer</title>
  <link rel="alternate" href="https://www.youtube.com/watch?v=qrs456JKL_2"/>
 </entry>
</feed>"#.to_string()
}

#[test]
fn test_parse_rss_xml_count() {
    let xml = sample_rss_xml();
    let items = parse_rss_xml(&xml);
    assert_eq!(items.len(), 3);
}

#[test]
fn test_parse_rss_xml_first_entry() {
    let xml = sample_rss_xml();
    let items = parse_rss_xml(&xml);
    assert_eq!(items[0].0, "abc123DEF_0");
    // Note: &#8211; is the HTML entity for en-dash, but our simple parser returns it as-is
    assert!(items[0].1.contains("La Traviata"));
}

#[test]
fn test_parse_rss_xml_second_entry() {
    let xml = sample_rss_xml();
    let items = parse_rss_xml(&xml);
    assert_eq!(items[1].0, "xyz789GHI_1");
    assert!(items[1].1.contains("Swan Lake"));
}

#[test]
fn test_parse_rss_xml_empty() {
    let items = parse_rss_xml("<feed></feed>");
    assert!(items.is_empty());
}

#[test]
fn test_parse_rss_xml_malformed() {
    let items = parse_rss_xml("not xml at all");
    assert!(items.is_empty());
}

#[test]
fn test_parse_rss_xml_missing_video_id() {
    let xml = r#"<feed><entry><title>No Video ID</title></entry></feed>"#;
    let items = parse_rss_xml(xml);
    assert!(items.is_empty());
}

// ============================================================
// Playlist HTML parsing tests
// ============================================================

fn sample_playlist_html() -> String {
    // Simulates the relevant parts of a YouTube playlist HTML page
    // with ytInitialData containing playlistVideoRenderer objects
    r#"<html><body><script>var ytInitialData = {
        "contents":{"twoColumnBrowseResultsRenderer":{"tabs":[{"tabRenderer":{"content":{"sectionListRenderer":{"contents":[{"itemSectionRenderer":{"contents":[{"playlistVideoListRenderer":{"contents":[
        {"playlistVideoRenderer":{"videoId":"LMn6rDScs_Y","title":{"runs":[{"text":"The Corsair -- Estonian National Ballet"}]},"lengthSeconds":"5400"}},
        {"playlistVideoRenderer":{"videoId":"aBcDeFgHiJk","title":{"runs":[{"text":"Tosca Puccini -- Wiener Staatsoper"}]},"lengthSeconds":"7200"}},
        {"playlistVideoRenderer":{"videoId":"xYz123456_0","title":{"simpleText":"Carmen Bizet -- Opera de Paris"},"lengthSeconds":"6000"}}
        ]}}]}}]}}}}]}}
    };</script></body></html>"#.to_string()
}

#[test]
fn test_parse_playlist_html_count() {
    let html = sample_playlist_html();
    let items = parse_playlist_html(&html);
    assert_eq!(items.len(), 3);
}

#[test]
fn test_parse_playlist_html_video_ids() {
    let html = sample_playlist_html();
    let items = parse_playlist_html(&html);
    assert_eq!(items[0].0, "LMn6rDScs_Y");
    assert_eq!(items[1].0, "aBcDeFgHiJk");
    assert_eq!(items[2].0, "xYz123456_0");
}

#[test]
fn test_parse_playlist_html_titles() {
    let html = sample_playlist_html();
    let items = parse_playlist_html(&html);
    assert_eq!(items[0].1, "The Corsair -- Estonian National Ballet");
    assert_eq!(items[1].1, "Tosca Puccini -- Wiener Staatsoper");
    assert_eq!(items[2].1, "Carmen Bizet -- Opera de Paris");
}

#[test]
fn test_parse_playlist_html_deduplicates() {
    // Same video ID appearing twice should only be returned once
    let html = r#"<html>
        {"playlistVideoRenderer":{"videoId":"LMn6rDScs_Y","title":{"runs":[{"text":"Title A"}]}}}
        {"playlistVideoRenderer":{"videoId":"LMn6rDScs_Y","title":{"runs":[{"text":"Title A Again"}]}}}
        {"playlistVideoRenderer":{"videoId":"aBcDeFgHiJk","title":{"runs":[{"text":"Title B"}]}}}
    </html>"#;
    let items = parse_playlist_html(html);
    assert_eq!(items.len(), 2);
    assert_eq!(items[0].0, "LMn6rDScs_Y");
    assert_eq!(items[1].0, "aBcDeFgHiJk");
}

#[test]
fn test_parse_playlist_html_empty() {
    let items = parse_playlist_html("<html><body></body></html>");
    assert!(items.is_empty());
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
// Stream merging tests
// ============================================================

#[test]
fn test_merge_streams_deduplicates() {
    let primary = vec![
        Stream {
            id: "vid1".to_string(),
            name: "Primary Title".to_string(),
            url: "https://www.youtube.com/watch?v=vid1".to_string(),
            group: "Group A".to_string(),
            logo: None,
            vod_type: "youtube".to_string(),
            tags: None,
        },
    ];
    let secondary = vec![
        Stream {
            id: "vid1".to_string(),
            name: "Secondary Title".to_string(),
            url: "https://www.youtube.com/watch?v=vid1".to_string(),
            group: "Group B".to_string(),
            logo: None,
            vod_type: "youtube".to_string(),
            tags: None,
        },
        Stream {
            id: "vid2".to_string(),
            name: "New Entry".to_string(),
            url: "https://www.youtube.com/watch?v=vid2".to_string(),
            group: "Group C".to_string(),
            logo: None,
            vod_type: "youtube".to_string(),
            tags: None,
        },
    ];

    let merged = merge_streams(primary, secondary);
    assert_eq!(merged.len(), 2);
    // Primary takes precedence for vid1
    assert_eq!(merged[0].name, "Primary Title");
    assert_eq!(merged[0].group, "Group A");
    assert_eq!(merged[1].id, "vid2");
}

#[test]
fn test_merge_streams_no_overlap() {
    let a = vec![Stream {
        id: "a".to_string(),
        name: "A".to_string(),
        url: "u".to_string(),
        group: "G".to_string(),
        logo: None,
        vod_type: "youtube".to_string(),
        tags: None,
    }];
    let b = vec![Stream {
        id: "b".to_string(),
        name: "B".to_string(),
        url: "u".to_string(),
        group: "G".to_string(),
        logo: None,
        vod_type: "youtube".to_string(),
        tags: None,
    }];

    let merged = merge_streams(a, b);
    assert_eq!(merged.len(), 2);
}

#[test]
fn test_merge_streams_empty() {
    let merged = merge_streams(Vec::new(), Vec::new());
    assert!(merged.is_empty());
}

// ============================================================
// JSON string extraction tests
// ============================================================

#[test]
fn test_extract_json_string_value_basic() {
    let text = r#"{"videoId":"LMn6rDScs_Y","other":"stuff"}"#;
    let result = extract_json_string_value(text, "\"videoId\":\"");
    assert_eq!(result, Some("LMn6rDScs_Y".to_string()));
}

#[test]
fn test_extract_json_string_value_missing() {
    let text = r#"{"other":"stuff"}"#;
    let result = extract_json_string_value(text, "\"videoId\":\"");
    assert_eq!(result, None);
}

#[test]
fn test_extract_json_string_value_too_long() {
    let text = r#"{"videoId":"this_is_way_too_long_to_be_a_valid_video_id_string"}"#;
    let result = extract_json_string_value(text, "\"videoId\":\"");
    assert_eq!(result, None);
}

// ============================================================
// XML tag content extraction tests
// ============================================================

#[test]
fn test_extract_tag_content_basic() {
    let result = extract_tag_content("<root><yt:videoId>abc123</yt:videoId></root>", "<yt:videoId>", "</yt:videoId>");
    assert_eq!(result, Some("abc123".to_string()));
}

#[test]
fn test_extract_tag_content_missing_open() {
    let result = extract_tag_content("<root></root>", "<yt:videoId>", "</yt:videoId>");
    assert_eq!(result, None);
}

#[test]
fn test_extract_tag_content_missing_close() {
    let result = extract_tag_content("<root><yt:videoId>abc123</root>", "<yt:videoId>", "</yt:videoId>");
    assert_eq!(result, None);
}

// ============================================================
// Unescape tests
// ============================================================

#[test]
fn test_unescape_json_string() {
    assert_eq!(unescape_json_string("hello \\u0026 world"), "hello & world");
    assert_eq!(unescape_json_string("a \\\"quote\\\""), "a \"quote\"");
    assert_eq!(unescape_json_string("no escapes"), "no escapes");
}

// ============================================================
// End-to-end: RSS to streams
// ============================================================

#[test]
fn test_rss_to_streams_integration() {
    let xml = sample_rss_xml();
    let items = parse_rss_xml(&xml);
    let streams = build_streams(&items);

    assert_eq!(streams.len(), 3);
    assert_eq!(streams[0].url, "https://www.youtube.com/watch?v=abc123DEF_0");
    assert_eq!(streams[0].vod_type, "youtube");
    assert!(streams[0].logo.as_ref().unwrap().contains("abc123DEF_0"));
}

// ============================================================
// End-to-end: Playlist HTML to streams
// ============================================================

#[test]
fn test_playlist_to_streams_integration() {
    let html = sample_playlist_html();
    let items = parse_playlist_html(&html);
    let streams = build_streams(&items);

    assert_eq!(streams.len(), 3);

    // First stream: The Corsair
    assert_eq!(streams[0].id, "LMn6rDScs_Y");
    assert_eq!(streams[0].name, "The Corsair");
    assert_eq!(streams[0].group, "Estonian National Ballet");
    assert!(streams[0].tags.as_ref().unwrap().contains(&"ballet".to_string()));

    // Second stream: Tosca
    assert_eq!(streams[1].name, "Tosca Puccini");
    assert_eq!(streams[1].group, "Wiener Staatsoper");

    // Third stream: Carmen
    assert_eq!(streams[2].name, "Carmen Bizet");
    assert_eq!(streams[2].group, "Opera de Paris");
}
