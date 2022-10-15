use serde::Deserialize;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
};

#[derive(Deserialize, Debug)]
struct Transfer {
    method: String,
    number: f32,
}

const MALFORMED: &'static [u8; 39] = b"{\"method\": \"malformed\", \"prime\": false}";
const PRIME: &'static [u8; 36] = b"{\"method\": \"isPrime\", \"prime\": true}";
const NOT_PRIME: &'static [u8; 37] = b"{\"method\": \"isPrime\", \"prime\": false}";

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

                if input.method != "isPrime" {
                    writer.write_all(MALFORMED).await.unwrap();
                    return;
                }

                if input.number.trunc() != input.number {
                    writer.write_all(NOT_PRIME).await.unwrap();
                    continue;
                }

                let abs = input.number.abs();
                let root = abs.sqrt();
                let mut factor_found = false;

                println!("abs: {}, root: {}", abs, root);

                for i in 2..=root.ceil() as u32 {
                    let result = abs / i as f32;

                    if result.trunc() == result {
                        writer.write_all(NOT_PRIME).await.unwrap();
                        factor_found = true;
                        break;
                    }
                }

                if !factor_found {
                    writer.write_all(PRIME).await.unwrap();
                }
            }
            Err(err) => {
                println!("{:?}", err);

                writer.write_all(MALFORMED).await.unwrap();

                return;
            }
        }
    }
}
