mod server;
mod connection;
mod fanout;

pub use server::Server;

use std::net::TcpListener;

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:9000")?;
    let mut server = Server::new("data/wal.log")?;

    for stream in listener.incoming() {
        let stream = stream?;
        connection::handle(stream, &mut server)?;
    }

    Ok(())
}

