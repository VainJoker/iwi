use deadpool_redis::{
    redis::{AsyncCommands, FromRedisValue, ToRedisArgs},
    Connection, Pool, Runtime,
};

use crate::library::{
    cfg,
    error::{InnerResult, RedisorError},
};

pub struct Redisor {
    pub pool: Pool,
    pub prefix: &'static str,
}

pub struct Redis {
    pub connection: Connection,
    pub prefix: &'static str,
}

impl Redisor {
    pub fn init() -> Self {
        let cfg = cfg::config();
        let url = cfg.app.redis_url.clone();
        let prefix = &cfg.app.redis_prefix;
        let deadpool = deadpool_redis::Config::from_url(url);
        match deadpool.create_pool(Some(Runtime::Tokio1)) {
            Ok(pool) => {
                tracing::info!("🚀 Connection to the redis is successful!");
                Self { pool, prefix }
            }
            Err(err) => {
                panic!("💥 Failed to connect to the redis: {err:?}");
            }
        }
    }

    pub async fn get_redis(&self) -> InnerResult<Redis> {
        Ok(Redis {
            prefix: self.prefix,
            connection: self
                .pool
                .get()
                .await
                .map_err(RedisorError::PoolError)?,
        })
    }
}

impl Redis {
    pub fn key(&mut self, key: &str) -> String {
        format!("{}:{}", self.prefix, key)
    }

    pub async fn get<T: FromRedisValue + Send + Sync>(
        &mut self,
        key: &str,
    ) -> InnerResult<Option<T>> {
        let key = self.key(key);
        let result: Option<T> = self
            .connection
            .get(key)
            .await
            .map_err(RedisorError::ExeError)?;
        Ok(result)
    }

    pub async fn set<T: ToRedisArgs + Send + Sync>(
        &mut self,
        key: &str,
        value: T,
    ) -> InnerResult<()> {
        let key = self.key(key);
        self.connection
            .set::<_, _, ()>(key, value)
            .await
            .map_err(RedisorError::ExeError)?;
        Ok(())
    }

    pub async fn hkeys<T: FromRedisValue + Send + Sync>(
        &mut self,
        key: &str,
    ) -> InnerResult<Option<Vec<T>>> {
        let key = self.key(key);
        let result: Option<Vec<T>> = self
            .connection
            .hkeys(key)
            .await
            .map_err(RedisorError::ExeError)?;
        Ok(result)
    }

    pub async fn hset<T: ToRedisArgs + Send + Sync>(
        &mut self,
        key: &str,
        field: &str,
        value: T,
    ) -> InnerResult<()> {
        let key = self.key(key);
        self.connection
            .hset::<_, _, _, ()>(key, field, value)
            .await
            .map_err(RedisorError::ExeError)?;
        Ok(())
    }

    pub async fn del(&mut self, key: &str) -> InnerResult<()> {
        let key = self.key(key);
        self.connection
            .del::<_, ()>(key)
            .await
            .map_err(RedisorError::ExeError)?;
        Ok(())
    }

    pub async fn set_ex<T: ToRedisArgs + Send + Sync>(
        &mut self,
        key: &str,
        value: T,
        ttl: u64,
    ) -> InnerResult<()> {
        let key = self.key(key);
        self.connection
            .set_ex::<_, _, ()>(key, value, ttl)
            .await
            .map_err(RedisorError::ExeError)?;
        Ok(())
    }

    pub async fn expire(&mut self, key: &str, ttl: i64) -> InnerResult<()> {
        let key = self.key(key);
        self.connection
            .expire::<_, ()>(key, ttl)
            .await
            .map_err(RedisorError::ExeError)?;
        Ok(())
    }

    // pub async fn mget(
    //     &mut self,
    //     keys: &[&str],
    // ) -> InnerResult<Vec<Option<String>>> {
    //     // let key = self.key(key);
    //     let result: Vec<Option<String>> = self
    //         .connection
    //         .mget(keys)
    //         .await
    //         .map_err(RedisorError::ExeError)?;
    //     Ok(result)
    // }

    // pub async fn hgetalls(
    //     &mut self,
    //     keys: &[&str],
    // ) -> InnerResult<Vec<HashMap<String, String>>> {
    //     let mut pipe = redis::pipe();
    //     keys.into_iter().for_each(|key| {
    //         pipe.hgetall(key);
    //     });
    //     let result = pipe
    //         .query_async(&mut self.connection)
    //         .await
    //         .map_err(RedisorError::ExeError)?;
    //     Ok(result)
    // }

    // pub async fn hgets(
    //     &mut self,
    //     key: &str,
    //     fields: &[&str],
    // ) -> InnerResult<Vec<Option<String>>> {
    //     let result: Vec<Option<String>> = self
    //         .connection
    //         .hget(key, fields)
    //         .await
    //         .map_err(RedisorError::ExeError)?;
    //     Ok(result)
    // }
}

#[cfg(test)]
// ignore all
mod tests {
    use std::time;

    use super::*;

    #[tokio::test]
    #[ignore]
    async fn test_redisor_init() {
        cfg::init(&"./fixtures/config.toml".to_string());
        let redisor = Redisor::init();
        let mut redis = redisor.get_redis().await.unwrap();

        redis.set("ping", "pong").await.unwrap();
        assert_eq!(redis.get::<String>("ping").await.unwrap().unwrap(), "pong");
    }

    #[tokio::test]
    #[ignore]
    async fn test_redisor_del() {
        cfg::init(&"./fixtures/config.toml".to_string());
        let redisor = Redisor::init();
        let mut redis = redisor.get_redis().await.unwrap();

        redis.set("key2", "value").await.unwrap();
        assert_eq!(
            redis.get::<String>("key2").await.unwrap(),
            Some("value".to_string())
        );
        redis.del("key2").await.unwrap();
        assert_eq!(redis.get::<String>("key2").await.unwrap(), None);
        // redis.del("key2").await.unwrap();
    }

    #[tokio::test]
    #[ignore]
    async fn test_redisor_set_ex() {
        cfg::init(&"./fixtures/config.toml".to_string());
        let redisor = Redisor::init();
        let mut redis = redisor.get_redis().await.unwrap();
        redis.del("key3").await.unwrap();
        redis.set_ex("key3", "value", 10).await.unwrap();
        assert_eq!(
            redis.get::<String>("key3").await.unwrap(),
            Some("value".to_string())
        );
        tokio::time::sleep(time::Duration::from_millis(10000)).await;
        assert_eq!(redis.get::<String>("key3").await.unwrap(), None);
        redis.del("key3").await.unwrap();
    }

    #[tokio::test]
    #[ignore]
    async fn test_redisor_hset() {
        cfg::init(&"./fixtures/config.toml".to_string());
        let redisor = Redisor::init();
        let mut redis = redisor.get_redis().await.unwrap();
        redis.del("key4").await.unwrap();
        redis.hset("key4", "field1", "value1").await.unwrap();
        redis.hset("key4", "field2", "value2").await.unwrap();
        assert_eq!(
            redis.hkeys("key4").await.unwrap(),
            Some(vec!["field1".to_string(), "field2".to_string()])
        );
        redis.del("key4").await.unwrap();
    }

    #[tokio::test]
    #[ignore]
    async fn test_redisor_hkeys() {
        cfg::init(&"./fixtures/config.toml".to_string());
        let redisor = Redisor::init();
        let mut redis = redisor.get_redis().await.unwrap();
        redis.del("key5").await.unwrap();
        assert_eq!(redis.hkeys::<String>("key5").await.unwrap(), Some(vec![]));
        redis.hset("key5", "field1", "value1").await.unwrap();
        redis.hset("key5", "field2", "value2").await.unwrap();
        assert_eq!(
            redis.hkeys("key5").await.unwrap(),
            Some(vec!["field1".to_string(), "field2".to_string()])
        );
        redis.del("key5").await.unwrap();
    }

    #[tokio::test]
    #[ignore]
    async fn test_redisor_expire() {
        cfg::init(&"./fixtures/config.toml".to_string());
        let redisor = Redisor::init();
        let mut redis = redisor.get_redis().await.unwrap();
        redis.del("key6").await.unwrap();
        redis.set_ex("key6", "value", 10).await.unwrap();
        assert_eq!(
            redis.get::<String>("key6").await.unwrap(),
            Some("value".to_string())
        );
        tokio::time::sleep(time::Duration::from_millis(10000)).await;
        assert_eq!(redis.get::<String>("key6").await.unwrap(), None);
        redis.del("key6").await.unwrap();
    }

    // #[tokio::test]
    // async fn test_redisor_mget() {
    //     cfg::init(&"./fixtures/config.toml".to_string());
    //     let redisor = Redisor::init();
    //     let mut redis = redisor.get_redis().await.unwrap();
    //     redis.set("key7", "value1").await.unwrap();
    //     redis.set("key8", "value2").await.unwrap();
    //     assert_eq!(
    //         redis.mget(&["key7", "key8"].to_vec()).await.unwrap(),
    //         vec![Some("value1".to_string()), Some("value2".to_string())]
    //     );
    //     redis.del("key7").await.unwrap();
    //     redis.del("key8").await.unwrap();
    // }

    // #[tokio::test]
    // async fn test_redisor_hget() {
    //     cfg::init(&"./fixtures/config.toml".to_string());
    //     let redisor = Redisor::init();
    //     let mut redis = redisor.get_redis().await.unwrap();
    //     redis.del("key9").await.unwrap();
    //     redis.hset("key9", "field1", "value1").await.unwrap();
    //     redis.hset("key9", "field2", "value2").await.unwrap();
    //     assert_eq!(
    //         redis.hgets("key9", &["field1", "field2"]).await.unwrap(),
    //         vec![Some("value1".to_string()), Some("value2".to_string())]
    //     );
    //     redis.del("key9").await.unwrap();
    // }

    // #[tokio::test]
    // async fn test_redisor_hgetalls() {
    //     cfg::init(&"./fixtures/config.toml".to_string());
    //     let redisor = Redisor::init();
    //     let mut redis = redisor.get_redis().await.unwrap();
    //     redis.del("key10").await.unwrap();
    //     redis.hset("key10", "field1", "value1").await.unwrap();
    //     redis.hset("key10", "field2", "value2").await.unwrap();
    //     redis.del("key11").await.unwrap();
    //     redis.hset("key11", "field1", "value1").await.unwrap();
    //     redis.hset("key11", "field2", "value2").await.unwrap();
    //     eprintln!(
    //         "{:#?}",
    //         redis.hgetalls(&["key10", "key11", "key12"]).await.unwrap()
    //     );
    //     let mut hm1 = HashMap::new();
    //     hm1.insert("field1".to_string(), "value1".to_string());
    //     hm1.insert("field2".to_string(), "value2".to_string());
    //     let mut hm2 = HashMap::new();
    //     hm2.insert("field1".to_string(), "value1".to_string());
    //     hm2.insert("field2".to_string(), "value2".to_string());
    //     assert_eq!(
    //         redis.hgetalls(&["key10", "key11"]).await.unwrap(),
    //         vec![hm1, hm2]
    //     );
    //     redis.del("key10").await.unwrap();
    //     redis.del("key11").await.unwrap();
    // }
}
