use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::{
    app,
    library::{cfg, logger},
};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Test {
        #[arg(short, long)]
        _case: String,
    },
    Run,
    Start,
    Restart,
    Shutdown,
}

pub async fn cmd() {
    let cli = Cli::parse();

    if let Some(config_path) = cli.config.as_deref() {
        cfg::init(&config_path.to_string_lossy().to_string());
    } else {
        println!("loading default config file!!!!");
        cfg::init(&"./fixtures/config.toml".to_string());
    }

    let (_guard1, _guard2, _guard3, _guard4) = logger::init(cfg::config());

    #[allow(clippy::single_match)]
    match &cli.command {
        Some(command) => match command {
            Commands::Test { _case } => todo!(),
            Commands::Run => {
                tracing::info!("Application started");
                app::serve().await;
                tracing::info!("Application stopped");
            }
            Commands::Start => todo!(),
            Commands::Restart => todo!(),
            Commands::Shutdown => todo!(),
        },
        None => {}
    }
}
