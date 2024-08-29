use std::sync::Arc;

use axum::{extract::State, response::IntoResponse, Json};

use crate::{
    app::{
        bootstrap::{
            constants::{self, MQ_SEND_EMAIL_QUEUE},
            AppState,
        },
        entity::{
            account::{
                ActiveAccountRequest, LoginResponse, LoginUserRequest,
                RegisterUserRequest, ResetPasswordRequest, TokenResponse,
                UserResponse,
            },
            common::SuccessResponse,
        },
        service::jwt_service::{Claims, RefreshTokenRequest},
    },
    library::{
        crypto,
        error::{
            ApiInnerError,
            AppError::{ApiError, AuthError},
            AppResult, AuthInnerError,
        },
        mailor::Email,
    },
    models::{
        account::{Account, RegisterSchema, ResetPasswordSchema},
        types::AccountStatus,
    },
};

pub async fn register_user_handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<RegisterUserRequest>,
) -> AppResult<impl IntoResponse> {
    if Account::check_user_exists_by_email(state.get_db(), &body.email)
        .await?
        .unwrap_or(true)
    {
        return Err(AuthError(AuthInnerError::UserAlreadyExists));
    }

    let hashed_password = crypto::hash_password(body.password.as_bytes())?;
    let item = RegisterSchema {
        name: body.name,
        email: body.email,
        password: hashed_password,
    };

    let user = Account::register_account(state.get_db(), &item).await?;

    Ok(SuccessResponse {
        msg: "success",
        data: Some(Json(UserResponse {
            email: user.email,
            language: user.language,
            status: user.status,
        })),
    })
}

pub async fn login_user_handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<LoginUserRequest>,
) -> AppResult<impl IntoResponse> {
    let users = Account::fetch_user_by_email_or_name(
        state.get_db(),
        &body.email_or_name,
    )
    .await?;
    if users.is_empty() {
        return Err(AuthError(AuthInnerError::WrongCredentials));
    }
    for user in users {
        if crypto::verify_password(&user.password, &body.password)? {
            let tokens = Claims::generate_tokens_for_user(&user).await?;
            return Ok(SuccessResponse {
                msg: "Tokens generated successfully",
                data: Some(Json(LoginResponse::new(tokens, user))),
            });
        }
    }
    Err(AuthError(AuthInnerError::WrongCredentials))
}

pub async fn refresh_token_handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<RefreshTokenRequest>,
) -> AppResult<impl IntoResponse> {
    let tokens = Claims::refresh_token(&body.refresh_token, state).await?;
    Ok(SuccessResponse {
        msg: "Tokens refreshed successfully",
        data: Some(Json(TokenResponse { tokens })),
    })
}

pub async fn get_me_handler(
    State(state): State<Arc<AppState>>,
    claims: Claims,
) -> AppResult<impl IntoResponse> {
    if let Some(user) =
        Account::fetch_user_by_email(state.get_db(), &claims.email).await?
    {
        Ok(SuccessResponse {
            msg: "success",
            data: Some(Json(UserResponse {
                email: user.email,
                language: user.language,
                status: user.status,
            })),
        })
    } else {
        Err(AuthError(AuthInnerError::InvalidToken))
    }
}

pub async fn send_active_account_email_handler(
    State(state): State<Arc<AppState>>,
    claims: Claims,
) -> AppResult<impl IntoResponse> {
    let mut redis = state.get_redis().await?;
    let key = redis.key(&format!(
        "{}:{}",
        claims.uid,
        constants::REDIS_ACTIVE_ACCOUNT_KEY
    ));
    if redis.get::<String>(&key).await?.is_some() {
        return Err(ApiError(ApiInnerError::CodeIntervalRejection));
    }
    if claims.status != AccountStatus::Inactive {
        return Err(AuthError(AuthInnerError::UserAlreadyActivated));
    }
    let code = crypto::random_words(6);
    let body = format!("Active Code: {}", code);

    redis.set_ex(&key, &code, 60 * 5).await?;

    let email = Email::new(&claims.email, "Active your account", &body);
    let email_json = serde_json::to_string(&email).map_err(|e| {
        anyhow::anyhow!("Error occurred while sending email: {}", e)
    })?;
    state
        .get_mq()?
        .basic_send(MQ_SEND_EMAIL_QUEUE, &email_json)
        .await?;

    Ok(SuccessResponse {
        msg: "success",
        data: None::<()>,
    })
}

pub async fn send_reset_password_email_handler(
    State(state): State<Arc<AppState>>,
    claims: Claims,
) -> AppResult<impl IntoResponse> {
    let mut redis = state.get_redis().await?;
    let key = redis.key(&format!(
        "{}:{}",
        claims.uid,
        constants::REDIS_RESET_PASSWORD_KEY
    ));
    if redis.get::<String>(&key).await?.is_some() {
        return Err(ApiError(ApiInnerError::CodeIntervalRejection));
    }

    let code = crypto::random_words(6);
    let body = format!("ResetPassword Code: {}", code);

    redis.set_ex(&key, &code, 60).await?;

    let email = Email::new(&claims.email, "Reset Password", &body);
    let email_json = serde_json::to_string(&email).map_err(|e| {
        anyhow::anyhow!("Error occurred while sending email: {}", e)
    })?;
    state
        .get_mq()?
        .basic_send(MQ_SEND_EMAIL_QUEUE, &email_json)
        .await?;

    Ok(SuccessResponse {
        msg: "success",
        data: None::<()>,
    })
}

pub async fn verify_active_account_code_handler(
    State(state): State<Arc<AppState>>,
    claims: Claims,
    Json(body): Json<ActiveAccountRequest>,
) -> AppResult<impl IntoResponse> {
    let mut redis = state.get_redis().await?;
    if claims.status != AccountStatus::Inactive {
        return Err(AuthError(AuthInnerError::UserAlreadyActivated));
    }
    let key = redis.key(&format!(
        "{}:{}",
        claims.uid,
        constants::REDIS_ACTIVE_ACCOUNT_KEY
    ));

    if let Some(stored) = redis.get::<String>(&key).await? {
        if stored == body.code {
            redis.del(&key).await?;
        } else {
            return Err(AuthError(AuthInnerError::WrongCode));
        }
    }

    let user = Account::fetch_user_by_uid(state.get_db(), claims.uid)
        .await?
        .ok_or(AuthError(AuthInnerError::WrongCredentials))?;

    let tokens = Claims::generate_tokens_for_user(&user).await?;

    redis.del(&key).await?;

    Ok(SuccessResponse {
        msg: "success",
        data: Some(Json(TokenResponse { tokens })),
    })
}

pub async fn change_password_handler(
    State(state): State<Arc<AppState>>,
    claims: Claims,
    Json(body): Json<ResetPasswordRequest>,
) -> AppResult<impl IntoResponse> {
    let mut redis = state.get_redis().await?;
    let key = redis.key(&format!(
        "{}:{}",
        claims.uid,
        constants::REDIS_RESET_PASSWORD_KEY
    ));

    if let Some(stored) = redis.get::<String>(&key).await? {
        if stored == body.code {
            let item = ResetPasswordSchema {
                uid: claims.uid,
                password: crypto::hash_password(body.password.as_bytes())?,
            };
            Account::update_password_by_uid(state.get_db(), &item).await?;
            redis.del(&key).await?;
        } else {
            return Err(AuthError(AuthInnerError::WrongCode));
        }
    }

    Ok(SuccessResponse {
        msg: "success",
        data: None::<()>,
    })
}
