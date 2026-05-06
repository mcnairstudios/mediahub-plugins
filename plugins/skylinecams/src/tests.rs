use super::*;

// ============================================================
// Test: extract_cam_links
// ============================================================

#[test]
fn test_extract_cam_links_basic() {
    let html = r#"
        <div class="webcam-list">
            <a href="/en/webcam/italy/lazio/roma/trevi-fountain.html">Trevi Fountain</a>
            <a href="/en/webcam/spain/cataluna/barcelona/playa-de-barceloneta.html">Barcelona Beach</a>
            <a href="/en/webcam/greece/attica/athens/acropolis.html">Acropolis</a>
        </div>
    "#;

    let links = extract_cam_links(html);
    assert_eq!(links.len(), 3);
    assert_eq!(links[0], "/en/webcam/italy/lazio/roma/trevi-fountain.html");
    assert_eq!(links[1], "/en/webcam/spain/cataluna/barcelona/playa-de-barceloneta.html");
    assert_eq!(links[2], "/en/webcam/greece/attica/athens/acropolis.html");
}

#[test]
fn test_extract_cam_links_skips_index_pages() {
    // Links to country/category index pages should be filtered out
    // because they have fewer than 4 path segments
    let html = r#"
        <a href="/en/webcam/italy.html">Italy</a>
        <a href="/en/webcam/italy/lazio.html">Lazio</a>
        <a href="/en/webcam/italy/lazio/roma/colosseum.html">Colosseum</a>
    "#;

    let links = extract_cam_links(html);
    assert_eq!(links.len(), 1);
    assert_eq!(links[0], "/en/webcam/italy/lazio/roma/colosseum.html");
}

#[test]
fn test_extract_cam_links_deduplicates() {
    let html = r#"
        <a href="/en/webcam/italy/lazio/roma/trevi-fountain.html">Link 1</a>
        <a href="/en/webcam/italy/lazio/roma/trevi-fountain.html">Link 2</a>
    "#;

    let links = extract_cam_links(html);
    assert_eq!(links.len(), 1);
}

#[test]
fn test_extract_cam_links_empty_html() {
    let links = extract_cam_links("");
    assert!(links.is_empty());
}

#[test]
fn test_extract_cam_links_no_webcam_links() {
    let html = r#"
        <a href="/en/top-live-cams.html">Top Cams</a>
        <a href="/en/about.html">About</a>
    "#;

    let links = extract_cam_links(html);
    assert!(links.is_empty());
}

// ============================================================
// Test: extract_hls_token
// ============================================================

#[test]
fn test_extract_hls_token_single_quotes() {
    let html = r#"
        <script>
            var player = new Clappr.Player({
                source:'livee.m3u8?a=abc123def456',
                parentId: '#player'
            });
        </script>
    "#;

    let token = extract_hls_token(html);
    assert_eq!(token, Some("abc123def456".to_string()));
}

#[test]
fn test_extract_hls_token_double_quotes() {
    let html = r#"
        source:"livee.m3u8?a=xyz789token",
    "#;

    let token = extract_hls_token(html);
    assert_eq!(token, Some("xyz789token".to_string()));
}

#[test]
fn test_extract_hls_token_with_spaces() {
    let html = r#"
        source: 'livee.m3u8?a=spaced_token_123',
    "#;

    let token = extract_hls_token(html);
    assert_eq!(token, Some("spaced_token_123".to_string()));
}

#[test]
fn test_extract_hls_token_not_found() {
    let html = r#"
        <script>
            var player = new Clappr.Player({
                source:'some-other-stream.m3u8',
            });
        </script>
    "#;

    let token = extract_hls_token(html);
    assert!(token.is_none());
}

#[test]
fn test_extract_hls_token_realistic() {
    // Simulate a realistic HTML snippet from SkylineWebcams
    let html = r#"
        <html><head><title>Trevi Fountain - Rome | SkylineWebcams</title></head>
        <body>
        <script>
        var player = new Clappr.Player({
            source:'livee.m3u8?a=bG9uZy1iYXNlNjQtdG9rZW4tZm9yLXRyZXZpLWZvdW50YWlu',
            poster:'/img/poster.jpg',
            nkey:'286.jpg',
            parentId:'#player',
            autoPlay:true
        });
        </script>
        </body></html>
    "#;

    let token = extract_hls_token(html);
    assert_eq!(
        token,
        Some("bG9uZy1iYXNlNjQtdG9rZW4tZm9yLXRyZXZpLWZvdW50YWlu".to_string())
    );
}

#[test]
fn test_extract_hls_token_empty_html() {
    assert!(extract_hls_token("").is_none());
}

// ============================================================
// Test: extract_nkey
// ============================================================

#[test]
fn test_extract_nkey_basic() {
    let html = "nkey:'286.jpg'";
    assert_eq!(extract_nkey(html), Some("286".to_string()));
}

#[test]
fn test_extract_nkey_with_spaces() {
    let html = "nkey: '1234.jpg'";
    assert_eq!(extract_nkey(html), Some("1234".to_string()));
}

#[test]
fn test_extract_nkey_double_quotes() {
    let html = r#"nkey:"42.jpg""#;
    assert_eq!(extract_nkey(html), Some("42".to_string()));
}

#[test]
fn test_extract_nkey_not_found() {
    let html = "some random html without nkey";
    assert!(extract_nkey(html).is_none());
}

#[test]
fn test_extract_nkey_in_context() {
    let html = r#"
        var player = new Clappr.Player({
            source:'livee.m3u8?a=token123',
            nkey:'999.jpg',
            parentId:'#player'
        });
    "#;
    assert_eq!(extract_nkey(html), Some("999".to_string()));
}

// ============================================================
// Test: extract_title
// ============================================================

#[test]
fn test_extract_title_basic() {
    let html = "<html><head><title>Trevi Fountain - Rome | SkylineWebcams</title></head></html>";
    let title = extract_title(html);
    assert_eq!(title, Some("Trevi Fountain - Rome".to_string()));
}

#[test]
fn test_extract_title_strips_skylinewebcams_suffix() {
    let html = "<title>Barcelona Beach - SkylineWebcams</title>";
    let title = extract_title(html);
    assert_eq!(title, Some("Barcelona Beach".to_string()));
}

#[test]
fn test_extract_title_no_suffix() {
    let html = "<title>Mount Vesuvius Live</title>";
    let title = extract_title(html);
    assert_eq!(title, Some("Mount Vesuvius Live".to_string()));
}

#[test]
fn test_extract_title_fallback_h1() {
    let html = r#"<html><body><h1 class="cam-title">Amalfi Coast</h1></body></html>"#;
    let title = extract_title(html);
    assert_eq!(title, Some("Amalfi Coast".to_string()));
}

#[test]
fn test_extract_title_not_found() {
    let html = "<html><body><p>no title here</p></body></html>";
    let title = extract_title(html);
    assert!(title.is_none());
}

// ============================================================
// Test: extract_country_from_path
// ============================================================

#[test]
fn test_extract_country_basic() {
    let country = extract_country_from_path("/en/webcam/italy/lazio/roma/trevi-fountain.html");
    assert_eq!(country, "Italy");
}

#[test]
fn test_extract_country_hyphenated() {
    let country = extract_country_from_path("/en/webcam/united-states/california/los-angeles/hollywood.html");
    assert_eq!(country, "United States");
}

#[test]
fn test_extract_country_short_path() {
    let country = extract_country_from_path("/en/webcam");
    assert_eq!(country, "Other");
}

// ============================================================
// Test: extract_tags_from_path
// ============================================================

#[test]
fn test_extract_tags_basic() {
    let tags = extract_tags_from_path("/en/webcam/italy/lazio/roma/trevi-fountain.html");
    assert_eq!(tags, vec!["Lazio".to_string(), "Roma".to_string()]);
}

#[test]
fn test_extract_tags_hyphenated() {
    let tags = extract_tags_from_path("/en/webcam/spain/islas-baleares/palma-de-mallorca/beach.html");
    assert_eq!(tags, vec!["Islas Baleares".to_string(), "Palma De Mallorca".to_string()]);
}

#[test]
fn test_extract_tags_short_path() {
    let tags = extract_tags_from_path("/en/webcam/italy");
    assert!(tags.is_empty());
}

// ============================================================
// Test: URL construction
// ============================================================

#[test]
fn test_build_hls_url() {
    let url = build_hls_url("abc123token");
    assert_eq!(
        url,
        "https://www.skylinewebcams.com/livee.m3u8?a=abc123token"
    );
}

#[test]
fn test_build_thumbnail_url() {
    let url = build_thumbnail_url("286");
    assert_eq!(url, "https://cdn.skylinewebcams.com/live286.jpg");
}

// ============================================================
// Test: capitalize_slug
// ============================================================

#[test]
fn test_capitalize_slug_single_word() {
    assert_eq!(capitalize_slug("italy"), "Italy");
}

#[test]
fn test_capitalize_slug_multi_word() {
    assert_eq!(capitalize_slug("united-states"), "United States");
}

#[test]
fn test_capitalize_slug_empty() {
    assert_eq!(capitalize_slug(""), "");
}

// ============================================================
// Test: strip_html_tags
// ============================================================

#[test]
fn test_strip_html_tags() {
    assert_eq!(strip_html_tags("<b>Hello</b> <i>World</i>"), "Hello World");
}

#[test]
fn test_strip_html_tags_nested() {
    assert_eq!(strip_html_tags("<div><span>Text</span></div>"), "Text");
}

#[test]
fn test_strip_html_tags_plain() {
    assert_eq!(strip_html_tags("No tags here"), "No tags here");
}

// ============================================================
// Test: full pipeline (token + nkey + title extraction)
// ============================================================

#[test]
fn test_full_cam_page_extraction() {
    let html = r#"
        <html>
        <head><title>Venice - St. Mark's Basin | SkylineWebcams</title></head>
        <body>
        <div id="player"></div>
        <script>
        var player = new Clappr.Player({
            source:'livee.m3u8?a=dG9rZW5fZm9yX3ZlbmljZQ==',
            poster:'/img/venice.jpg',
            nkey:'512.jpg',
            parentId:'#player',
            autoPlay:true,
            width:'100%',
            height:'100%'
        });
        </script>
        </body>
        </html>
    "#;

    let token = extract_hls_token(html).unwrap();
    assert_eq!(token, "dG9rZW5fZm9yX3ZlbmljZQ==");

    let nkey = extract_nkey(html).unwrap();
    assert_eq!(nkey, "512");

    let title = extract_title(html).unwrap();
    assert_eq!(title, "Venice - St. Mark's Basin");

    let stream_url = build_hls_url(&token);
    assert_eq!(
        stream_url,
        "https://www.skylinewebcams.com/livee.m3u8?a=dG9rZW5fZm9yX3ZlbmljZQ=="
    );

    let thumb_url = build_thumbnail_url(&nkey);
    assert_eq!(thumb_url, "https://cdn.skylinewebcams.com/live512.jpg");
}

// ============================================================
// Test: index page with multiple cam links extraction
// ============================================================

#[test]
fn test_index_page_extraction() {
    let html = r#"
        <html>
        <head><title>Top Live Cams | SkylineWebcams</title></head>
        <body>
        <div class="webcam-list-container">
            <div class="col">
                <a href="/en/webcam/italy/lazio/roma/trevi-fountain.html" class="thumbLink">
                    <img src="/img/thumb1.jpg" alt="Trevi Fountain" />
                </a>
            </div>
            <div class="col">
                <a href="/en/webcam/spain/cataluna/barcelona/la-rambla.html" class="thumbLink">
                    <img src="/img/thumb2.jpg" alt="La Rambla" />
                </a>
            </div>
            <div class="col">
                <a href="/en/webcam/greece/attica/athens/parthenon.html" class="thumbLink">
                    <img src="/img/thumb3.jpg" alt="Parthenon" />
                </a>
            </div>
            <div class="sidebar">
                <a href="/en/webcam/italy.html">All Italy Webcams</a>
                <a href="/en/top-live-cams.html">Top Cams</a>
            </div>
        </div>
        </body>
        </html>
    "#;

    let links = extract_cam_links(html);
    assert_eq!(links.len(), 3);

    // Verify country extraction for each
    assert_eq!(extract_country_from_path(&links[0]), "Italy");
    assert_eq!(extract_country_from_path(&links[1]), "Spain");
    assert_eq!(extract_country_from_path(&links[2]), "Greece");

    // Verify tags extraction
    let tags0 = extract_tags_from_path(&links[0]);
    assert_eq!(tags0, vec!["Lazio".to_string(), "Roma".to_string()]);

    let tags1 = extract_tags_from_path(&links[1]);
    assert_eq!(tags1, vec!["Cataluna".to_string(), "Barcelona".to_string()]);
}

// ============================================================
// Test: Config deserialization
// ============================================================

#[test]
fn test_config_defaults() {
    let config: Config = serde_json::from_str("{}").unwrap();
    assert_eq!(config.mode, "top");
    assert!(config.countries.is_empty());
    assert!(config.categories.is_empty());
}

#[test]
fn test_config_country_mode() {
    let json = r#"{"mode":"country","countries":["italy","spain"]}"#;
    let config: Config = serde_json::from_str(json).unwrap();
    assert_eq!(config.mode, "country");
    assert_eq!(config.countries, vec!["italy", "spain"]);
}

#[test]
fn test_config_category_mode() {
    let json = r#"{"mode":"category","categories":["beach-cams","city-cams"]}"#;
    let config: Config = serde_json::from_str(json).unwrap();
    assert_eq!(config.mode, "category");
    assert_eq!(config.categories, vec!["beach-cams", "city-cams"]);
}
