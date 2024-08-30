use iwi::cmd;

#[tokio::main]
async fn main() {
    cmd::cmd().await;
}
