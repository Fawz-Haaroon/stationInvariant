mod server;
mod connection;
mod protocol;
mod ledger;
mod disk;
mod replay;
mod fanout;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    server::run("127.0.0.1:9000").await
}
