use super::*;

// ============================================================
// Sample RSS feed data for testing
// ============================================================

fn sample_rss_feed() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<rss xmlns:itunes="http://www.itunes.com/dtds/podcast-1.0.dtd" version="2.0">
  <channel>
    <title>TED Talks Daily</title>
    <link>https://www.ted.com/talks</link>
    <item>
      <title>The power of introverts | Susan Cain</title>
      <link>https://www.ted.com/talks/susan_cain_the_power_of_introverts</link>
      <enclosure url="https://download.ted.com/talks/SusanCain_2012-480p.mp4" length="43250000" type="video/mp4"/>
      <itunes:duration>00:19:04</itunes:duration>
      <itunes:summary>In a culture where being social and outgoing are prized above all else, it can be difficult, even shameful, to be an introvert.</itunes:summary>
      <itunes:image href="https://pi.tedcdn.com/r/talkstar-photos.s3.amazonaws.com/uploads/susan_cain.jpg"/>
      <category>Psychology</category>
    </item>
    <item>
      <title>Do schools kill creativity? | Sir Ken Robinson</title>
      <link>https://www.ted.com/talks/sir_ken_robinson_do_schools_kill_creativity</link>
      <enclosure url="https://download.ted.com/talks/KenRobinson_2006-480p.mp4" length="58120000" type="video/mp4"/>
      <itunes:duration>19:24</itunes:duration>
      <itunes:summary>Sir Ken Robinson makes an entertaining and profoundly moving case for creating an education system that nurtures creativity.</itunes:summary>
      <itunes:image href="https://pi.tedcdn.com/r/talkstar-photos.s3.amazonaws.com/uploads/ken_robinson.jpg"/>
      <category>Education</category>
    </item>
    <item>
      <title>Your body language may shape who you are | Amy Cuddy</title>
      <link>https://www.ted.com/talks/amy_cuddy_your_body_language_may_shape_who_you_are</link>
      <enclosure url="https://download.ted.com/talks/AmyCuddy_2012G-480p.mp4" length="61530000" type="video/mp4"/>
      <itunes:duration>00:21:02</itunes:duration>
      <itunes:summary>Body language affects how others see us, but it may also change how we see ourselves.</itunes:summary>
      <itunes:image href="https://pi.tedcdn.com/r/talkstar-photos.s3.amazonaws.com/uploads/amy_cuddy.jpg"/>
      <category>Psychology</category>
    </item>
    <item>
      <title>How great leaders inspire action | Simon Sinek</title>
      <link>https://www.ted.com/talks/simon_sinek_how_great_leaders_inspire_action</link>
      <enclosure url="https://download.ted.com/talks/SimonSinek_2009X-480p.mp4" length="54800000" type="video/mp4"/>
      <itunes:duration>1110</itunes:duration>
      <itunes:summary>Simon Sinek has a simple but powerful model for inspirational leadership.</itunes:summary>
      <itunes:image href="https://pi.tedcdn.com/r/talkstar-photos.s3.amazonaws.com/uploads/simon_sinek.jpg"/>
      <category>Leadership</category>
    </item>
  </channel>
</rss>"#
}

fn sample_rss_no_category() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <item>
      <title>A talk without a category</title>
      <link>https://www.ted.com/talks/no_category_talk</link>
      <enclosure url="https://download.ted.com/talks/NoCategory-480p.mp4" type="video/mp4"/>
      <itunes:duration>15:30</itunes:duration>
      <itunes:summary>This talk has no category tag.</itunes:summary>
    </item>
  </channel>
</rss>"#
}

fn sample_rss_cdata() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <item>
      <title><![CDATA[Why we &amp; our world need rest | Claudia Hammond]]></title>
      <link>https://www.ted.com/talks/claudia_hammond_rest</link>
      <enclosure url="https://download.ted.com/talks/ClaudiaHammond-480p.mp4" type="video/mp4"/>
      <itunes:duration>12:45</itunes:duration>
      <itunes:summary><![CDATA[Claudia Hammond explores why rest isn&apos;t laziness.]]></itunes:summary>
      <category><![CDATA[Health & Wellness]]></category>
    </item>
  </channel>
</rss>"#
}

// ============================================================
// Tests: Basic RSS parsing
// ============================================================

#[test]
fn test_parse_rss_items_count() {
    let streams = parse_rss_items(sample_rss_feed());
    assert_eq!(streams.len(), 4);
}

#[test]
fn test_parse_rss_title_extraction() {
    let streams = parse_rss_items(sample_rss_feed());
    assert_eq!(streams[0].name, "The power of introverts | Susan Cain");
    assert_eq!(streams[1].name, "Do schools kill creativity? | Sir Ken Robinson");
    assert_eq!(streams[2].name, "Your body language may shape who you are | Amy Cuddy");
    assert_eq!(streams[3].name, "How great leaders inspire action | Simon Sinek");
}

#[test]
fn test_parse_rss_url_extraction() {
    let streams = parse_rss_items(sample_rss_feed());
    assert_eq!(streams[0].url, "https://download.ted.com/talks/SusanCain_2012-480p.mp4");
    assert_eq!(streams[1].url, "https://download.ted.com/talks/KenRobinson_2006-480p.mp4");
    assert_eq!(streams[2].url, "https://download.ted.com/talks/AmyCuddy_2012G-480p.mp4");
    assert_eq!(streams[3].url, "https://download.ted.com/talks/SimonSinek_2009X-480p.mp4");
}

#[test]
fn test_parse_rss_grouping_by_category() {
    let streams = parse_rss_items(sample_rss_feed());
    assert_eq!(streams[0].group, "Psychology");
    assert_eq!(streams[1].group, "Education");
    assert_eq!(streams[2].group, "Psychology");
    assert_eq!(streams[3].group, "Leadership");
}

#[test]
fn test_parse_rss_default_group_when_no_category() {
    let streams = parse_rss_items(sample_rss_no_category());
    assert_eq!(streams.len(), 1);
    assert_eq!(streams[0].group, "TED Talks");
}

#[test]
fn test_parse_rss_vod_type() {
    let streams = parse_rss_items(sample_rss_feed());
    for s in &streams {
        assert_eq!(s.vod_type, "movie");
    }
}

#[test]
fn test_parse_rss_image_extraction() {
    let streams = parse_rss_items(sample_rss_feed());
    assert_eq!(
        streams[0].logo.as_deref(),
        Some("https://pi.tedcdn.com/r/talkstar-photos.s3.amazonaws.com/uploads/susan_cain.jpg")
    );
    assert_eq!(
        streams[1].logo.as_deref(),
        Some("https://pi.tedcdn.com/r/talkstar-photos.s3.amazonaws.com/uploads/ken_robinson.jpg")
    );
}

#[test]
fn test_parse_rss_no_image_gives_none() {
    let streams = parse_rss_items(sample_rss_no_category());
    assert!(streams[0].logo.is_none());
}

#[test]
fn test_parse_rss_summary_as_episode_name() {
    let streams = parse_rss_items(sample_rss_feed());
    assert_eq!(
        streams[0].episode_name.as_deref(),
        Some("In a culture where being social and outgoing are prized above all else, it can be difficult, even shameful, to be an introvert.")
    );
}

#[test]
fn test_parse_rss_id_from_link() {
    let streams = parse_rss_items(sample_rss_feed());
    assert_eq!(streams[0].id, "susan_cain_the_power_of_introverts");
    assert_eq!(streams[1].id, "sir_ken_robinson_do_schools_kill_creativity");
}

// ============================================================
// Tests: Duration normalization
// ============================================================

#[test]
fn test_normalize_duration_hhmmss() {
    assert_eq!(normalize_duration("00:19:04"), "19");
    assert_eq!(normalize_duration("01:02:30"), "63");
    assert_eq!(normalize_duration("00:00:45"), "1"); // 45s rounds up
}

#[test]
fn test_normalize_duration_mmss() {
    assert_eq!(normalize_duration("19:24"), "19");
    assert_eq!(normalize_duration("05:45"), "6"); // 45s rounds up
    assert_eq!(normalize_duration("15:10"), "15");
}

#[test]
fn test_normalize_duration_seconds_plain() {
    // 1110 seconds = 18.5 minutes -> 18
    assert_eq!(normalize_duration("1110"), "18");
    // 600 seconds = 10 minutes
    assert_eq!(normalize_duration("600"), "10");
}

#[test]
fn test_normalize_duration_small_number_is_minutes() {
    assert_eq!(normalize_duration("18"), "18");
    assert_eq!(normalize_duration("5"), "5");
}

#[test]
fn test_parse_rss_duration_tags() {
    let streams = parse_rss_items(sample_rss_feed());
    // "00:19:04" -> "19min"
    assert_eq!(streams[0].tags.as_ref().unwrap(), &vec!["19min".to_string()]);
    // "19:24" -> "19min"
    assert_eq!(streams[1].tags.as_ref().unwrap(), &vec!["19min".to_string()]);
    // "1110" seconds -> "18min"
    assert_eq!(streams[3].tags.as_ref().unwrap(), &vec!["18min".to_string()]);
}

// ============================================================
// Tests: CDATA and XML entity handling
// ============================================================

#[test]
fn test_parse_rss_cdata_title() {
    let streams = parse_rss_items(sample_rss_cdata());
    assert_eq!(streams[0].name, "Why we & our world need rest | Claudia Hammond");
}

#[test]
fn test_parse_rss_cdata_summary() {
    let streams = parse_rss_items(sample_rss_cdata());
    assert_eq!(
        streams[0].episode_name.as_deref(),
        Some("Claudia Hammond explores why rest isn't laziness.")
    );
}

#[test]
fn test_parse_rss_cdata_category() {
    let streams = parse_rss_items(sample_rss_cdata());
    assert_eq!(streams[0].group, "Health & Wellness");
}

#[test]
fn test_decode_xml_entities() {
    assert_eq!(decode_xml_entities("&amp;"), "&");
    assert_eq!(decode_xml_entities("&lt;b&gt;"), "<b>");
    assert_eq!(decode_xml_entities("&quot;hello&quot;"), "\"hello\"");
    assert_eq!(decode_xml_entities("it&apos;s"), "it's");
    assert_eq!(decode_xml_entities("it&#39;s"), "it's");
    assert_eq!(decode_xml_entities("no entities"), "no entities");
}

#[test]
fn test_strip_cdata() {
    assert_eq!(strip_cdata("<![CDATA[hello world]]>"), "hello world");
    assert_eq!(strip_cdata("  <![CDATA[trimmed]]>  "), "trimmed");
    assert_eq!(strip_cdata("plain text"), "plain text");
    assert_eq!(strip_cdata(""), "");
}

// ============================================================
// Tests: XML tag and attribute extraction
// ============================================================

#[test]
fn test_extract_tag_simple() {
    assert_eq!(extract_tag("<title>Hello</title>", "title"), "Hello");
}

#[test]
fn test_extract_tag_with_attributes() {
    let xml = r#"<item><title type="text">My Title</title></item>"#;
    assert_eq!(extract_tag(xml, "title"), "My Title");
}

#[test]
fn test_extract_tag_not_found() {
    assert_eq!(extract_tag("<foo>bar</foo>", "title"), "");
}

#[test]
fn test_extract_tag_namespaced() {
    let xml = r#"<itunes:duration>00:15:30</itunes:duration>"#;
    assert_eq!(extract_tag(xml, "itunes:duration"), "00:15:30");
}

#[test]
fn test_extract_attr() {
    let tag = r#"<enclosure url="https://example.com/video.mp4" type="video/mp4"/>"#;
    assert_eq!(extract_attr(tag, "url"), "https://example.com/video.mp4");
    assert_eq!(extract_attr(tag, "type"), "video/mp4");
}

#[test]
fn test_extract_attr_not_found() {
    let tag = r#"<enclosure url="https://example.com/video.mp4"/>"#;
    assert_eq!(extract_attr(tag, "missing"), "");
}

#[test]
fn test_extract_enclosure_url() {
    let item = r#"<item>
      <title>Test</title>
      <enclosure url="https://download.ted.com/talks/Test-480p.mp4" length="12345" type="video/mp4"/>
    </item>"#;
    assert_eq!(
        extract_enclosure_url(item),
        "https://download.ted.com/talks/Test-480p.mp4"
    );
}

#[test]
fn test_extract_enclosure_url_not_found() {
    let item = r#"<item><title>No enclosure</title></item>"#;
    assert_eq!(extract_enclosure_url(item), "");
}

// ============================================================
// Tests: Edge cases
// ============================================================

#[test]
fn test_parse_rss_empty_xml() {
    let streams = parse_rss_items("");
    assert!(streams.is_empty());
}

#[test]
fn test_parse_rss_no_items() {
    let xml = r#"<?xml version="1.0"?><rss><channel><title>Empty</title></channel></rss>"#;
    let streams = parse_rss_items(xml);
    assert!(streams.is_empty());
}

#[test]
fn test_parse_rss_item_without_enclosure_skipped() {
    let xml = r#"<rss><channel>
    <item>
      <title>No Video</title>
      <link>https://www.ted.com/talks/no_video</link>
    </item>
    <item>
      <title>Has Video</title>
      <link>https://www.ted.com/talks/has_video</link>
      <enclosure url="https://download.ted.com/talks/HasVideo-480p.mp4" type="video/mp4"/>
    </item>
  </channel></rss>"#;

    let streams = parse_rss_items(xml);
    assert_eq!(streams.len(), 1);
    assert_eq!(streams[0].name, "Has Video");
}

#[test]
fn test_parse_rss_item_without_title_skipped() {
    let xml = r#"<rss><channel>
    <item>
      <enclosure url="https://download.ted.com/talks/NoTitle-480p.mp4" type="video/mp4"/>
    </item>
  </channel></rss>"#;

    let streams = parse_rss_items(xml);
    assert!(streams.is_empty());
}

#[test]
fn test_parse_rss_deduplicates_by_id() {
    let xml = r#"<rss><channel>
    <item>
      <title>Same Talk</title>
      <link>https://www.ted.com/talks/same_talk</link>
      <enclosure url="https://download.ted.com/talks/Same-480p.mp4" type="video/mp4"/>
    </item>
    <item>
      <title>Same Talk (duplicate)</title>
      <link>https://www.ted.com/talks/same_talk</link>
      <enclosure url="https://download.ted.com/talks/Same-720p.mp4" type="video/mp4"/>
    </item>
  </channel></rss>"#;

    let streams = parse_rss_items(xml);
    assert_eq!(streams.len(), 1);
    assert_eq!(streams[0].name, "Same Talk");
}

#[test]
fn test_parse_rss_long_summary_truncated() {
    let long_summary = "A".repeat(400);
    let xml = format!(
        r#"<rss><channel>
    <item>
      <title>Long Summary Talk</title>
      <link>https://www.ted.com/talks/long_summary</link>
      <enclosure url="https://download.ted.com/talks/Long-480p.mp4" type="video/mp4"/>
      <itunes:summary>{}</itunes:summary>
    </item>
  </channel></rss>"#,
        long_summary
    );

    let streams = parse_rss_items(&xml);
    assert_eq!(streams.len(), 1);
    let ep = streams[0].episode_name.as_ref().unwrap();
    assert!(ep.len() <= 303); // 297 chars + "..."
    assert!(ep.ends_with("..."));
}
