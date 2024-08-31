use axum::{http::StatusCode, response::IntoResponse};

#[allow(clippy::unused_async)]
pub async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "Nothing to see here")
}
