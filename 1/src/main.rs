use serde::Deserialize;
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader, Lines},
    net::{TcpListener, TcpStream},
};

#[derive(Deserialize, Debug)]
struct Transfer {
    method: String,
    number: f32,
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
    let (reader, mut writer) = socket.split();

    let buf_reader = BufReader::new(reader);
    let mut lines = buf_reader.lines();

    while let Some(line) = lines.next_line().await.unwrap() {
        println!("{}", line);

        let input: Result<Transfer, _> = serde_json::from_str(&line);

        match input {
            Ok(input) => {
                println!("{:?}", input);

                let output = format!("Method: {}\nNumber: {}", input.method, input.number);

                writer.write_all(output.as_ref()).await.unwrap();
            }
            Err(err) => {
                println!("{:?}", err);

                writer
                    .write_all(b"{\"method\": \"malformed\", \"prime\": false}")
                    .await
                    .unwrap();

                return;
            }
        }
    }
}
