use std::env::{self};

use sorted_vec::SortedVec;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
    sync::mpsc::{self, Sender},
};

#[derive(Debug, Clone, Copy)]
struct Point {
    timestamp: i32,
    price: i32,
}

#[derive(Debug)]
struct Query {
    start: i32,
    end: i32,
    total: i64,
    amount: i64,
    mean: i32,
}

#[derive(Debug)]
struct QueryLog {
    full_history: History,
    history: History,
    query: Query,
    sexed: bool,
    idx: usize,
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
    let removal = std::fs::remove_dir_all("logs");

    if removal.is_err() {
    } else {
        removal.unwrap();
    }

    std::fs::create_dir("logs").unwrap();

    loop {
        let (mut socket, addr) = listener.accept().await.unwrap();

        println!("Connection from {}", &addr);

        let (tx, mut rx) = mpsc::channel::<QueryLog>(16);

        tokio::spawn(async move {
            process(&mut socket, tx).await;
        });

        tokio::spawn(async move {
            let dir = format!("{}", addr);
            let mut count = 0;
            let path = format!("logs/{}", &dir);

            println!("{:?}", path);

            tokio::fs::create_dir(path).await.unwrap();

            while let Some(message) = rx.recv().await {
                println!("message");

                count += 1;
                let output = format!("{:#?}", message);
                let output_buf = output.as_bytes();
                println!("logs, {}, {}", &dir, &count.to_string());

                let path = format!("logs/{}/{}", &dir, &count.to_string());

                let mut file = tokio::fs::File::create(path).await.unwrap();
                file.write_all(output_buf).await.unwrap();
            }
        });
    }
}

async fn process(socket: &mut TcpStream, tx: Sender<QueryLog>) {
    println!("process");

    let (reader, mut writer) = socket.split();
    let mut buf_reader = BufReader::new(reader);

    let mut history = History::new();
    let mut sexed = false;

    loop {
        let command = &mut [0u8; 9];
        let read = buf_reader.read_exact(command).await;

        if read.is_err() {
            return;
        }

        read.unwrap();

        println!("new command: {:?}", command);

        let command_type = command[0];

        if command_type == b'I' {
            // Insert command

            let timestamp_buf: [u8; 4] = command[1..=4].try_into().unwrap();
            let price_buf: [u8; 4] = command[5..=8].try_into().unwrap();

            let timestamp = i32::from_be_bytes(timestamp_buf);
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

                let log = QueryLog {
                    full_history: history.clone(),
                    history: history.clone(),
                    query: Query {
                        start: 0,
                        end: 0,
                        total: 0,
                        amount: 0,
                        mean: 69,
                    },
                    sexed,
                    idx: 0,
                };

                tx.send(log).await.unwrap();

                continue;
            }

            let start_buf: [u8; 4] = command[1..=4].try_into().unwrap();
            let end_buf: [u8; 4] = command[5..=8].try_into().unwrap();

            let start = i32::from_be_bytes(start_buf);
            let end = i32::from_be_bytes(end_buf);

            println!("start: {}, end: {}", start, end);

            let mut idx: usize = 0;

            for i in (0..history.len()).rev() {
                if history[i].timestamp < start {
                    idx = i + 1;
                    break;
                }
            }

            let mut amount = 0;
            let mut total: i64 = 0;
            let mut subhistory = History::new();

            for i in idx..history.len() {
                if history[i].timestamp > end {
                    break;
                }

                println!("price: {}, total: {total}", history[i].price);

                amount += 1;
                total += history[i].price as i64;
                subhistory.insert(history[i]);

                println!("timestamp: {}, end: {end}", history[i].timestamp);
            }

            if amount == 0 {
                println!("0");

                let log = QueryLog {
                    full_history: history.clone(),
                    history: History::new(),
                    query: Query {
                        start,
                        end,
                        total,
                        amount,
                        mean: 0,
                    },
                    sexed,
                    idx,
                };

                tx.send(log).await.unwrap();
                writer.write_i32(0).await.unwrap();
            } else {
                let mean = (total / amount) as i32;

                println!("total: {}, amount: {}", total, amount);

                println!("mean: {}", mean);

                let log = QueryLog {
                    full_history: history.clone(),
                    history: subhistory,
                    query: Query {
                        start,
                        end,
                        total,
                        amount,
                        mean,
                    },
                    sexed,
                    idx,
                };

                tx.send(log).await.unwrap();
                writer.write_i32(mean).await.unwrap();
            }
        }
    }
}
