use sqlx::{postgres::PgPoolOptions, PgPool};

use crate::library::cfg;

pub type DB = PgPool;

pub struct Dber {
    pub pool: PgPool,
}

impl Dber {
    pub async fn init() -> Self {
        let cfg = cfg::config();
        let database_url = &cfg.app.db_url;
        match PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await
        {
            Ok(pool) => {
                tracing::info!("🚀 Connection to the database is successful!");
                Self { pool }
            }
            Err(err) => {
                panic!("💥 Failed to connect to the database: {err:?}");
            }
        }
    }
}
