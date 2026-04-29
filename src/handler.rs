use crate::channels::Channel;
use crate::config::EndpointConfig;
use crate::notification::Notification;
use http::{Request, StatusCode};
use serde_json::Value;
use std::collections::HashMap;

pub async fn handle_request(
    request: &Request<bytes::Bytes>,
    endpoints: &[EndpointConfig],
    channels: &[Box<dyn Channel>],
) -> (StatusCode, String) {
    let method = request.method().as_str();
    let (path, query) = parse_path_query(request.uri().path_and_query().map(|pq| pq.as_str()).unwrap_or(""));
    let auth_header = request.headers().get("authorization").and_then(|v| v.to_str().ok()).map(|s| s.to_string());
    let body = if request.body().is_empty() {
        None
    } else {
        Some(String::from_utf8_lossy(request.body()).into_owned())
    };

    // Route matching
    let endpoint = match endpoints.iter().find(|ep| ep.path == path) {
        Some(ep) => ep,
        None => {
            log::warn!("{method} {path}: endpoint not found");
            return (StatusCode::NOT_FOUND, json_response("not_found", "endpoint not found"));
        }
    };

    // Token auth
    if let Some(ref expected) = endpoint.token {
        let provided = query
            .get("token")
            .cloned()
            .or_else(|| {
                auth_header.as_ref().and_then(|h| {
                    h.strip_prefix("Bearer ").map(|s| s.to_string())
                })
            });

        match provided {
            Some(ref t) if t == expected => {}
            _ => {
                log::warn!("{method} {path}: unauthorized");
                return (StatusCode::UNAUTHORIZED, json_response("unauthorized", "invalid or missing token"));
            }
        }
    }

    // Parse notification fields
    let body_json: Option<Value> = body.as_ref().and_then(|b| serde_json::from_str(b).ok());

    let title = extract_field(&query, body_json.as_ref(), "title", "title_path");
    let message = extract_field(&query, body_json.as_ref(), "message", "message_path");

    match (title, message) {
        (Some(title), Some(message)) => {
            let to = query.get("to").cloned();
            let from = query.get("from").cloned();
            let title_prefix = query.get("title_prefix").cloned();

            let title = match title_prefix {
                Some(prefix) => format!("[{prefix}] {title}"),
                None => title,
            };

            let level = query
                .get("level")
                .cloned()
                .or_else(|| {
                    body_json.as_ref().and_then(|json| json.get("level").and_then(|v| v.as_str().map(|s| s.to_string())))
                })
                .unwrap_or_else(|| "info".to_string());

            let notification = Notification { title, message, to, from, level };

            // Send to all channels bound to this endpoint
            let channel_map: HashMap<&str, &Box<dyn Channel>> =
                channels.iter().map(|ch| (ch.name(), ch)).collect();

            let mut errors = Vec::new();
            for ch_name in &endpoint.channels {
                if let Some(ch) = channel_map.get(ch_name.as_str()) {
                    if let Err(e) = ch.send(&notification).await {
                        errors.push(format!("{}: {}", ch_name, e));
                    }
                }
            }

            if errors.is_empty() {
                log::info!("{method} {path}: sent to [{}]", endpoint.channels.join(", "));
                (StatusCode::OK, json_response("ok", "notification sent"))
            } else {
                log::error!("{method} {path}: partial failure — {}", errors.join("; "));
                (StatusCode::INTERNAL_SERVER_ERROR, json_response("partial_failure", &errors.join("; ")))
            }
        }
        _ => {
            log::warn!("{method} {path}: missing required fields");
            (StatusCode::BAD_REQUEST, json_response("bad_request", "missing required fields: title and message"))
        }
    }
}

fn parse_path_query(raw: &str) -> (String, HashMap<String, String>) {
    let (path, query_str) = match raw.split_once('?') {
        Some((p, q)) => (p.to_string(), q),
        None => (raw.to_string(), ""),
    };
    let query = urlencoded_params(query_str);
    (path, query)
}

fn urlencoded_params(s: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for pair in s.split('&') {
        if pair.is_empty() {
            continue;
        }
        if let Some((k, v)) = pair.split_once('=') {
            map.insert(url_decode(k), url_decode(v));
        } else {
            map.insert(url_decode(pair), String::new());
        }
    }
    map
}

fn url_decode(s: &str) -> String {
    let mut bytes = Vec::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '%' {
            let hex: String = chars.by_ref().take(2).collect();
            if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                bytes.push(byte);
            } else {
                bytes.push(b'%');
                bytes.extend(hex.as_bytes());
            }
        } else if c == '+' {
            bytes.push(b' ');
        } else {
            bytes.extend(c.encode_utf8(&mut [0u8; 4]).as_bytes());
        }
    }
    String::from_utf8_lossy(&bytes).into_owned()
}

fn extract_field(
    query: &HashMap<String, String>,
    body_json: Option<&Value>,
    direct_key: &str,
    path_key: &str,
) -> Option<String> {
    if let Some(path) = query.get(path_key) {
        return body_json.and_then(|json| json_path_extract(json, path));
    }
    if let Some(val) = query.get(direct_key) {
        return Some(val.clone());
    }
    body_json
        .and_then(|json| json.get(direct_key))
        .and_then(|v| v.as_str().map(|s| s.to_string()))
}

fn json_path_extract(json: &Value, path: &str) -> Option<String> {
    let path = path.strip_prefix('.')?;
    let mut current = json;
    for key in path.split('.') {
        current = current.get(key)?;
    }
    current.as_str().map(|s| s.to_string())
}

fn json_response(status: &str, message: &str) -> String {
    format!(r#"{{"status":"{status}","message":"{message}"}}"#)
}
