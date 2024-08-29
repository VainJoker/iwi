use crate::{
    app,
    library::{cfg, logger},
};

pub async fn web_serve() {
    cfg::init(&"./fixtures/config.toml".to_string());
    let (_guard1, _guard2, _guard3, _guard4) = logger::init(cfg::config());
    tracing::info!("Application started");
    app::serve().await;
    tracing::info!("Application stopped");
}
