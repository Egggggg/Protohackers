use std::{
    env,
    sync::{Arc, Mutex},
};

use regex::Regex;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{tcp::ReadHalf, TcpListener, TcpStream},
    sync::broadcast::{self, Sender},
};

type ID = u8;
type Input = Vec<u8>;
type UsedIDs = [bool; (ID::MAX as usize) + 1];
type UserListArc = Arc<Mutex<UserList>>;

struct User {
    name: Input,
    id: ID,
}

#[derive(Clone)]
struct UserMessage {
    from_id: ID,
    from_name: Input,
    content: Input,
}

// Special is the ID of the user the message refers to, include sends it to
// only them if true or everyone except them if false
#[derive(Clone)]
struct SysMessage {
    special: ID,
    content: Input,
    include: bool,
}

struct UserList {
    users: Vec<User>,
    used_ids: UsedIDs,
}

impl UserList {
    fn insert(&mut self, name: Input) -> Result<(), ()> {
        let re = Regex::new("[a-zA-Z0-9]{1,32}").unwrap();

        if !re.is_match(String::from_utf8(name).unwrap().as_ref()) {
            return Err(());
        }

        let mut id: ID = 0;

        for (idx, taken) in self.used_ids.iter().enumerate() {
            if !taken {
                id = idx as ID;
            }
        }

        let user = User { name, id };

        self.users.push(user);
        self.used_ids[id as usize] = true;

        Ok(())
    }
}

#[derive(Clone)]
enum Message {
    User(UserMessage),
    Sys(SysMessage),
}

#[tokio::main]
async fn main() {
    let mut addr = "0.0.0.0:8080";

    if env::var("STAGE").unwrap_or("none".to_owned()) == "dev" {
        addr = "127.0.0.1:8080";
    }

    let listener = TcpListener::bind(addr).await.unwrap();
    let (tx, mut rx) = broadcast::channel::<Message>(32);

    let tx_arc = Arc::new(tx);
    let users = Arc::new(Mutex::new(UserList {
        users: Vec::new(),
        used_ids: [false; (ID::MAX as usize) + 1],
    }));

    // Accept loop
    loop {
        let (mut socket, addr) = listener.accept().await.unwrap();

        println!("Connection from {}", addr);

        let tx = tx_arc.clone();
        let users = users.clone();

        tokio::spawn(async move {
            process(&mut socket, tx, users).await;
        });
    }
}

async fn process(socket: &mut TcpStream, tx: Arc<Sender<Message>>, users: UserListArc) {
    let (reader, mut writer) = socket.split();
    let mut buf_reader = BufReader::new(reader);

    writer
        .write_all(b"welcome to ghoul chat... who are you ?")
        .await
        .unwrap();

    let mut username: Vec<u8> = Vec::new();

    buf_reader.read_until(b'\n', &mut username).await.unwrap();
    users.lock().unwrap().insert(username);

    let chat_tx = tx.clone();

    chat(buf_reader, chat_tx).await;

    tx.send()
}

async fn chat(reader: BufReader<ReadHalf<'_>>, tx: Arc<Sender<Message>>) {}
