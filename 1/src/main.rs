use serde::{Deserialize, Serialize};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};

#[derive(Serialize, Deserialize, Debug)]
struct Transfer {
    method: String,
    number: f64,
}

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();

    loop {
        let (mut socket, addr) = listener.accept().await.unwrap();

        println!("Connection from {}", addr);

        tokio::spawn(async move {
            loop {
                let mut content = "".to_owned();
                socket.read_to_string(&mut content).await.unwrap();

                println!("Content: {}\n\n", content);

                socket.write_all(content.as_ref()).await.unwrap();
            }
        });
    }
}
