use std::{
    future::Future,
    pin::Pin,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering::SeqCst},
        Arc,
    },
    time::{Duration, Instant},
};

use deadpool_lapin::{
    lapin::{
        message::DeliveryResult,
        options::{
            BasicAckOptions, BasicConsumeOptions, BasicPublishOptions,
            QueueDeclareOptions,
        },
        types::FieldTable,
        BasicProperties, ConsumerDelegate,
    },
    Object, Runtime,
};

use super::error::AppResult;
use crate::library::{
    cfg,
    error::{InnerResult, MqerError},
};

pub type MQ = Object;
const TIMEOUT: u64 = 5;

#[derive(Clone)]
pub struct Mqer {
    pub pool: deadpool_lapin::Pool,
    pub running: Arc<AtomicBool>,
    pub count: Arc<AtomicUsize>,
}

#[derive(Clone)]
pub struct Subscriber {
    pub func: Arc<Box<dyn Fn(String) + Send + Sync>>,
    pub mqer: Arc<Mqer>,
}

impl Subscriber {
    pub fn new<F>(func: F, mqer: Arc<Mqer>) -> Self
    where
        F: Fn(String) + Send + Sync + 'static,
    {
        Self {
            func: Arc::new(Box::new(func)),
            mqer,
        }
    }
}

impl ConsumerDelegate for Subscriber {
    fn on_new_delivery(
        &self,
        delivery: DeliveryResult,
    ) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        let func_cloned = Arc::clone(&self.func);
        let mqer_cloned = Arc::clone(&self.mqer);
        Box::pin(async move {
            if let Ok(Some(delivery)) = delivery {
                mqer_cloned.increase_count();
                if !mqer_cloned.running.load(SeqCst) {
                    return;
                }

                let message = String::from_utf8_lossy(&delivery.data);
                (func_cloned)(message.to_string());
                if let Err(e) = delivery.ack(BasicAckOptions::default()).await {
                    tracing::error!("Failed to acknowledge message: {:?}", e);
                }
                mqer_cloned.decrease_count();
            } else {
                tracing::error!("Failed to consume queue message");
            };
        })
    }
}

impl Mqer {
    pub fn init() -> Self {
        let cfg = cfg::config();
        let mq_url = cfg.app.mq_url.clone();

        let deadpool = deadpool_lapin::Config {
            url: Some(mq_url),
            ..Default::default()
        };
        match deadpool.create_pool(Some(Runtime::Tokio1)) {
            Ok(pool) => {
                tracing::info!("🚀 Connection to the rabbit_mq is successful!");
                Self {
                    pool,
                    running: Arc::new(AtomicBool::new(true)),
                    count: Arc::new(AtomicUsize::new(0)),
                }
            }
            Err(err) => {
                panic!("💥 Failed to connect to the MQ: {err:?}");
            }
        }
    }

    pub async fn get_conn(&self) -> InnerResult<Option<MQ>> {
        // This block makes sure we release the lock before the async function.
        self.increase_count();

        if !self.running.load(SeqCst) {
            return Ok(None);
        }

        Ok(Some(self.pool.get().await.map_err(MqerError::PoolError)?))
    }

    fn decrease_count(&self) {
        self.count.fetch_sub(1, SeqCst);
    }

    fn increase_count(&self) {
        self.count.fetch_add(1, SeqCst);
    }

    pub fn graceful_shutdown(&self) -> AppResult<()> {
        self.running.store(false, SeqCst);

        let start = Instant::now();

        while self.count.load(SeqCst) > 0 {
            if start.elapsed() > Duration::from_secs(TIMEOUT) {
                tracing::warn!("Graceful shutdown timed out, exiting.");
                break;
            }
            std::thread::sleep(Duration::from_secs(1));
        }

        tracing::info!("MQ Stopped");
        Ok(())
    }

    pub async fn basic_send(
        &self,
        queue_name: &str,
        payload: &str,
    ) -> InnerResult<()> {
        let chan = self
            .get_conn()
            .await?
            .ok_or(anyhow::anyhow!("Channel is going to be closed"))?
            .create_channel()
            .await
            .map_err(MqerError::ExeError)?;

        let queue = chan
            .queue_declare(
                queue_name,
                QueueDeclareOptions::default(),
                FieldTable::default(),
            )
            .await
            .map_err(MqerError::ExeError)?;

        let payload = payload.as_bytes();

        chan.basic_publish(
            "",
            queue.name().as_str(),
            BasicPublishOptions::default(),
            payload,
            BasicProperties::default(),
        )
        .await
        .map_err(MqerError::ExeError)?
        .await
        .map_err(MqerError::ExeError)?;
        self.decrease_count();
        Ok(())
    }

    pub async fn basic_receive(
        &self,
        queue_name: &str,
        tag: &str,
        delegate: impl ConsumerDelegate + 'static,
    ) -> InnerResult<()> {
        let chan = self
            .get_conn()
            .await?
            .ok_or(anyhow::anyhow!("Channel is going to be closed"))?
            .create_channel()
            .await
            .map_err(MqerError::ExeError)?;

        let queue = chan
            .queue_declare(
                queue_name,
                QueueDeclareOptions::default(),
                FieldTable::default(),
            )
            .await
            .map_err(MqerError::ExeError)?;

        chan.basic_consume(
            queue.name().as_str(),
            tag,
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await
        .map_err(MqerError::ExeError)?
        .set_delegate(delegate);
        self.decrease_count();
        Ok(())
    }
}

// pub async fn topic_send(
//     &self,
//     exchange: &str,
//     queue_name: &str,
//     routing_key: &str,
//     payload: &str,
// ) -> InnerResult<()> {
//     let chan = self
//         .get_conn()
//         .await?
//         .create_channel()
//         .await
//         .map_err(MqerError::ExeError)?;
//
//     let () = chan
//         .exchange_declare(
//             exchange,
//             ExchangeKind::Topic,
//             ExchangeDeclareOptions::default(),
//             FieldTable::default(),
//         )
//         .await
//         .map_err(MqerError::ExeError)?;
//
//     let queue = chan
//         .queue_declare(
//             queue_name,
//             QueueDeclareOptions::default(),
//             FieldTable::default(),
//         )
//         .await
//         .map_err(MqerError::ExeError)?;
//
//     chan.queue_bind(
//         queue.name().as_str(),
//         exchange,
//         routing_key,
//         QueueBindOptions::default(),
//         FieldTable::default(),
//     )
//     .await
//     .map_err(MqerError::ExeError)?;
//
//     let payload = payload.as_bytes();
//
//     chan.basic_publish(
//         exchange,
//         routing_key,
//         BasicPublishOptions::default(),
//         payload,
//         BasicProperties::default(),
//     )
//     .await
//     .map_err(MqerError::ExeError)?
//     .await
//     .map_err(MqerError::ExeError)?;
//     Ok(())
// }
// pub async fn topic_receive<D: ConsumerDelegate + 'static>(
//     &self,
//     exchange: &str,
//     queue_name: &str,
//     routing_key: &str,
//     tag: &str,
//     delegate: D,
// ) -> InnerResult<()> {
//     let chan = self
//         .get_conn()
//         .await?
//         .create_channel()
//         .await
//         .map_err(MqerError::ExeError)?;
//
//     chan.exchange_declare(
//         exchange,
//         ExchangeKind::Topic,
//         ExchangeDeclareOptions::default(),
//         FieldTable::default(),
//     )
//     .await
//     .map_err(MqerError::ExeError)?;
//
//     let queue = chan
//         .queue_declare(
//             queue_name,
//             QueueDeclareOptions::default(),
//             FieldTable::default(),
//         )
//         .await
//         .map_err(MqerError::ExeError)?;
//
//     chan.queue_bind(
//         queue.name().as_str(),
//         exchange,
//         routing_key,
//         QueueBindOptions::default(),
//         FieldTable::default(),
//     )
//     .await
//     .map_err(MqerError::ExeError)?;
//
//     let consumer = chan
//         .basic_consume(
//             queue.name().as_str(),
//             tag,
//             BasicConsumeOptions::default(),
//             FieldTable::default(),
//         )
//         .await
//         .map_err(MqerError::ExeError)?;
//
//     consumer.set_delegate(delegate);
//
//     loop {
//         tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
//     }
// }

#[cfg(test)]
mod tests {
    // use deadpool_lapin::lapin::{
    //     message::DeliveryResult, options::BasicAckOptions,
    // };

    use std::sync::Arc;

    use crate::library::{cfg, mqer::Subscriber, Mqer};

    #[tokio::test]
    #[ignore]
    async fn test_basic_send() {
        cfg::init(&"./fixtures/config.toml".to_string());
        // let mqer = init("app.dev.queue", Some("app.dev.exchange"),
        // Some("app.dev.routine")).await;
        let mqer = Mqer::init();

        for i in 0..10 {
            let msg = format!("#{i} Testtest");
            eprintln!("{msg}");
            let confirm = mqer.basic_send("app.dev.queue", &msg).await;
            match confirm {
                Ok(()) => tracing::info!("[x] 消息已发送成功！{}", msg),
                Err(e) => tracing::error!("{:?}", e),
            };

            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    }

    #[tokio::test]
    #[ignore]
    async fn test_basic_receive() {
        cfg::init(&"./fixtures/config.toml".to_string());
        let mqer = Arc::new(Mqer::init());
        let func = |message: String| {
            eprintln!("{message}");
        };
        let delegate = Subscriber::new(func, mqer.clone());
        // tokio::spawn(async move {
        mqer.basic_receive("app.dev.queue", "app.dev.tag", delegate)
            .await
            .unwrap();
        // });
        // loop{}
    }

    // #[tokio::test]
    // #[ignore]
    // async fn test_topic_send() {
    //     cfg::init(&"../fixtures/config.toml".to_string());
    //     let mqer = Mqer::init();
    //     for i in 0..10 {
    //         let msg = format!("#{i} Testtest");
    //         let confirm = mqer
    //             .topic_send(
    //                 "app.dev.exchange",
    //                 "app.dev.queue",
    //                 "app.dev.routine",
    //                 &msg,
    //             )
    //             .await;
    //         match confirm {
    //             Ok(()) => tracing::info!("[x] 消息已发送成功！{}", msg),
    //             Err(e) => tracing::error!("{:?}", e),
    //         };
    //
    //         tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    //     }
    // }
    //
    // #[tokio::test]
    // #[ignore]
    // async fn test_topic_receive() {
    //     cfg::init(&"../fixtures/config.toml".to_string());
    //     let mqer = Mqer::init();
    //     mqer.topic_receive(
    //         "app.dev.queue",
    //         "app.dev.exchange",
    //         "app.dev.routine",
    //         "app.dev.tag",
    //         move |delivery: DeliveryResult| async move {
    //             tracing::debug!("aaa");
    //             let delivery = match delivery {
    //                 Ok(Some(delivery)) => delivery,
    //                 Ok(None) => {
    //                     tracing::error!("None ");
    //                     return;
    //                 }
    //                 Err(err) => {
    //                     tracing::error!(
    //                         "Failed to consume queue message {}",
    //                         err
    //                     );
    //                     return;
    //                 }
    //             };
    //
    //             let message = String::from_utf8_lossy(&delivery.data);
    //             tracing::info!("Received a message: {}", message);
    //
    //             delivery.ack(BasicAckOptions::default()).await.unwrap();
    //         },
    //     )
    //     .await
    //     .unwrap();
    // }
}
