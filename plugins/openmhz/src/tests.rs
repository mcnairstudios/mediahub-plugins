use super::*;
use serde_json::json;

// ============================================================
// Systems parsing tests
// ============================================================

#[test]
fn test_parse_systems_basic() {
    let data = serde_json::to_vec(&json!({
        "success": true,
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
    let data = serde_json::to_vec(&json!({"success": true, "systems": []})).unwrap();
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
// Talkgroups parsing tests
// ============================================================

#[test]
fn test_parse_talkgroups_basic() {
    let data = serde_json::to_vec(&json!({
        "talkgroups": [
            {"num": 8, "description": "Zone 1 - Dispatch"},
            {"num": 51001, "description": "Fire Main"}
        ]
    }))
    .unwrap();

    let tg_map = parse_talkgroups(&data);
    assert_eq!(tg_map.len(), 2);
    assert_eq!(tg_map.get(&8).unwrap(), "Zone 1 - Dispatch");
    assert_eq!(tg_map.get(&51001).unwrap(), "Fire Main");
}

#[test]
fn test_parse_talkgroups_empty() {
    let data = serde_json::to_vec(&json!({"talkgroups": []})).unwrap();
    let tg_map = parse_talkgroups(&data);
    assert!(tg_map.is_empty());
}

#[test]
fn test_parse_talkgroups_invalid_json() {
    let tg_map = parse_talkgroups(b"garbage");
    assert!(tg_map.is_empty());
}

#[test]
fn test_parse_talkgroups_missing_description() {
    let data = serde_json::to_vec(&json!({
        "talkgroups": [
            {"num": 8},
            {"num": 9, "description": "Has Description"}
        ]
    }))
    .unwrap();

    let tg_map = parse_talkgroups(&data);
    assert_eq!(tg_map.len(), 1);
    assert_eq!(tg_map.get(&9).unwrap(), "Has Description");
}

// ============================================================
// Calls parsing tests
// ============================================================

fn sample_calls_json() -> Vec<u8> {
    serde_json::to_vec(&json!({
        "calls": [
            {
                "_id": "69fc2dd8d7432e855cce328e",
                "url": "https://media2.openmhz.com/media/chi_cpd/8/chi_cpd-8-1778134488.m4a",
                "talkgroupNum": 8,
                "srcList": [{"pos": 0, "src": "-1", "tag": "", "_id": "abc123"}],
                "time": "2026-05-07T06:14:48.000Z",
                "len": 1,
                "freq": 460200000,
                "emergency": false,
                "star": 0,
                "patches": []
            },
            {
                "_id": "69fc2dd7d7432e855cce3288",
                "url": "https://media2.openmhz.com/media/chi_cpd/52001/chi_cpd-52001-1778134400.m4a",
                "talkgroupNum": 52001,
                "srcList": [],
                "time": "2026-05-07T06:13:20.000Z",
                "len": 8,
                "freq": 460250000,
                "emergency": false
            },
            {
                "_id": "69fc2dd6d7432e855cce3282",
                "url": "https://media2.openmhz.com/media/chi_cpd/53001/chi_cpd-53001-1778134300.m4a",
                "talkgroupNum": 53001,
                "srcList": [{"pos": 0, "src": "5678", "tag": ""}],
                "time": "2026-05-07T06:11:40.000Z",
                "len": 22.3,
                "freq": 460300000,
                "emergency": true
            }
        ]
    }))
    .unwrap()
}

fn sample_tg_map() -> std::collections::HashMap<i64, String> {
    let mut m = std::collections::HashMap::new();
    m.insert(8, "Zone 1 - Dispatch".to_string());
    m.insert(52001, "Fire Main".to_string());
    m.insert(53001, "EMS North".to_string());
    m
}

#[test]
fn test_parse_calls_basic() {
    let calls = parse_calls(&sample_calls_json());
    assert_eq!(calls.len(), 3);
    assert_eq!(calls[0].id.as_deref(), Some("69fc2dd8d7432e855cce328e"));
    assert_eq!(
        calls[0].url.as_deref(),
        Some("https://media2.openmhz.com/media/chi_cpd/8/chi_cpd-8-1778134488.m4a")
    );
    assert_eq!(calls[0].talkgroup_num, Some(8));
    assert_eq!(calls[0].len, Some(1.0));
    assert_eq!(calls[0].time.as_deref(), Some("2026-05-07T06:14:48.000Z"));
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
        id: Some("69fc2dd8d7432e855cce328e".to_string()),
        url: Some("https://media2.openmhz.com/media/chi_cpd/8/chi_cpd-8-1778134488.m4a".to_string()),
        talkgroup_num: Some(8),
        src_list: Some(vec![SrcEntry { src: Some(json!("-1")), tag: Some("".to_string()) }]),
        time: Some("2026-05-07T06:14:48.000Z".to_string()),
        len: Some(12.5),
        freq: Some(460200000),
        emergency: Some(false),
    };

    let tg_map = sample_tg_map();
    let stream = call_to_stream(&call, "Chicago Police", &tg_map).unwrap();
    assert_eq!(stream.id, "69fc2dd8d7432e855cce328e");
    assert_eq!(stream.name, "Zone 1 - Dispatch (12s)");
    assert_eq!(stream.url, "https://media2.openmhz.com/media/chi_cpd/8/chi_cpd-8-1778134488.m4a");
    assert_eq!(stream.group, "Chicago Police");
    assert_eq!(stream.logo, "");
    assert_eq!(stream.vod_type, "");
    assert_eq!(stream.tags, vec!["tg8"]);
}

#[test]
fn test_call_to_stream_emergency_tag() {
    let call = Call {
        id: Some("emerg1".to_string()),
        url: Some("https://media2.openmhz.com/media/test/emerg1.m4a".to_string()),
        talkgroup_num: Some(53001),
        src_list: None,
        time: Some("2026-05-07T06:11:40.000Z".to_string()),
        len: Some(5.0),
        freq: None,
        emergency: Some(true),
    };

    let tg_map = sample_tg_map();
    let stream = call_to_stream(&call, "Test", &tg_map).unwrap();
    assert_eq!(stream.tags, vec!["tg53001", "emergency"]);
}

#[test]
fn test_call_to_stream_no_url_returns_none() {
    let call = Call {
        id: Some("x".to_string()),
        url: None,
        talkgroup_num: Some(1),
        src_list: None,
        time: Some("2026-05-07T00:00:00.000Z".to_string()),
        len: Some(5.0),
        freq: None,
        emergency: None,
    };
    let tg_map = std::collections::HashMap::new();
    assert!(call_to_stream(&call, "Test System", &tg_map).is_none());
}

#[test]
fn test_call_to_stream_empty_url_returns_none() {
    let call = Call {
        id: Some("x".to_string()),
        url: Some("".to_string()),
        talkgroup_num: Some(1),
        src_list: None,
        time: Some("2026-05-07T00:00:00.000Z".to_string()),
        len: Some(5.0),
        freq: None,
        emergency: None,
    };
    let tg_map = std::collections::HashMap::new();
    assert!(call_to_stream(&call, "Test System", &tg_map).is_none());
}

#[test]
fn test_call_to_stream_missing_id_synthesizes() {
    let call = Call {
        id: None,
        url: Some("https://example.com/audio.m4a".to_string()),
        talkgroup_num: Some(5000),
        src_list: None,
        time: Some("2026-05-07T00:00:00.000Z".to_string()),
        len: Some(3.0),
        freq: None,
        emergency: None,
    };

    let tg_map = std::collections::HashMap::new();
    let stream = call_to_stream(&call, "Test", &tg_map).unwrap();
    assert_eq!(stream.id, "5000-2026-05-07T00:00:00.000Z");
}

#[test]
fn test_call_to_stream_zero_duration() {
    let call = Call {
        id: Some("test1".to_string()),
        url: Some("https://example.com/audio.m4a".to_string()),
        talkgroup_num: Some(100),
        src_list: None,
        time: Some("2026-05-07T00:00:00.000Z".to_string()),
        len: Some(0.0),
        freq: None,
        emergency: None,
    };

    let mut tg_map = std::collections::HashMap::new();
    tg_map.insert(100, "Channel 1".to_string());
    let stream = call_to_stream(&call, "Test", &tg_map).unwrap();
    assert_eq!(stream.name, "Channel 1");
}

#[test]
fn test_call_to_stream_missing_talkgroup_description() {
    let call = Call {
        id: Some("test2".to_string()),
        url: Some("https://example.com/audio.m4a".to_string()),
        talkgroup_num: Some(100),
        src_list: None,
        time: Some("2026-05-07T00:00:00.000Z".to_string()),
        len: Some(5.0),
        freq: None,
        emergency: None,
    };

    // Empty tg_map means no description available
    let tg_map = std::collections::HashMap::new();
    let stream = call_to_stream(&call, "Test", &tg_map).unwrap();
    assert_eq!(stream.name, "Unknown Talkgroup (5s)");
}

// ============================================================
// calls_to_streams integration tests
// ============================================================

#[test]
fn test_calls_to_streams_basic() {
    let data = sample_calls_json();
    let tg_map = sample_tg_map();
    let streams = calls_to_streams(&data, "Chicago Police", 50, &tg_map);
    assert_eq!(streams.len(), 3);
    assert_eq!(streams[0].group, "Chicago Police");
    assert_eq!(streams[1].group, "Chicago Police");
    assert_eq!(streams[2].group, "Chicago Police");
}

#[test]
fn test_calls_to_streams_respects_limit() {
    let data = sample_calls_json();
    let tg_map = sample_tg_map();
    let streams = calls_to_streams(&data, "Test", 2, &tg_map);
    assert_eq!(streams.len(), 2);
}

#[test]
fn test_calls_to_streams_empty_response() {
    let data = serde_json::to_vec(&json!({"calls": []})).unwrap();
    let tg_map = std::collections::HashMap::new();
    let streams = calls_to_streams(&data, "Test", 50, &tg_map);
    assert!(streams.is_empty());
}

#[test]
fn test_calls_to_streams_skips_no_url() {
    let data = serde_json::to_vec(&json!({
        "calls": [
            {
                "_id": "good",
                "url": "https://media2.openmhz.com/media/test/good.m4a",
                "talkgroupNum": 1,
                "time": "2026-05-07T00:00:00.000Z",
                "len": 5
            },
            {
                "_id": "bad",
                "talkgroupNum": 2,
                "time": "2026-05-07T00:00:01.000Z",
                "len": 3
            }
        ]
    }))
    .unwrap();

    let tg_map = std::collections::HashMap::new();
    let streams = calls_to_streams(&data, "Test", 50, &tg_map);
    assert_eq!(streams.len(), 1);
    assert_eq!(streams[0].id, "good");
}

// ============================================================
// System name lookup tests
// ============================================================

#[test]
fn test_build_system_names() {
    let data = serde_json::to_vec(&json!({
        "success": true,
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
                "url": "https://media2.openmhz.com/media/sys1/100/call1.m4a",
                "talkgroupNum": 100,
                "time": "2026-05-07T00:00:00.000Z",
                "len": 10,
                "freq": 460200000,
                "emergency": false
            },
            {
                "_id": "call2",
                "url": "https://media2.openmhz.com/media/sys1/101/call2.m4a",
                "talkgroupNum": 101,
                "time": "2026-05-07T00:00:10.000Z",
                "len": 5.5,
                "freq": 460250000,
                "emergency": false
            }
        ]
    }))
    .unwrap();

    let sys2_data = serde_json::to_vec(&json!({
        "calls": [
            {
                "_id": "call3",
                "url": "https://media2.openmhz.com/media/sys2/200/call3.m4a",
                "talkgroupNum": 200,
                "time": "2026-05-07T00:00:20.000Z",
                "len": 15,
                "freq": 460300000,
                "emergency": false
            }
        ]
    }))
    .unwrap();

    let mut tg_map1 = std::collections::HashMap::new();
    tg_map1.insert(100, "Dispatch".to_string());
    tg_map1.insert(101, "Tactical 1".to_string());

    let mut tg_map2 = std::collections::HashMap::new();
    tg_map2.insert(200, "Fire Main".to_string());

    let mut all_streams: Vec<Stream> = Vec::new();
    all_streams.extend(calls_to_streams(&sys1_data, "Chicago Police", 50, &tg_map1));
    all_streams.extend(calls_to_streams(&sys2_data, "Chicago Fire", 50, &tg_map2));

    assert_eq!(all_streams.len(), 3);
    assert_eq!(all_streams[0].group, "Chicago Police");
    assert_eq!(all_streams[0].id, "call1");
    assert_eq!(all_streams[1].group, "Chicago Police");
    assert_eq!(all_streams[2].group, "Chicago Fire");
    assert_eq!(all_streams[2].name, "Fire Main (15s)");
    assert_eq!(all_streams[2].tags, vec!["tg200"]);
}

// ============================================================
// Stream format verification (matches space/demo format)
// ============================================================

#[test]
fn test_stream_serialization_format() {
    let stream = Stream {
        id: "abc123".to_string(),
        name: "Zone 1 - Dispatch (13s)".to_string(),
        url: "https://media2.openmhz.com/media/chi_cpd/8/abc123.m4a".to_string(),
        group: "Chicago Police".to_string(),
        logo: String::new(),
        vod_type: String::new(),
        tags: vec!["tg8".to_string()],
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
    let mut tg_map = std::collections::HashMap::new();
    for i in 0..100 {
        calls.push(json!({
            "_id": format!("call{}", i),
            "url": format!("https://media2.openmhz.com/media/test/{}/call{}.m4a", 1000 + i, i),
            "talkgroupNum": 1000 + i,
            "time": format!("2026-05-07T00:00:{:02}.000Z", i % 60),
            "len": 5.0 + (i as f64) * 0.1,
            "freq": 460200000 + i * 1000,
            "emergency": false
        }));
        tg_map.insert(1000 + i as i64, format!("Talkgroup {}", i));
    }

    let data = serde_json::to_vec(&json!({"calls": calls})).unwrap();

    // With limit 50, should only get 50
    let streams = calls_to_streams(&data, "Test System", 50, &tg_map);
    assert_eq!(streams.len(), 50);
    assert_eq!(streams[0].id, "call0");
    assert_eq!(streams[49].id, "call49");
}
