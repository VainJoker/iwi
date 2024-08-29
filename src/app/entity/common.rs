use axum::{
    response::{IntoResponse, Response},
    Json,
};
use hyper::StatusCode;
use serde::Serialize;

use crate::library::error::AppError;

pub struct AppResponse<'a, T: IntoResponse> {
    pub code: u16,
    pub msg: &'a str,
    pub data: Option<T>,
    pub err: Option<AppError>,
}

pub trait IntoAppResponse
where
    Self: IntoResponse,
{
}

pub struct SuccessResponse<'a, T: IntoResponse> {
    pub msg: &'a str,
    pub data: Option<T>,
}

impl<'a, T: IntoResponse> From<SuccessResponse<'a, T>> for AppResponse<'a, T> {
    fn from(val: SuccessResponse<'a, T>) -> Self {
        Self {
            code: 0,
            msg: val.msg,
            data: val.data,
            err: None,
        }
    }
}

impl<'a, U: Serialize> IntoResponse for AppResponse<'a, Json<U>> {
    fn into_response(self) -> Response {
        let (status, code) = if let Some(app_error) = self.err {
            AppError::select_status_code(&app_error)
        } else {
            (StatusCode::OK, 0)
        };
        let body = Json(serde_json::json!({
            "code": code,
            "msg": self.msg,
            "data": self.data.map(|d| d.0)
        }));
        (status, body).into_response()
    }
}

impl<'a, U: Serialize> IntoResponse for SuccessResponse<'a, Json<U>> {
    fn into_response(self) -> Response {
        let status = StatusCode::OK;
        let body = Json(serde_json::json!({
            "code": 0,
            "msg": self.msg,
            "data": self.data.map(|d| d.0)
        }));
        (status, body).into_response()
    }
}

impl<'a> IntoResponse for SuccessResponse<'a, ()> {
    fn into_response(self) -> Response {
        let status = StatusCode::OK;
        let body = Json(serde_json::json!({
            "code": 0,
            "msg": self.msg,
            "data": None::<()>
        }));
        (status, body).into_response()
    }
}
