use std::{
    env,
    sync::{Arc, Mutex},
};

use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{tcp::ReadHalf, TcpListener, TcpStream},
    sync::mpsc::{self, Sender},
};

struct User<'a> {
    name: &'a [u8],
    id: u16,
    tx: Sender<Message<'a>>,
}

struct NewUser<'a> {
    name: &'a [u8],
    tx: Sender<Message<'a>>,
}

#[derive(Clone, Copy)]
struct Message<'a> {
    from_id: u16,
    from_name: &'a [u8],
    content: &'a [u8],
}

struct UserList<'a> {
    users: Vec<User<'a>>,
    next_id: u16,
}

#[tokio::main]
async fn main() {
    let mut addr = "0.0.0.0:8080";

    if env::var("STAGE").unwrap_or("none".to_owned()) == "dev" {
        addr = "127.0.0.1:8080";
    }

    let listener = TcpListener::bind(addr).await.unwrap();
    let (tx, mut rx) = mpsc::channel::<Message>(32);
    let user_list = Arc::new(Mutex::new(UserList {
        users: Vec::new(),
        next_id: 0,
    }));

    let user_list_send = user_list.clone();

    // Message sender
    tokio::spawn(async move {
        while let Some(message) = rx.recv().await {
            for user in user_list.lock().unwrap().users.iter() {
                if user.id != message.from_id {
                    user.tx.send(message);
                }
            }
        }
    });

    // Accept loop
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
    let mut lines = buf_reader.lines();

    writer
        .write_all(b"welcome to ghoul chat... who are you ?")
        .await
        .unwrap();

    if let Some(username) = lines.next_line().await.unwrap() {}
}

async fn chat(reader: BufReader<ReadHalf<'_>>) {}
