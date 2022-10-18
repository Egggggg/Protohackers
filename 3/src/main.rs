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

const USER_LIMIT: usize = 20;

type ID = u8;
type UsedIDs = [bool; USER_LIMIT];
type UserListArc = Arc<Mutex<UserList>>;

#[derive(Debug, Clone)]
struct User {
    name: String,
    id: ID,
}

#[derive(Clone, Debug)]
struct UserMessage {
    from_id: ID,
    from_name: String,
    content: String,
}

// Special is the ID of the user the message refers to, include sends it to
// only them if true or everyone except them if false
#[derive(Clone, Debug)]
struct SysMessage {
    special: ID,
    content: String,
    include: bool,
}

#[derive(Debug)]
struct UserList {
    users: Vec<User>,
    used_ids: UsedIDs,
}

impl UserList {
    fn insert(&mut self, name: String) -> Result<ID, Error> {
        let re = Regex::new("[a-zA-Z0-9]{1,32}").unwrap();

        if !re.is_match(name.as_ref()) {
            return Err(Error::InvalidName);
        }

        let mut idx: Option<ID> = None;

        for (i, taken) in self.used_ids.iter().enumerate() {
            if !taken {
                idx = Some(i as ID);
                break;
            }
        }

        if let Some(id) = idx {
            println!("New user: {}", name);
            let user = User { name, id };

            self.users.push(user);
            self.used_ids[id as usize] = true;
            dbg!(self);

            Ok(id)
        } else {
            println!("Name taken");
            Err(Error::Full)
        }
    }

    fn list_users(&self, to: ID) -> String {
        let mut users = self.users.clone();
        let idx = users.binary_search_by(|f| f.id.cmp(&to));

        if idx.is_err() {
            return "* we fucked up\r\n".to_owned();
        }

        users.remove(idx.unwrap());

        let mut output = format!("* current ghouls: {}", self.users[0].name);

        if users.len() == 0 {
            return "* you are the first ghoulie\r\n".to_owned();
        }

        if users.len() > 1 {
            for (i, user) in users.iter().enumerate() {
                if i == 0 || i == users.len() - 1 {
                    continue;
                }

                output = format!("{}, {}", output, user.name);
            }

            output = format!("{} and {}", output, users[users.len() - 1].name);
        }

        output = format!("{}\r\n", output);
        output
    }
}

#[derive(Clone)]
enum Message {
    User(UserMessage),
    Sys(SysMessage),
}

#[derive(Clone, Debug)]
enum Error {
    InvalidName,
    Full,
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
        used_ids: [false; USER_LIMIT],
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
        .write_all(b"* welcome to ghoul chat... who are you ?\r\n")
        .await
        .unwrap();

    let mut name_buf: Vec<u8> = Vec::new();

    buf_reader.read_until(b'\n', &mut name_buf).await.unwrap();

    let name = String::from_utf8(name_buf).unwrap();
    let inserted = users.lock().unwrap().insert(name.trim().to_owned());
    let mut id: ID = 0;

    match inserted {
        Ok(e) => id = e,
        Err(Error::Full) => {
            writer
                .write_all(b"* ghoul chat is full :(\r\n")
                .await
                .unwrap();
            return;
        }
        Err(Error::InvalidName) => {
            writer
                .write_all(b"* invalid name, valid names are 1-32 alphanumeric characters\r\n")
                .await
                .unwrap();
            return;
        }
    }

    let user_list = users.lock().unwrap().list_users(id);

    writer.write_all(user_list.as_bytes()).await.unwrap();

    let chat_tx = tx.clone();

    chat(buf_reader, chat_tx).await;
}

async fn chat(reader: BufReader<ReadHalf<'_>>, tx: Arc<Sender<Message>>) {
    let rx = tx.subscribe();

    while let Some(message) = rx.recv().await {}
}
