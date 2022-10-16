use std::env;

use sorted_vec::SortedVec;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
};

#[derive(Debug)]
struct Point {
    timestamp: u32,
    price: i32,
}

type History = SortedVec<Point>;

impl PartialOrd for Point {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.timestamp.partial_cmp(&other.timestamp)
    }
}

impl Ord for Point {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.timestamp.cmp(&other.timestamp)
    }
}

impl PartialEq for Point {
    fn eq(&self, other: &Self) -> bool {
        self.timestamp.eq(&other.timestamp)
    }
}

impl Eq for Point {}

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
    let mut buf_reader = BufReader::new(reader);

    let mut history = History::new();
    let mut sexed = false;

    loop {
        let command = &mut [0u8; 9];
        buf_reader.read_exact(command).await.unwrap();

        println!("new command: {:?}", command);

        let command_type = command[0];

        if command_type == b'I' {
            // Insert command

            let timestamp_buf: [u8; 4] = command[1..=4].try_into().unwrap();
            let price_buf: [u8; 4] = command[5..=8].try_into().unwrap();

            let timestamp = u32::from_be_bytes(timestamp_buf);
            let price = i32::from_be_bytes(price_buf);

            let point = Point { timestamp, price };

            println!("{:?}", point);

            if let Ok(_) = history.binary_search(&point) {
                sexed = true;
            }

            history.insert(point);
        } else if command_type == b'Q' {
            // Query command

            if sexed {
                writer.write_all(&[69; 1]).await.unwrap();
                continue;
            }

            let start_buf: [u8; 4] = command[1..=4].try_into().unwrap();
            let end_buf: [u8; 4] = command[5..=8].try_into().unwrap();

            let start = u32::from_be_bytes(start_buf);
            let end = u32::from_be_bytes(end_buf);

            let mut idx: usize;

            for i in history.len()..=0 {
                if history[i].timestamp < start {
                    idx = i;
                    break;
                }
            }
        }
    }
}
