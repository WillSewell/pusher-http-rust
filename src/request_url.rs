use std::time::SystemTime;
use url::form_urlencoded::Serializer;

use super::json_structures::*;
use super::signature::*;

const AUTH_VERSION: &'static str = "1.0";

pub fn timestamp() -> String {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        .to_string()
}

pub fn build_query(
    method: &str,
    path: &str,
    key: &str,
    secret: &str,
    timestamp: String,
    data: Option<&str>,
    query_parameters: Option<QueryParameters>,
) -> String {
    let body_md5: String;

    let mut query_pairs: Vec<(&str, &str)> = vec![
        ("auth_key", key),
        ("auth_timestamp", &timestamp),
        ("auth_version", AUTH_VERSION),
    ];

    if let Some(body) = data {
        body_md5 = create_body_md5(body);
        query_pairs.push(("body_md5", &body_md5));
    }

    let params = match query_parameters {
        Some(params) => params,
        None => Vec::new(),
    };
    for (k, v) in &params {
        query_pairs.push((k.as_str(), v.as_str()));
    }

    // query_pairs.sort_by(|&(a, _), &(b, _)| { a.cmp(b) });

    // Build the query string to sign by hand because we don't want it URL encoded
    let mut query_string_to_sign = String::new();
    let mut first = true;
    for &(k, v) in &query_pairs {
        if first {
            first = false;
        } else {
            query_string_to_sign.push_str("&");
        }
        query_string_to_sign.push_str(k);
        query_string_to_sign.push_str("=");
        query_string_to_sign.push_str(v);
    }

    let to_sign = format!("{}\n{}\n{}", method, path, query_string_to_sign);
    let auth_signature = sign(&to_sign, &secret);

    let query_buffer = String::new();
    let mut query_serializer = Serializer::new(query_buffer);

    for (k, v) in query_pairs {
        query_serializer.append_pair(k, v);
    }
    query_serializer.append_pair("auth_signature", auth_signature.as_str());

    return query_serializer.finish();
}

#[test]
fn test_trigger_request_url() {
    let expected = "auth_key=key&auth_timestamp=1353088179&auth_version=1.0&body_md5=ec365a775a4cd0599faeb73354201b6f&auth_signature=3695357e49aa04ae6f3cd76039dcefd82da079d0564bac566033d48ebae75459";
    let payload =
        "{\"name\":\"foo\",\"channels\":[\"project-3\"],\"data\":\"{\\\"some\\\":\\\"data\\\"}\"}";
    let query = build_query(
        "POST",
        "/apps/3/events",
        "key",
        "secret",
        "1353088179".to_string(),
        Some(payload),
        None,
    );
    assert_eq!(expected, query)
}

#[test]
fn test_get_channels_url() {
    let expected = "auth_key=key&auth_timestamp=1427034994&auth_version=1.0&filter_by_prefix=presence-&info=user_count&auth_signature=0ba82990cff5311f09d88d8c9317d1ceb1b2e085c01deb65618f4eaea1624d89";
    let query_parameters = Some(vec![
        ("filter_by_prefix".to_string(), "presence-".to_string()),
        ("info".to_string(), "user_count".to_string()),
    ]);
    let query = build_query(
        "GET",
        "/apps/102015/channels",
        "key",
        "secret",
        "1427034994".to_string(),
        None,
        query_parameters,
    );
    assert_eq!(expected, query)
}

#[test]
fn test_get_channels_url_with_one_additional_param() {
    let expected = "auth_key=key&auth_timestamp=1427036577&auth_version=1.0&filter_by_prefix=presence-&auth_signature=a27c87175390e1748e14fb6531769362ffb1a4fb437e9f353ff09e7fa314ce84";
    let query_parameters = Some(vec![(
        "filter_by_prefix".to_string(),
        "presence-".to_string(),
    )]);
    let query = build_query(
        "GET",
        "/apps/102015/channels",
        "key",
        "secret",
        "1427036577".to_string(),
        None,
        query_parameters,
    );
    assert_eq!(expected, query)
}

#[test]
fn test_get_channels_url_with_no_params() {
    let expected = "auth_key=key&auth_timestamp=1427036787&auth_version=1.0&auth_signature=805473a9346a00c6ddca6059286f7f6b4e4c45dea1ead355f115decba06bfa4d";
    let query = build_query(
        "GET",
        "/apps/102015/channels",
        "key",
        "secret",
        "1427036787".to_string(),
        None,
        None,
    );
    assert_eq!(expected, query)
}

#[test]
fn test_get_channel_url() {
    let expected = "auth_key=key&auth_timestamp=1427053326&auth_version=1.0&info=user_count%2Csubscription_count&auth_signature=c39bf2e1ef8a4cbfc8e283daa610862cf01fd250437476e1ff4100677ebd3dab";
    let query_parameters = Some(vec![(
        "info".to_string(),
        "user_count,subscription_count".to_string(),
    )]);
    let query = build_query("GET", "/apps/102015/channels/presence-session-d41a439c438a100756f5-4bf35003e819bb138249-ROpCFmgFhXY", "key", "secret", "1427053326".to_string(), None, query_parameters);
    assert_eq!(expected, query)
}

#[test]
fn test_get_users_url() {
    let expected = "auth_key=key&auth_timestamp=1427053326&auth_version=1.0&auth_signature=15f3d742965b5728ed2089c4fdae186a5684a8a17c9bf230ad5bd244bc8164f7";
    let query = build_query("GET", "/apps/102015/channels/presence-session-d41a439c438a100756f5-4bf35003e819bb138249-nYJLy67qh52/users", "key", "secret", "1427053326".to_string(), None, None);
    assert_eq!(expected, query)
}
