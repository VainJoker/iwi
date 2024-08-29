use anyhow::anyhow;
use argon2::{
    password_hash::SaltString, Argon2, PasswordHash, PasswordHasher,
    PasswordVerifier,
};
use rand::{distributions::Alphanumeric, Rng};
use rand_core::OsRng;

use crate::library::error::{AppError, AppResult};

pub fn hash_password(password: &[u8]) -> AppResult<String> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password, &salt)
        .map_err(|e| {
            AppError::Anyhow(anyhow!("Error while hashing password: {}", e))
        })
        .map(|hash| hash.to_string())
}

pub fn verify_password(input: &str, hashed: &str) -> AppResult<bool> {
    Ok(match PasswordHash::new(input) {
        Ok(parsed_hash) => Argon2::default()
            .verify_password(hashed.as_bytes(), &parsed_hash)
            .map_or(false, |()| true),
        Err(_) => false,
    })
}

pub fn random_words(length: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}
