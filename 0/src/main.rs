use std::env;

use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let mut addr = "0.0.0.0:8080";

    if env::var("STAGE").unwrap_or("none".to_owned()) == "dev" {
        addr = "127.0.0.1:8080";
    }

    let listener = TcpListener::bind(addr).await.unwrap();

    loop {
        let (mut socket, _) = listener.accept().await.unwrap();

        println!("nice");

        tokio::spawn(async move {
            let (mut reader, mut writer) = socket.split();

            tokio::io::copy(&mut reader, &mut writer).await.unwrap();
        });
    }
}
