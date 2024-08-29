use std::collections::HashMap;

use axum::{
    body::Body,
    extract::Request,
    http::header::CONTENT_TYPE,
    middleware::Next,
    response::{IntoResponse, Response},
};
use http_body_util::BodyExt;
use hyper::HeaderMap;

use crate::library::error::AppError;

pub async fn handle(request: Request, next: Next) -> Response {
    let enter_time = chrono::Local::now();
    let req_method = request.method().to_string();
    let req_uri = request.uri().to_string();
    let req_header = header_to_string(request.headers());

    let (response, body) = match drain_body(request, next).await {
        Err(err) => return err.into_response(),
        Ok(v) => v,
    };

    let duration = chrono::Local::now()
        .signed_duration_since(enter_time)
        .to_string();

    tracing::debug!(
        method = req_method,
        uri = req_uri,
        body = body,
        duration = duration,
        headers = req_header,
    );

    response
}

fn header_to_string(h: &HeaderMap) -> String {
    let mut map: HashMap<String, Vec<String>> = HashMap::new();

    for k in h.keys() {
        let mut vals: Vec<String> = Vec::new();

        for v in h.get_all(k) {
            if let Ok(s) = v.to_str() {
                vals.push(s.to_string());
            }
        }

        map.insert(k.to_string(), vals);
    }

    serde_json::to_string(&map).unwrap_or_else(|_| String::from("<none>"))
}

async fn drain_body(
    request: Request,
    next: Next,
) -> Result<(Response, Option<String>), AppError> {
    let ok = match request
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
    {
        Some(v) => {
            v.starts_with("application/json")
                || v.starts_with("application/x-www-form-urlencoded")
        }
        None => false,
    };

    if !ok {
        return Ok((next.run(request).await, None));
    }

    let (parts, body) = request.into_parts();

    // this wont work if the body is an long running stream
    let bytes = match body.collect().await {
        Ok(v) => v.to_bytes(),
        Err(err) => {
            tracing::error!("err parse request body : {err:?}");
            return Err(AppError::ErrSystem(String::new()));
        }
    };

    let body = std::str::from_utf8(&bytes)
        .map(std::string::ToString::to_string)
        .ok();

    let response = next
        .run(Request::from_parts(parts, Body::from(bytes)))
        .await;

    Ok((response, body))
}
