use serde::{Deserialize, Serialize};

use crate::{
    app::service::jwt_service::TokenSchema,
    models::{
        account::Account,
        types::{AccountStatus, Language},
    },
};

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub tokens: TokenSchema,
    pub name: String,
    pub email: String,
    pub language: Language,
}

impl LoginResponse {
    pub fn new(tokens: TokenSchema, user: Account) -> Self {
        Self {
            tokens,
            name: user.name,
            email: user.email,
            language: user.language,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct TokenResponse {
    pub tokens: TokenSchema,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub email: String,
    pub language: Language,
    pub status: AccountStatus,
}

#[derive(Debug, Deserialize)]
pub struct RegisterUserRequest {
    pub name: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginUserRequest {
    pub email_or_name: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum CodeType {
    ActiveAccount,
    ResetPassword,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActiveAccountRequest {
    pub code: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResetPasswordRequest {
    pub code: String,
    pub password: String,
}
