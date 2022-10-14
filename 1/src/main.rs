use serde::{Deserialize, Serialize};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
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
            process(&mut socket).await;
        });
    }
}

async fn process(socket: &mut TcpStream) {
    loop {
        let mut content = vec![0; 64];

        loop {
            let next = socket.read_u8().await;

            match next {
                Ok(real) => {
                    if real == b'\n' {
                        break;
                    }

                    content.push(real);
                }
                Err(_) => {
                    return;
                }
            }
        }

        let nice = String::from_utf8(content).unwrap();

        println!("Content: {}", nice);

        socket.write_all(nice.as_ref()).await.unwrap();
    }
}
