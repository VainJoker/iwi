use std::sync::{Arc, OnceLock};

use axum::{
    async_trait, extract::FromRequestParts, http::request::Parts,
    RequestPartsExt,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use jsonwebtoken::{
    decode, encode, DecodingKey, EncodingKey, Header, Validation,
};
use serde::{Deserialize, Serialize};

use crate::{
    app::bootstrap::AppState,
    library::{
        cfg,
        error::{AppError, AppError::AuthError, AppResult, AuthInnerError},
    },
    models::{account::Account, types::AccountStatus},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub uid: i64,
    pub email: String,
    pub status: AccountStatus,
    pub iat: usize,
    pub exp: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserInfo {
    pub uid: i64,
    pub email: String,
    pub status: AccountStatus,
}

#[derive(Debug, Serialize)]
pub struct TokenSchema {
    pub refresh_token: String,
    pub access_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

pub struct TokenSecretInfo<'a> {
    secret: &'a [u8],
    expiration: i64,
}

impl<'a> TokenSecretInfo<'a> {
    fn new(token_type: TokenType) -> Self {
        Self {
            secret: Self::get_secret(token_type),
            expiration: Self::get_secret_expiration(token_type),
        }
    }

    fn get_secret(token_type: TokenType) -> &'a [u8] {
        match token_type {
            TokenType::ACCESS => cfg::config().app.access_token.secret.as_ref(),
            TokenType::REFRESH => {
                cfg::config().app.refresh_token.secret.as_ref()
            }
        }
    }

    fn get_secret_expiration(token_type: TokenType) -> i64 {
        match token_type {
            TokenType::ACCESS => {
                cfg::config().app.access_token.secret_expiration.into()
            }
            TokenType::REFRESH => {
                cfg::config().app.refresh_token.secret_expiration.into()
            }
        }
    }
}

static ACCESS_INFO: OnceLock<Arc<TokenSecretInfo<'static>>> = OnceLock::new();
static REFRESH_INFO: OnceLock<Arc<TokenSecretInfo<'static>>> = OnceLock::new();

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TokenType {
    ACCESS,
    REFRESH,
}

pub trait TokenAuth {
    fn generate_token(&self, credential: &UserInfo) -> AppResult<String>;
    fn parse_token(&self, token: &str) -> AppResult<Claims>;
}

impl TokenAuth for TokenSecretInfo<'_> {
    fn generate_token(&self, credential: &UserInfo) -> AppResult<String> {
        let now = chrono::Utc::now();
        let duration = self.expiration;
        let claims = Claims {
            uid: credential.uid,
            email: credential.email.clone(),
            status: credential.status,
            exp: (now + chrono::Duration::seconds(duration)).timestamp()
                as usize,
            iat: now.timestamp() as usize,
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret),
        )
        .map_err(|_| AuthError(AuthInnerError::TokenCreation))?;

        Ok(token)
    }

    fn parse_token(&self, token: &str) -> AppResult<Claims> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret),
            &Validation::default(),
        )
        .map_err(|_| AuthError(AuthInnerError::InvalidToken))?;

        Ok(token_data.claims)
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> AppResult<Self> {
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| AuthError(AuthInnerError::InvalidToken))?;

        let claims =
            Self::parse_token(bearer.token(), TokenType::ACCESS, false)?;
        Ok(claims)
    }
}

impl Claims {
    pub fn generate_tokens(credential: &UserInfo) -> AppResult<TokenSchema> {
        let access_info = ACCESS_INFO
            .get_or_init(|| Arc::new(TokenSecretInfo::new(TokenType::ACCESS)));
        let refresh_info = REFRESH_INFO
            .get_or_init(|| Arc::new(TokenSecretInfo::new(TokenType::REFRESH)));

        let access_token = access_info.generate_token(credential)?;
        let refresh_token = refresh_info.generate_token(credential)?;

        Ok(TokenSchema {
            refresh_token,
            access_token,
        })
    }

    pub fn parse_token(
        token: &str,
        token_type: TokenType,
        flag: bool,
    ) -> AppResult<Self> {
        let info = match token_type {
            TokenType::ACCESS => ACCESS_INFO
                .get_or_init(|| Arc::new(TokenSecretInfo::new(token_type))),
            TokenType::REFRESH => REFRESH_INFO
                .get_or_init(|| Arc::new(TokenSecretInfo::new(token_type))),
        };
        let claims = info.parse_token(token)?;
        if (flag && claims.status == AccountStatus::Active)
            || (!flag && claims.status != AccountStatus::Suspend)
        {
            return Ok(claims);
        }
        Err(AuthError(AuthInnerError::InvalidToken))
    }

    pub async fn generate_tokens_for_user(
        user: &Account,
    ) -> AppResult<TokenSchema> {
        let user_info = UserInfo {
            uid: user.id,
            email: user.email.clone(),
            status: user.status,
        };
        let token = Claims::generate_tokens(&user_info)?;

        Ok(token)
    }

    pub async fn refresh_token(
        token: &str,
        state: Arc<AppState>,
    ) -> AppResult<TokenSchema> {
        let claims = Claims::parse_token(token, TokenType::REFRESH, false)?;

        let user = Account::fetch_user_by_uid(state.get_db(), claims.uid)
            .await?
            .ok_or(AuthError(AuthInnerError::WrongCredentials))?;

        Claims::generate_tokens_for_user(&user).await
    }
}
