use super::*;
use serde_json::json;

// ============================================================
// Systems parsing tests
// ============================================================

#[test]
fn test_parse_systems_basic() {
    let data = serde_json::to_vec(&json!({
        "systems": [
            {"shortName": "chi_cpd", "name": "Chicago Police"},
            {"shortName": "chi_cfd", "name": "Chicago Fire"},
            {"shortName": "sdrtrunk", "name": "San Diego"}
        ]
    }))
    .unwrap();

    let systems = parse_systems(&data);
    assert_eq!(systems.len(), 3);
    assert_eq!(systems[0], ("chi_cpd".to_string(), "Chicago Police".to_string()));
    assert_eq!(systems[1], ("chi_cfd".to_string(), "Chicago Fire".to_string()));
    assert_eq!(systems[2], ("sdrtrunk".to_string(), "San Diego".to_string()));
}

#[test]
fn test_parse_systems_empty() {
    let data = serde_json::to_vec(&json!({"systems": []})).unwrap();
    let systems = parse_systems(&data);
    assert!(systems.is_empty());
}

#[test]
fn test_parse_systems_invalid_json() {
    let data = b"not json at all";
    let systems = parse_systems(data);
    assert!(systems.is_empty());
}

#[test]
fn test_parse_systems_missing_key() {
    let data = serde_json::to_vec(&json!({"other": "stuff"})).unwrap();
    let systems = parse_systems(&data);
    assert!(systems.is_empty());
}

// ============================================================
// Calls parsing tests
// ============================================================

fn sample_calls_json() -> Vec<u8> {
    serde_json::to_vec(&json!({
        "calls": [
            {
                "_id": "abc123",
                "url": "https://media.openmhz.com/chi_cpd/2025/1/14/abc123.m4a",
                "talkgroupDescription": "Zone 1 - Dispatch",
                "talkgroupNum": 51001,
                "srcList": [{"src": 1234, "tag": "Unit 42"}],
                "time": 1705200000.0,
                "len": 12.5,
                "talkgroupGroup": "Police",
                "talkgroupTag": "Dispatch"
            },
            {
                "_id": "def456",
                "url": "https://media.openmhz.com/chi_cpd/2025/1/14/def456.m4a",
                "talkgroupDescription": "Fire Main",
                "talkgroupNum": 52001,
                "srcList": [],
                "time": 1705200030.0,
                "len": 8.0,
                "talkgroupGroup": "Fire",
                "talkgroupTag": "Tactical"
            },
            {
                "_id": "ghi789",
                "url": "https://media.openmhz.com/chi_cpd/2025/1/14/ghi789.m4a",
                "talkgroupDescription": "EMS North",
                "talkgroupNum": 53001,
                "srcList": [{"src": 5678}],
                "time": 1705200060.0,
                "len": 22.3,
                "talkgroupGroup": "EMS",
                "talkgroupTag": "Dispatch"
            }
        ]
    }))
    .unwrap()
}

#[test]
fn test_parse_calls_basic() {
    let calls = parse_calls(&sample_calls_json());
    assert_eq!(calls.len(), 3);
    assert_eq!(calls[0].id.as_deref(), Some("abc123"));
    assert_eq!(
        calls[0].url.as_deref(),
        Some("https://media.openmhz.com/chi_cpd/2025/1/14/abc123.m4a")
    );
    assert_eq!(calls[0].talkgroup_description.as_deref(), Some("Zone 1 - Dispatch"));
    assert_eq!(calls[0].talkgroup_num, Some(51001));
    assert_eq!(calls[0].len, Some(12.5));
}

#[test]
fn test_parse_calls_empty() {
    let data = serde_json::to_vec(&json!({"calls": []})).unwrap();
    let calls = parse_calls(&data);
    assert!(calls.is_empty());
}

#[test]
fn test_parse_calls_null_calls() {
    let data = serde_json::to_vec(&json!({"calls": null})).unwrap();
    let calls = parse_calls(&data);
    assert!(calls.is_empty());
}

#[test]
fn test_parse_calls_missing_key() {
    let data = serde_json::to_vec(&json!({"other": "data"})).unwrap();
    let calls = parse_calls(&data);
    assert!(calls.is_empty());
}

#[test]
fn test_parse_calls_invalid_json() {
    let calls = parse_calls(b"garbage data");
    assert!(calls.is_empty());
}

// ============================================================
// Call to stream conversion tests
// ============================================================

#[test]
fn test_call_to_stream_basic() {
    let call = Call {
        id: Some("abc123".to_string()),
        url: Some("https://media.openmhz.com/chi_cpd/abc123.m4a".to_string()),
        talkgroup_description: Some("Zone 1 - Dispatch".to_string()),
        talkgroup_num: Some(51001),
        src_list: Some(vec![SrcEntry { src: Some(1234), tag: Some("Unit 42".to_string()) }]),
        time: Some(1705200000.0),
        len: Some(12.5),
        talkgroup_group: Some("Police".to_string()),
        talkgroup_tag: Some("Dispatch".to_string()),
    };

    let stream = call_to_stream(&call, "Chicago Police").unwrap();
    assert_eq!(stream.id, "abc123");
    assert_eq!(stream.name, "Zone 1 - Dispatch (12s)");
    assert_eq!(stream.url, "https://media.openmhz.com/chi_cpd/abc123.m4a");
    assert_eq!(stream.group, "Chicago Police");
    assert_eq!(stream.logo, "");
    assert_eq!(stream.vod_type, "");
    assert_eq!(stream.tags, vec!["police", "dispatch"]);
}

#[test]
fn test_call_to_stream_no_url_returns_none() {
    let call = Call {
        id: Some("x".to_string()),
        url: None,
        talkgroup_description: Some("Test".to_string()),
        talkgroup_num: Some(1),
        src_list: None,
        time: Some(0.0),
        len: Some(5.0),
        talkgroup_group: None,
        talkgroup_tag: None,
    };
    assert!(call_to_stream(&call, "Test System").is_none());
}

#[test]
fn test_call_to_stream_empty_url_returns_none() {
    let call = Call {
        id: Some("x".to_string()),
        url: Some("".to_string()),
        talkgroup_description: Some("Test".to_string()),
        talkgroup_num: Some(1),
        src_list: None,
        time: Some(0.0),
        len: Some(5.0),
        talkgroup_group: None,
        talkgroup_tag: None,
    };
    assert!(call_to_stream(&call, "Test System").is_none());
}

#[test]
fn test_call_to_stream_missing_id_synthesizes() {
    let call = Call {
        id: None,
        url: Some("https://example.com/audio.m4a".to_string()),
        talkgroup_description: Some("Channel 5".to_string()),
        talkgroup_num: Some(5000),
        src_list: None,
        time: Some(1705200000.0),
        len: Some(3.0),
        talkgroup_group: None,
        talkgroup_tag: None,
    };

    let stream = call_to_stream(&call, "Test").unwrap();
    assert_eq!(stream.id, "5000-1705200000");
}

#[test]
fn test_call_to_stream_zero_duration() {
    let call = Call {
        id: Some("test1".to_string()),
        url: Some("https://example.com/audio.m4a".to_string()),
        talkgroup_description: Some("Channel 1".to_string()),
        talkgroup_num: Some(100),
        src_list: None,
        time: Some(0.0),
        len: Some(0.0),
        talkgroup_group: None,
        talkgroup_tag: None,
    };

    let stream = call_to_stream(&call, "Test").unwrap();
    assert_eq!(stream.name, "Channel 1");
}

#[test]
fn test_call_to_stream_missing_description() {
    let call = Call {
        id: Some("test2".to_string()),
        url: Some("https://example.com/audio.m4a".to_string()),
        talkgroup_description: None,
        talkgroup_num: Some(100),
        src_list: None,
        time: Some(0.0),
        len: Some(5.0),
        talkgroup_group: None,
        talkgroup_tag: None,
    };

    let stream = call_to_stream(&call, "Test").unwrap();
    assert_eq!(stream.name, "Unknown Talkgroup (5s)");
}

#[test]
fn test_call_to_stream_duplicate_tags_deduped() {
    let call = Call {
        id: Some("test3".to_string()),
        url: Some("https://example.com/audio.m4a".to_string()),
        talkgroup_description: Some("Test".to_string()),
        talkgroup_num: Some(100),
        src_list: None,
        time: Some(0.0),
        len: Some(1.0),
        talkgroup_group: Some("Fire".to_string()),
        talkgroup_tag: Some("Fire".to_string()),
    };

    let stream = call_to_stream(&call, "Test").unwrap();
    assert_eq!(stream.tags, vec!["fire"]);
}

// ============================================================
// calls_to_streams integration tests
// ============================================================

#[test]
fn test_calls_to_streams_basic() {
    let data = sample_calls_json();
    let streams = calls_to_streams(&data, "Chicago Police", 50);
    assert_eq!(streams.len(), 3);
    assert_eq!(streams[0].group, "Chicago Police");
    assert_eq!(streams[1].group, "Chicago Police");
    assert_eq!(streams[2].group, "Chicago Police");
}

#[test]
fn test_calls_to_streams_respects_limit() {
    let data = sample_calls_json();
    let streams = calls_to_streams(&data, "Test", 2);
    assert_eq!(streams.len(), 2);
}

#[test]
fn test_calls_to_streams_empty_response() {
    let data = serde_json::to_vec(&json!({"calls": []})).unwrap();
    let streams = calls_to_streams(&data, "Test", 50);
    assert!(streams.is_empty());
}

#[test]
fn test_calls_to_streams_skips_no_url() {
    let data = serde_json::to_vec(&json!({
        "calls": [
            {
                "_id": "good",
                "url": "https://example.com/good.m4a",
                "talkgroupDescription": "Good Call",
                "talkgroupNum": 1,
                "time": 1000.0,
                "len": 5.0
            },
            {
                "_id": "bad",
                "talkgroupDescription": "No URL Call",
                "talkgroupNum": 2,
                "time": 1001.0,
                "len": 3.0
            }
        ]
    }))
    .unwrap();

    let streams = calls_to_streams(&data, "Test", 50);
    assert_eq!(streams.len(), 1);
    assert_eq!(streams[0].id, "good");
}

// ============================================================
// System name lookup tests
// ============================================================

#[test]
fn test_build_system_names() {
    let data = serde_json::to_vec(&json!({
        "systems": [
            {"shortName": "chi_cpd", "name": "Chicago Police"},
            {"shortName": "sdrtrunk", "name": "San Diego Trunked"}
        ]
    }))
    .unwrap();

    let names = build_system_names(&data);
    assert_eq!(names.get("chi_cpd").unwrap(), "Chicago Police");
    assert_eq!(names.get("sdrtrunk").unwrap(), "San Diego Trunked");
    assert!(names.get("unknown").is_none());
}

#[test]
fn test_build_system_names_invalid_json() {
    let names = build_system_names(b"bad json");
    assert!(names.is_empty());
}

// ============================================================
// System selection tests
// ============================================================

#[test]
fn test_select_systems_defaults() {
    let config = serde_json::Map::new();
    let selected = select_systems(&config);
    assert_eq!(selected.len(), MAX_SYSTEMS);
    assert_eq!(selected[0], "chi_cpd");
}

#[test]
fn test_select_systems_from_string_array_config() {
    let mut config = serde_json::Map::new();
    config.insert(
        "systems".to_string(),
        json!(["kcmo", "daltrunk", "lasvegas"]),
    );

    let selected = select_systems(&config);
    assert_eq!(selected, vec!["kcmo", "daltrunk", "lasvegas"]);
}

#[test]
fn test_select_systems_from_object_array_config() {
    let mut config = serde_json::Map::new();
    config.insert(
        "systems".to_string(),
        json!([
            {"shortName": "kcmo"},
            {"shortName": "daltrunk"}
        ]),
    );

    let selected = select_systems(&config);
    assert_eq!(selected, vec!["kcmo", "daltrunk"]);
}

#[test]
fn test_select_systems_from_stringified_json() {
    let mut config = serde_json::Map::new();
    config.insert(
        "systems".to_string(),
        json!(r#"["chi_cpd","chi_cfd"]"#),
    );

    let selected = select_systems(&config);
    assert_eq!(selected, vec!["chi_cpd", "chi_cfd"]);
}

#[test]
fn test_select_systems_capped_at_max() {
    let mut config = serde_json::Map::new();
    let many: Vec<String> = (0..20).map(|i| format!("sys{}", i)).collect();
    config.insert("systems".to_string(), json!(many));

    let selected = select_systems(&config);
    assert_eq!(selected.len(), MAX_SYSTEMS);
}

#[test]
fn test_select_systems_empty_array_uses_defaults() {
    let mut config = serde_json::Map::new();
    config.insert("systems".to_string(), json!([]));

    let selected = select_systems(&config);
    assert_eq!(selected.len(), MAX_SYSTEMS);
    assert_eq!(selected[0], "chi_cpd");
}

// ============================================================
// Full pipeline integration test
// ============================================================

#[test]
fn test_full_pipeline_multi_system() {
    let sys1_data = serde_json::to_vec(&json!({
        "calls": [
            {
                "_id": "call1",
                "url": "https://media.openmhz.com/sys1/call1.m4a",
                "talkgroupDescription": "Dispatch",
                "talkgroupNum": 100,
                "time": 1705200000.0,
                "len": 10.0,
                "talkgroupGroup": "Police",
                "talkgroupTag": "Dispatch"
            },
            {
                "_id": "call2",
                "url": "https://media.openmhz.com/sys1/call2.m4a",
                "talkgroupDescription": "Tactical 1",
                "talkgroupNum": 101,
                "time": 1705200010.0,
                "len": 5.5,
                "talkgroupGroup": "Police",
                "talkgroupTag": "Tactical"
            }
        ]
    }))
    .unwrap();

    let sys2_data = serde_json::to_vec(&json!({
        "calls": [
            {
                "_id": "call3",
                "url": "https://media.openmhz.com/sys2/call3.m4a",
                "talkgroupDescription": "Fire Main",
                "talkgroupNum": 200,
                "time": 1705200020.0,
                "len": 15.0,
                "talkgroupGroup": "Fire",
                "talkgroupTag": "Dispatch"
            }
        ]
    }))
    .unwrap();

    let mut all_streams: Vec<Stream> = Vec::new();
    all_streams.extend(calls_to_streams(&sys1_data, "Chicago Police", 50));
    all_streams.extend(calls_to_streams(&sys2_data, "Chicago Fire", 50));

    assert_eq!(all_streams.len(), 3);
    assert_eq!(all_streams[0].group, "Chicago Police");
    assert_eq!(all_streams[0].id, "call1");
    assert_eq!(all_streams[1].group, "Chicago Police");
    assert_eq!(all_streams[2].group, "Chicago Fire");
    assert_eq!(all_streams[2].name, "Fire Main (15s)");
    assert_eq!(all_streams[2].tags, vec!["fire", "dispatch"]);
}

// ============================================================
// Stream format verification (matches space/demo format)
// ============================================================

#[test]
fn test_stream_serialization_format() {
    let stream = Stream {
        id: "abc123".to_string(),
        name: "Zone 1 - Dispatch (13s)".to_string(),
        url: "https://media.openmhz.com/chi_cpd/abc123.m4a".to_string(),
        group: "Chicago Police".to_string(),
        logo: String::new(),
        vod_type: String::new(),
        tags: vec!["police".to_string(), "dispatch".to_string()],
    };

    let json_val: serde_json::Value = serde_json::to_value(&stream).unwrap();
    assert!(json_val.get("id").is_some());
    assert!(json_val.get("name").is_some());
    assert!(json_val.get("url").is_some());
    assert!(json_val.get("group").is_some());
    assert!(json_val.get("logo").is_some());
    assert!(json_val.get("vod_type").is_some());
    assert!(json_val.get("tags").is_some());

    // Verify exact field values round-trip
    assert_eq!(json_val["id"].as_str().unwrap(), "abc123");
    assert_eq!(json_val["group"].as_str().unwrap(), "Chicago Police");
}

#[test]
fn test_refresh_response_serialization() {
    let resp = RefreshResponse {
        streams: vec![
            Stream {
                id: "s1".to_string(),
                name: "Stream 1".to_string(),
                url: "https://example.com/s1.m4a".to_string(),
                group: "System A".to_string(),
                logo: String::new(),
                vod_type: String::new(),
                tags: vec![],
            },
        ],
    };

    let json_val: serde_json::Value = serde_json::to_value(&resp).unwrap();
    assert!(json_val.get("streams").is_some());
    let streams_arr = json_val["streams"].as_array().unwrap();
    assert_eq!(streams_arr.len(), 1);
    assert_eq!(streams_arr[0]["id"].as_str().unwrap(), "s1");
}

// ============================================================
// Edge case: large number of calls
// ============================================================

#[test]
fn test_calls_to_streams_large_batch() {
    let mut calls = Vec::new();
    for i in 0..100 {
        calls.push(json!({
            "_id": format!("call{}", i),
            "url": format!("https://media.openmhz.com/test/call{}.m4a", i),
            "talkgroupDescription": format!("Talkgroup {}", i),
            "talkgroupNum": 1000 + i,
            "time": 1705200000.0 + (i as f64),
            "len": 5.0 + (i as f64) * 0.1,
            "talkgroupGroup": "Police",
            "talkgroupTag": "Dispatch"
        }));
    }

    let data = serde_json::to_vec(&json!({"calls": calls})).unwrap();

    // With limit 50, should only get 50
    let streams = calls_to_streams(&data, "Test System", 50);
    assert_eq!(streams.len(), 50);
    assert_eq!(streams[0].id, "call0");
    assert_eq!(streams[49].id, "call49");
}
