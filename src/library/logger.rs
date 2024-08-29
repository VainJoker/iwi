use std::{str::FromStr, sync::Arc};

use chrono::Local;
use tracing::{
    level_filters::LevelFilter, subscriber::set_global_default, Level,
};
use tracing_appender::non_blocking::{NonBlocking, WorkerGuard};
use tracing_subscriber::{
    filter, fmt,
    fmt::{format::Writer, time::FormatTime},
    layer::SubscriberExt,
    Layer, Registry,
};

use crate::library::cfg::Config;

struct LocalTimer;

impl FormatTime for LocalTimer {
    fn format_time(&self, w: &mut Writer<'_>) -> std::fmt::Result {
        write!(w, "{}", Local::now().format("%Y-%m-%d %H:%M:%S"))
    }
}

pub trait LogLayer<S: tracing::Subscriber>: Layer<S> + Send + Sync {}
impl<S: tracing::Subscriber, L: Layer<S> + Send + Sync> LogLayer<S> for L {}

struct RouterLayer<S> {
    mine_layer: Box<dyn LogLayer<S>>,
    database_layer: Box<dyn LogLayer<S>>,
    other_layer: Box<dyn LogLayer<S>>,
    error_layer: Box<dyn LogLayer<S>>,
    mine_target: String,
    database_target: String,
}

impl<S> Layer<S> for RouterLayer<S>
where
    S: tracing::Subscriber
        + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
{
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        match (event.metadata().level(), event.metadata().target()) {
            (level, _) if level <= &Level::ERROR => {
                self.error_layer.on_event(event, ctx)
            }
            (_, target) if target.starts_with(&self.mine_target) => {
                self.mine_layer.on_event(event, ctx);
            }
            (_, target) if target.starts_with(&self.database_target) => {
                self.database_layer.on_event(event, ctx);
            }
            _ => {
                self.other_layer.on_event(event, ctx);
            }
        }
    }
}

pub fn init(
    cfg: &Config,
) -> (WorkerGuard, WorkerGuard, WorkerGuard, WorkerGuard) {
    let (
        (mine_non_blocking, mine_guard),
        (database_non_blocking, database_guard),
        (other_non_blocking, other_guard),
        (error_non_blocking, error_guard),
        stdout,
    ) = {
        let stdout = cfg.app.env == "dev";

        let (mine_file, database_file, other_file, error_file) = (
            &cfg.log.mine_file,
            &cfg.log.database_file,
            &cfg.log.other_file,
            &cfg.log.error_file,
        );
        let setup_appender = |file| {
            tracing_appender::non_blocking(tracing_appender::rolling::daily(
                &cfg.log.path,
                file,
            ))
        };

        let mine_appender = setup_appender(mine_file);
        let database_appender = setup_appender(database_file);
        let other_appender = setup_appender(other_file);
        let error_appender = setup_appender(error_file);

        (
            mine_appender,
            database_appender,
            other_appender,
            error_appender,
            stdout,
        )
    };

    let setup_layer = |non_blocking: NonBlocking| {
        fmt::layer()
            .json()
            .with_timer(LocalTimer)
            .with_ansi(false)
            .with_writer(non_blocking)
            .flatten_event(true)
    };

    let mine_target = &cfg.log.mine_target;
    let database_target = &cfg.log.database_target;

    let router_file_layer = RouterLayer {
        mine_layer: Box::new(setup_layer(mine_non_blocking)),
        database_layer: Box::new(setup_layer(database_non_blocking)),
        other_layer: Box::new(setup_layer(other_non_blocking)),
        error_layer: Box::new(setup_layer(error_non_blocking)),
        mine_target: mine_target.clone(),
        database_target: database_target.clone(),
    };

    let (mine_level_formatting, other_level_formatting, level_file) = (
        LevelFilter::from_str(&cfg.log.mine_formatting_level)
            .unwrap_or(LevelFilter::INFO),
        LevelFilter::from_str(&cfg.log.other_formatting_level)
            .unwrap_or(LevelFilter::INFO),
        LevelFilter::from_str(&cfg.log.file_level).unwrap_or(LevelFilter::INFO),
    );

    if stdout {
        let mine_target = Arc::new(cfg.log.mine_target.clone());

        let mine_target_clone1 = Arc::clone(&mine_target);
        let mine_target_clone2 = Arc::clone(&mine_target);

        let mine_log = fmt::layer()
            .with_timer(LocalTimer)
            .pretty()
            .with_writer(std::io::stderr)
            .with_line_number(true)
            .with_filter(filter::filter_fn(move |metadata| {
                metadata.target().starts_with(&*mine_target_clone1)
            }));

        let other_log = fmt::layer()
            .with_timer(LocalTimer)
            .pretty()
            .with_writer(std::io::stderr)
            .with_line_number(true)
            .with_filter(filter::filter_fn(move |metadata| {
                !metadata.target().starts_with(&*mine_target_clone2)
            }));

        let registry = Registry::default()
            .with(router_file_layer.with_filter(level_file))
            .with(mine_log.with_filter(mine_level_formatting))
            .with(other_log.with_filter(other_level_formatting));

        set_global_default(registry).unwrap_or_else(|e| {
            panic!("💥 Failed to setting tracing subscriber: {e:?}");
        });
    } else {
        let registry =
            Registry::default().with(router_file_layer.with_filter(level_file));

        set_global_default(registry).unwrap_or_else(|e| {
            panic!("💥 Failed to setting tracing subscriber: {e:?}");
        });
    }

    (mine_guard, database_guard, other_guard, error_guard)
}
