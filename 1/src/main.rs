use std::env;

use serde::Deserialize;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
};

#[derive(Deserialize, Debug)]
struct Transfer {
    method: String,
    number: f64,
}

const MALFORMED: &'static [u8; 40] = b"{\"method\": \"malformed\", \"prime\": false}\n";
const PRIME: &'static [u8; 37] = b"{\"method\": \"isPrime\", \"prime\": true}\n";
const NOT_PRIME: &'static [u8; 38] = b"{\"method\": \"isPrime\", \"prime\": false}\n";

#[tokio::main]
async fn main() {
    let mut addr = "0.0.0.0:8080";

    if env::var("STAGE").unwrap_or("none".to_owned()) == "dev" {
        addr = "127.0.0.1:8080";
    }

    let listener = TcpListener::bind(addr).await.unwrap();

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

    loop {
        println!("loop");

        let line = lines.next_line().await.unwrap_or(None);

        if line.is_none() {
            return;
        }

        let line = line.unwrap();

        println!("{}", line);

        let input: Result<Transfer, _> = serde_json::from_str(&line);

        match input {
            Ok(input) => {
                println!("{:?}", input);

                if input.method != "isPrime" {
                    println!("malformed");
                    writer.write_all(MALFORMED).await.unwrap();
                    return;
                }

                if input.number.trunc() != input.number {
                    println!("trunc: {}, normal: {}", input.number.trunc(), input.number);
                    println!("not prime");
                    writer.write_all(NOT_PRIME).await.unwrap();
                    continue;
                }

                let number = input.number;

                println!("number: {}, input.number: {}", number, input.number);

                if number < 2.0 {
                    println!("not prime");
                    writer.write_all(NOT_PRIME).await.unwrap();
                    continue;
                }

                if number == 2.0 {
                    println!("prime");
                    writer.write_all(PRIME).await.unwrap();
                    continue;
                }

                let root = number.sqrt();
                let mut factor_found = false;

                println!("number: {}, root: {}", number, root);

                for i in 2..=root.ceil() as u32 {
                    let result = number / i as f64;

                    if result.trunc() == result {
                        println!("{}: not prime", input.number);
                        writer.write_all(NOT_PRIME).await.unwrap();
                        factor_found = true;
                        break;
                    }
                }

                println!("factor found: {}", factor_found);

                if !factor_found {
                    println!("{}: prime", input.number);
                    writer.write_all(PRIME).await.unwrap();
                    continue;
                }
            }
            Err(err) => {
                println!("{:?}", err);

                println!("malformed");
                writer.write_all(MALFORMED).await.unwrap();

                return;
            }
        }
    }
}
