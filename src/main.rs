use app_server::cmd;

#[tokio::main]
async fn main() {
    cmd::web_serve().await;
}
