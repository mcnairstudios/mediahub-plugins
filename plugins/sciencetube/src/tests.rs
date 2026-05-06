use super::*;

// ============================================================
// Feed URL construction tests
// ============================================================

#[test]
fn test_feed_url() {
    assert_eq!(
        feed_url("UC7_gcs09iThXybpVgjHZ_7g"),
        "https://www.youtube.com/feeds/videos.xml?channel_id=UC7_gcs09iThXybpVgjHZ_7g"
    );
}

#[test]
fn test_video_url() {
    assert_eq!(
        video_url("dQw4w9WgXcQ"),
        "https://www.youtube.com/watch?v=dQw4w9WgXcQ"
    );
}

#[test]
fn test_thumbnail_url() {
    assert_eq!(
        thumbnail_url("dQw4w9WgXcQ"),
        "https://i.ytimg.com/vi/dQw4w9WgXcQ/hqdefault.jpg"
    );
}

// ============================================================
// XML extraction tests
// ============================================================

#[test]
fn test_extract_entries_multiple() {
    let xml = r#"<feed>
<entry><yt:videoId>abc123</yt:videoId><title>Video One</title></entry>
<entry><yt:videoId>def456</yt:videoId><title>Video Two</title></entry>
<entry><yt:videoId>ghi789</yt:videoId><title>Video Three</title></entry>
</feed>"#;
    let entries = extract_entries(xml);
    assert_eq!(entries.len(), 3);
    assert!(entries[0].contains("abc123"));
    assert!(entries[1].contains("def456"));
    assert!(entries[2].contains("ghi789"));
}

#[test]
fn test_extract_entries_empty() {
    let xml = "<feed></feed>";
    let entries = extract_entries(xml);
    assert!(entries.is_empty());
}

#[test]
fn test_extract_entries_no_closing_tag() {
    let xml = "<feed><entry><yt:videoId>abc</yt:videoId></feed>";
    let entries = extract_entries(xml);
    assert!(entries.is_empty());
}

#[test]
fn test_extract_tag_simple() {
    let xml = "<entry><title>Hello World</title></entry>";
    assert_eq!(extract_tag(xml, "title"), Some("Hello World".to_string()));
}

#[test]
fn test_extract_tag_with_namespace() {
    let xml = "<entry><yt:videoId>abc123</yt:videoId></entry>";
    assert_eq!(extract_tag(xml, "yt:videoId"), Some("abc123".to_string()));
}

#[test]
fn test_extract_tag_missing() {
    let xml = "<entry><title>Hello</title></entry>";
    assert_eq!(extract_tag(xml, "description"), None);
}

#[test]
fn test_extract_tag_with_whitespace() {
    let xml = "<entry><title>  Trimmed Title  </title></entry>";
    assert_eq!(extract_tag(xml, "title"), Some("Trimmed Title".to_string()));
}

#[test]
fn test_extract_attr() {
    let xml = r#"<media:thumbnail url="https://i.ytimg.com/vi/abc/hq.jpg" width="480"/>"#;
    assert_eq!(
        extract_attr(xml, "media:thumbnail", "url"),
        Some("https://i.ytimg.com/vi/abc/hq.jpg".to_string())
    );
}

#[test]
fn test_extract_attr_missing_tag() {
    let xml = "<title>Hello</title>";
    assert_eq!(extract_attr(xml, "media:thumbnail", "url"), None);
}

#[test]
fn test_extract_attr_missing_attr() {
    let xml = r#"<media:thumbnail width="480"/>"#;
    assert_eq!(extract_attr(xml, "media:thumbnail", "url"), None);
}

// ============================================================
// Entry to stream tests
// ============================================================

#[test]
fn test_entry_to_stream_full() {
    let entry = r#"<entry>
<yt:videoId>dQw4w9WgXcQ</yt:videoId>
<title>Never Gonna Give You Up</title>
<published>2009-10-25T06:57:33+00:00</published>
<media:group>
<media:thumbnail url="https://i.ytimg.com/vi/dQw4w9WgXcQ/hqdefault.jpg" width="480" height="360"/>
</media:group>
</entry>"#;

    let stream = entry_to_stream(entry, "Veritasium").unwrap();
    assert_eq!(stream.id, "yt-dQw4w9WgXcQ");
    assert_eq!(stream.name, "Never Gonna Give You Up");
    assert_eq!(stream.url, "https://www.youtube.com/watch?v=dQw4w9WgXcQ");
    assert_eq!(stream.group, "Veritasium");
    assert_eq!(
        stream.logo,
        Some("https://i.ytimg.com/vi/dQw4w9WgXcQ/hqdefault.jpg".to_string())
    );
    assert_eq!(stream.vod_type, "movie");
    assert_eq!(stream.year, Some("2009".to_string()));
    assert_eq!(stream.tags, Some(vec!["youtube".to_string()]));
    assert_eq!(stream.episode_name, Some("Oct 25, 2009".to_string()));
}

#[test]
fn test_entry_to_stream_minimal() {
    let entry = r#"<entry>
<yt:videoId>xyz789</yt:videoId>
</entry>"#;

    let stream = entry_to_stream(entry, "3Blue1Brown").unwrap();
    assert_eq!(stream.id, "yt-xyz789");
    assert_eq!(stream.name, "xyz789"); // fallback to video ID
    assert_eq!(stream.group, "3Blue1Brown");
    assert_eq!(
        stream.logo,
        Some("https://i.ytimg.com/vi/xyz789/hqdefault.jpg".to_string())
    );
    assert_eq!(stream.year, None);
    assert_eq!(stream.episode_name, None);
}

#[test]
fn test_entry_to_stream_no_video_id() {
    let entry = "<entry><title>No Video ID</title></entry>";
    let result = entry_to_stream(entry, "Channel");
    assert!(result.is_none());
}

#[test]
fn test_entry_to_stream_html_entities() {
    let entry = r#"<entry>
<yt:videoId>abc123</yt:videoId>
<title>What&apos;s 1 + 1? It&amp;s &gt; 1 &amp; &lt; 3</title>
<published>2024-03-15T12:00:00+00:00</published>
</entry>"#;

    let stream = entry_to_stream(entry, "Numberphile").unwrap();
    assert_eq!(stream.name, "What's 1 + 1? It&s > 1 & < 3");
}

// ============================================================
// Full feed parsing tests
// ============================================================

#[test]
fn test_parse_feed_multiple_entries() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<feed xmlns:yt="http://www.youtube.com/xml/schemas/2015" xmlns:media="http://search.yahoo.com/mrss/">
<title>Veritasium</title>
<entry>
<yt:videoId>vid001</yt:videoId>
<title>Video One</title>
<published>2025-01-10T10:00:00+00:00</published>
<media:group>
<media:thumbnail url="https://i.ytimg.com/vi/vid001/hqdefault.jpg" width="480" height="360"/>
</media:group>
</entry>
<entry>
<yt:videoId>vid002</yt:videoId>
<title>Video Two</title>
<published>2025-01-05T08:00:00+00:00</published>
<media:group>
<media:thumbnail url="https://i.ytimg.com/vi/vid002/hqdefault.jpg" width="480" height="360"/>
</media:group>
</entry>
</feed>"#;

    let streams = parse_feed(xml, "Veritasium");
    assert_eq!(streams.len(), 2);
    assert_eq!(streams[0].id, "yt-vid001");
    assert_eq!(streams[0].name, "Video One");
    assert_eq!(streams[0].group, "Veritasium");
    assert_eq!(streams[1].id, "yt-vid002");
    assert_eq!(streams[1].name, "Video Two");
}

#[test]
fn test_parse_feed_empty() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<feed><title>Empty Channel</title></feed>"#;

    let streams = parse_feed(xml, "Empty");
    assert!(streams.is_empty());
}

// ============================================================
// Date formatting tests
// ============================================================

#[test]
fn test_format_date_valid() {
    assert_eq!(format_date("2025-01-14"), "Jan 14, 2025");
    assert_eq!(format_date("2024-12-31"), "Dec 31, 2024");
    assert_eq!(format_date("2023-06-01"), "Jun 01, 2023");
}

#[test]
fn test_format_date_short_string() {
    assert_eq!(format_date("2025"), "2025");
}

#[test]
fn test_format_date_invalid_month() {
    assert_eq!(format_date("2025-13-01"), "2025-13-01");
}

// ============================================================
// XML unescape tests
// ============================================================

#[test]
fn test_unescape_xml_entities() {
    assert_eq!(unescape_xml("Hello &amp; World"), "Hello & World");
    assert_eq!(unescape_xml("&lt;tag&gt;"), "<tag>");
    assert_eq!(unescape_xml("&quot;quoted&quot;"), "\"quoted\"");
    assert_eq!(unescape_xml("it&apos;s"), "it's");
    assert_eq!(unescape_xml("it&#39;s"), "it's");
}

#[test]
fn test_unescape_xml_no_entities() {
    assert_eq!(unescape_xml("plain text"), "plain text");
}

// ============================================================
// Channel definitions tests
// ============================================================

#[test]
fn test_channel_count() {
    assert_eq!(CHANNELS.len(), 14);
}

#[test]
fn test_channel_ids_unique() {
    let mut ids: Vec<&str> = CHANNELS.iter().map(|c| c.id).collect();
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), CHANNELS.len());
}

#[test]
fn test_channel_names_unique() {
    let mut names: Vec<&str> = CHANNELS.iter().map(|c| c.name).collect();
    names.sort();
    names.dedup();
    assert_eq!(names.len(), CHANNELS.len());
}

#[test]
fn test_known_channels_present() {
    let ids: Vec<&str> = CHANNELS.iter().map(|c| c.id).collect();
    assert!(ids.contains(&"UC7_gcs09iThXybpVgjHZ_7g")); // PBS Space Time
    assert!(ids.contains(&"UCHnyfMqiRRG1u-2MsSQLbXA")); // Veritasium
    assert!(ids.contains(&"UCYO_jab_esuFRV4b17AJtAw")); // 3Blue1Brown
    assert!(ids.contains(&"UCsXVk37bltHxD1rDPwtNM8Q")); // Kurzgesagt
}
