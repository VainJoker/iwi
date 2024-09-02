use serde::{Deserialize, Serialize};
use sqlx::{types::chrono::NaiveDateTime, PgPool};

use crate::{
    library::error::InnerResult,
    models::types::{AccountStatus, Language},
};

#[allow(dead_code)]
#[derive(sqlx::FromRow, Debug, Serialize, Deserialize, Clone)]
#[sqlx(rename_all = "lowercase")]
pub struct Account {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub password: String,
    pub status: AccountStatus,

    pub language: Language,

    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Debug, Deserialize)]
pub struct LoginUserSchema {
    pub email_or_name: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct ResetPasswordSchema {
    pub uid: i64,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct RegisterSchema {
    pub name: String,
    pub email: String,
    pub password: String,
}

impl Account {
    pub async fn register_account(
        db: &PgPool,
        item: &RegisterSchema,
    ) -> InnerResult<Self> {
        let sql = r#"
            INSERT INTO bw_account (name, email, password) VALUES ($1, $2, $3)
            RETURNING id,name,email,password,language,status,
            created_at,updated_at,deleted_at
            "#;
        let map = sqlx::query_as(sql)
            .bind(&item.name)
            .bind(&item.email)
            .bind(&item.password);

        Ok(map.fetch_one(db).await?)
    }

    pub async fn check_user_exists_by_email(
        db: &PgPool,
        email: &str,
    ) -> InnerResult<Option<bool>> {
        let sql = r#"SELECT EXISTS(SELECT 1 FROM bw_account WHERE email = $1)"#;
        let map = sqlx::query_scalar(sql).bind(email);
        Ok(map.fetch_one(db).await?)
    }

    pub async fn check_user_exists_by_uid(
        db: &PgPool,
        uid: &i64,
    ) -> InnerResult<Option<bool>> {
        let sql = r#"SELECT EXISTS(SELECT 1 FROM bw_account WHERE id = $1)"#;
        let map = sqlx::query_scalar(sql).bind(uid);
        Ok(map.fetch_one(db).await?)
    }

    pub async fn fetch_user_by_email_or_name(
        db: &PgPool,
        email_or_name: &str,
    ) -> InnerResult<Vec<Self>> {
        let sql = r#"SELECT id,name,email,password,
            language,status,
            created_at,updated_at,deleted_at
            FROM bw_account WHERE name = $1 or email = $1"#;
        let map = sqlx::query_as(sql).bind(email_or_name);
        Ok(map.fetch_all(db).await?)
    }

    pub async fn fetch_user_by_uid(
        db: &PgPool,
        uid: i64,
    ) -> InnerResult<Option<Self>> {
        let sql = r#"SELECT id,name,email,password,
            language, status,
            created_at,updated_at,deleted_at
            FROM bw_account WHERE id = $1"#;

        let map = sqlx::query_as(sql).bind(uid);
        Ok(map.fetch_optional(db).await?)
    }

    pub async fn fetch_user_by_email(
        db: &PgPool,
        email: &str,
    ) -> InnerResult<Option<Self>> {
        let sql = r#"SELECT id,name,email,password,
            language, status,
            created_at,updated_at,deleted_at
            FROM bw_account WHERE email = $1"#;
        let map = sqlx::query_as(sql).bind(email);
        Ok(map.fetch_optional(db).await?)
    }

    pub async fn update_password_by_uid(
        db: &PgPool,
        item: &ResetPasswordSchema,
    ) -> InnerResult<u64> {
        let map =
            sqlx::query(r#"UPDATE bw_account set password = $1 WHERE id = $2"#)
                .bind(&item.password)
                .bind(item.uid);
        Ok(map.execute(db).await?.rows_affected())
    }

    pub async fn check_user_active_by_uid(
        db: &PgPool,
        uid: i64,
    ) -> InnerResult<Option<bool>> {
        let map = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM bw_account WHERE id = $1 and status = 'active')",
        ).bind(uid);
        Ok(map.fetch_one(db).await?)
    }
}

#[cfg(test)]
mod tests {
    use sqlx::PgPool;

    use super::*;

    const ACCOUNT_ID: i64 = 6192889942050345985;
    const EMAIL: &str = "test@test.com";
    const MY_EMAIL: &str = "vainjoker@tuta.io";
    const NAME: &str = "Test User";
    const PASSWORD: &str = "password";
    const NONEXISTENT_ACCOUNT_ID: i64 = 0;
    const NONEXISTENT_EMAIL: &str = "nonexistent@test.com";

    #[sqlx::test(fixtures(path = "../../fixtures", scripts("account")))]
    #[ignore]
    async fn test_register_account(pool: PgPool) -> sqlx::Result<()> {
        let item = RegisterSchema {
            name: NAME.to_string(),
            email: EMAIL.to_string(),
            password: PASSWORD.to_string(),
        };
        let account = Account::register_account(&pool, &item).await.unwrap();
        assert_eq!(account.email, EMAIL);
        assert_eq!(account.name, NAME);

        Ok(())
    }

    #[sqlx::test(fixtures(path = "../../fixtures", scripts("account")))]
    #[ignore]
    async fn test_fetch_user_by_email(pool: PgPool) -> sqlx::Result<()> {
        let account =
            Account::fetch_user_by_email(&pool, MY_EMAIL).await.unwrap();
        assert_eq!(account.unwrap().email, MY_EMAIL);

        Ok(())
    }

    #[sqlx::test(fixtures(path = "../../fixtures", scripts("account")))]
    #[ignore]
    async fn test_fetch_user_by_uid(pool: PgPool) -> sqlx::Result<()> {
        let account =
            Account::fetch_user_by_uid(&pool, ACCOUNT_ID).await.unwrap();
        assert_eq!(account.unwrap().id, ACCOUNT_ID);

        Ok(())
    }

    #[sqlx::test(fixtures(path = "../../fixtures", scripts("account")))]
    #[ignore]
    async fn test_check_user_exists_by_email(pool: PgPool) -> sqlx::Result<()> {
        let exists = Account::check_user_exists_by_email(&pool, MY_EMAIL)
            .await
            .unwrap();
        assert!(exists.unwrap());

        Ok(())
    }

    #[sqlx::test(fixtures(path = "../../fixtures", scripts("account")))]
    #[ignore]
    async fn test_check_user_exists_by_uid(pool: PgPool) -> sqlx::Result<()> {
        let exists = Account::check_user_exists_by_uid(&pool, &ACCOUNT_ID)
            .await
            .unwrap();
        assert!(exists.unwrap());

        Ok(())
    }

    #[sqlx::test(fixtures(path = "../../fixtures", scripts("account")))]
    #[ignore]
    async fn test_check_user_active_by_uid(pool: PgPool) -> sqlx::Result<()> {
        let is_active = Account::check_user_active_by_uid(&pool, ACCOUNT_ID)
            .await
            .unwrap();
        assert!(!is_active.unwrap()); // Assuming the account is active

        Ok(())
    }

    #[sqlx::test(fixtures(path = "../../fixtures", scripts("account")))]
    #[ignore]
    async fn test_update_password_by_uid(pool: PgPool) -> sqlx::Result<()> {
        let item = ResetPasswordSchema {
            uid: ACCOUNT_ID,
            password: "new_password".to_string(),
        };
        let rows_affected =
            Account::update_password_by_uid(&pool, &item).await.unwrap();
        assert_eq!(rows_affected, 1);

        Ok(())
    }

    #[sqlx::test(fixtures(path = "../../fixtures", scripts("account")))]
    #[ignore]
    async fn test_register_account_with_existing_email(
        pool: PgPool,
    ) -> sqlx::Result<()> {
        let item = RegisterSchema {
            name: "New User".to_string(),
            email: MY_EMAIL.to_string(),
            password: "password".to_string(),
        };
        let result = Account::register_account(&pool, &item).await;
        assert!(result.is_err());

        Ok(())
    }

    #[sqlx::test(fixtures(path = "../../fixtures", scripts("account")))]
    #[ignore]
    async fn test_fetch_user_by_nonexistent_email(
        pool: PgPool,
    ) -> sqlx::Result<()> {
        let account = Account::fetch_user_by_email(&pool, NONEXISTENT_EMAIL)
            .await
            .unwrap();
        assert!(account.is_none());

        Ok(())
    }

    #[sqlx::test(fixtures(path = "../../fixtures", scripts("account")))]
    #[ignore]
    async fn test_fetch_user_by_nonexistent_uid(
        pool: PgPool,
    ) -> sqlx::Result<()> {
        let account = Account::fetch_user_by_uid(&pool, NONEXISTENT_ACCOUNT_ID)
            .await
            .unwrap();
        assert!(account.is_none());

        Ok(())
    }

    #[sqlx::test(fixtures(path = "../../fixtures", scripts("account")))]
    #[ignore]
    async fn test_check_user_exists_by_nonexistent_email(
        pool: PgPool,
    ) -> sqlx::Result<()> {
        let exists =
            Account::check_user_exists_by_email(&pool, NONEXISTENT_EMAIL)
                .await
                .unwrap();
        assert!(!exists.unwrap());

        Ok(())
    }

    #[sqlx::test(fixtures(path = "../../fixtures", scripts("account")))]
    #[ignore]
    async fn test_check_user_exists_by_nonexistent_uid(
        pool: PgPool,
    ) -> sqlx::Result<()> {
        let exists =
            Account::check_user_exists_by_uid(&pool, &NONEXISTENT_ACCOUNT_ID)
                .await
                .unwrap();
        assert!(!exists.unwrap());

        Ok(())
    }

    #[sqlx::test(fixtures(path = "../../fixtures", scripts("account")))]
    #[ignore]
    async fn test_check_user_active_by_nonexistent_uid(
        pool: PgPool,
    ) -> sqlx::Result<()> {
        let is_active =
            Account::check_user_active_by_uid(&pool, NONEXISTENT_ACCOUNT_ID)
                .await
                .unwrap();
        assert!(!is_active.unwrap()); // Assuming the account is inactive

        Ok(())
    }

    #[sqlx::test(fixtures(path = "../../fixtures", scripts("account")))]
    #[ignore]
    async fn test_update_password_for_nonexistent_account(
        pool: PgPool,
    ) -> sqlx::Result<()> {
        let item = ResetPasswordSchema {
            uid: NONEXISTENT_ACCOUNT_ID,
            password: "new_password".to_string(),
        };
        let rows_affected =
            Account::update_password_by_uid(&pool, &item).await.unwrap();
        assert_eq!(rows_affected, 0);

        Ok(())
    }
}
