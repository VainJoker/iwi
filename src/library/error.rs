use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

pub type InnerResult<T> = Result<T, AppInnerError>;

#[derive(Error, Debug)]
pub enum AppInnerError {
    // TODO: Better Not Show
    #[error("Database error: `{0}`")]
    DataBaseError(#[from] sqlx::Error),
    #[error(transparent)]
    RedisError(#[from] RedisorError),
    #[error(transparent)]
    MQError(#[from] MqerError),
    #[error("Json error: `{0}`")]
    JsonError(#[from] serde_json::Error),
    #[error("Email error: `{0}`")]
    EmailError(#[from] lettre::transport::smtp::Error),
    #[error("Internal server error")]
    Unknown(String),
    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
}

#[derive(Error, Debug)]
pub enum RedisorError {
    #[error("Redis connection error: `{0}`")]
    PoolError(#[from] deadpool_redis::PoolError),
    #[error("Redis execution error: `{0}`")]
    ExeError(#[from] deadpool_redis::redis::RedisError),
}

#[derive(Error, Debug)]
pub enum MqerError {
    #[error("Mq connection error: `{0}`")]
    PoolError(#[from] deadpool_lapin::PoolError),
    #[error("Mq execution error: `{0}`")]
    ExeError(#[from] deadpool_lapin::lapin::Error),
}

#[derive(Error, Debug)]
pub enum ApiInnerError {
    #[error(transparent)]
    ValidationError(#[from] validator::ValidationErrors),

    #[error(transparent)]
    AxumFormRejection(#[from] axum::extract::rejection::FormRejection),

    #[error("Verification Code Interval Not Satisfied")]
    CodeIntervalRejection,
}

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Unknown error `{0}`")]
    Unknown(String),

    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),

    #[error("System error `{0}`")]
    ErrSystem(String),

    #[error(transparent)]
    InnerError(#[from] AppInnerError),

    #[error(transparent)]
    AuthError(#[from] AuthInnerError),

    #[error(transparent)]
    ApiError(#[from] ApiInnerError),
}

#[derive(Error, Debug)]
pub enum AuthInnerError {
    #[error("UserAlreadyExists")]
    UserAlreadyExists,
    #[error("WrongCredentials")]
    WrongCredentials,
    #[error("MissingCredentials")]
    MissingCredentials,
    #[error("TokenCreation")]
    TokenCreation,
    #[error("InvalidToken")]
    InvalidToken,
    #[error("WrongCode")]
    WrongCode,
    #[error("AccountSuspended")]
    AccountSuspended,
    #[error("InvalidTokenType")]
    InvalidTokenType,
    #[error("UserAlreadyActivated")]
    UserAlreadyActivated,
}

impl AppError {
    pub fn select_status_code(app_error: &Self) -> (StatusCode, u32) {
        match app_error {
            Self::AuthError(e) => match e {
                AuthInnerError::WrongCredentials => {
                    (StatusCode::UNAUTHORIZED, 10001)
                }
                AuthInnerError::TokenCreation => (StatusCode::FORBIDDEN, 10002),
                AuthInnerError::InvalidToken => {
                    (StatusCode::UNAUTHORIZED, 10003)
                }
                AuthInnerError::UserAlreadyExists => {
                    (StatusCode::CONFLICT, 10004)
                }
                AuthInnerError::MissingCredentials => {
                    (StatusCode::UNAUTHORIZED, 10005)
                }
                AuthInnerError::WrongCode => (StatusCode::UNAUTHORIZED, 10006),
                AuthInnerError::AccountSuspended => {
                    (StatusCode::UNAUTHORIZED, 10007)
                }
                AuthInnerError::InvalidTokenType => {
                    (StatusCode::UNAUTHORIZED, 10008)
                }
                AuthInnerError::UserAlreadyActivated => {
                    (StatusCode::CONFLICT, 10009)
                }
            },
            Self::ApiError(e) => match e {
                ApiInnerError::ValidationError(_) => {
                    (StatusCode::UNPROCESSABLE_ENTITY, 20001)
                }
                ApiInnerError::AxumFormRejection(_) => {
                    (StatusCode::UNPROCESSABLE_ENTITY, 20001)
                }
                ApiInnerError::CodeIntervalRejection => (StatusCode::OK, 30001),
            },
            _ => (StatusCode::BAD_REQUEST, 99999),
        }
    }
}

pub type AppResult<T> = Result<T, AppError>;

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code) = Self::select_status_code(&self);
        let body = axum::Json(serde_json::json!({
            "code": code,
            "msg": format!("{self}")
        }));
        (status, body).into_response()
    }
}
