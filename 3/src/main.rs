use std::{
    env,
    sync::{Arc, Mutex},
};

use regex::Regex;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{
        tcp::{ReadHalf, WriteHalf},
        TcpListener, TcpStream,
    },
    sync::broadcast::{self, Sender},
};

const USER_LIMIT: usize = 20;

type ID = u8;
type UserListArc = Arc<Mutex<UserList>>;

#[derive(Debug, Clone)]
enum Message {
    User(UserMessage),
    Sys(SysMessage),
}

#[derive(Clone, Debug)]
enum Error {
    InvalidName,
    Full,
}

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
    only: bool,
}

#[derive(Debug)]
struct UserList {
    users: [Option<User>; USER_LIMIT],
}

impl UserList {
    fn insert(&mut self, name: String) -> Result<ID, Error> {
        let re = Regex::new("^[a-zA-Z0-9]{1,32}$").unwrap();

        dbg!(&re);

        if !re.is_match(name.as_ref()) {
            return Err(Error::InvalidName);
        }

        let mut idx: Option<ID> = None;

        for (i, user) in self.users.iter().enumerate() {
            if user.is_none() {
                idx = Some(i as ID);
                break;
            }
        }

        if let Some(id) = idx {
            println!("New user: {}", name);
            let user = Some(User { name, id });

            self.users[id as usize] = user;

            Ok(id)
        } else {
            println!("Too many ghouls");
            Err(Error::Full)
        }
    }

    fn list_users(&self, to: ID) -> String {
        let mut users: Vec<&User> = Vec::new();

        for (i, user) in self.users.iter().enumerate() {
            if i == to as usize {
                continue;
            }

            if let Some(user) = user {
                users.push(user);
            }
        }

        if users.len() == 0 {
            return "* you are the first ghoulie\r\n".to_owned();
        }

        let mut output = format!("* current ghouls: {}", users[0].name);

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

    fn remove(&mut self, id: ID) {
        self.users[id as usize] = None;
    }
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
        users: Default::default(),
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
    let name = name.trim().to_owned();
    let inserted = users.lock().unwrap().insert(name.clone());
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
    let welcome = SysMessage {
        special: id,
        content: format!("* {} has arrived !\r\n", name),
        only: false,
    };

    println!("{} joined ({})", name, id);

    writer.write_all(user_list.as_bytes()).await.unwrap();
    tx.send(Message::Sys(welcome)).unwrap();

    let chat_tx = tx.clone();

    chat(&mut buf_reader, &mut writer, chat_tx, id, name.clone()).await;

    users.lock().unwrap().remove(id);

    let message = SysMessage {
        special: id,
        content: format!("* {} has left the locale\r\n", name),
        only: false,
    };

    tx.send(Message::Sys(message)).unwrap();

    println!("{} left ({})", name, id);
}

async fn chat(
    reader: &mut BufReader<ReadHalf<'_>>,
    writer: &mut WriteHalf<'_>,
    tx: Arc<Sender<Message>>,
    id: ID,
    name: String,
) {
    let mut rx = tx.subscribe();
    let mut msg_buf: Vec<u8> = Vec::new();

    loop {
        tokio::select! {
            _ = reader.read_until(b'\n', &mut msg_buf) => {
                dbg!(&msg_buf);

                if msg_buf.len() == 0 {
                    return;
                }

                if msg_buf.len() > 1500 {
                    let message = SysMessage {
                        special: id,
                        content: format!("that message has {} characters which is more than 1500\r\n", msg_buf.len()),
                        only: true,
                    };

                    tx.send(Message::Sys(message)).unwrap();
                    continue;
                }

                let msg = String::from_utf8(msg_buf).unwrap();

                print!("[{name}] {msg}");

                let message = UserMessage {
                    from_id: id,
                    from_name: name.clone(),
                    content: msg,
                };

                tx.send(Message::User(message)).unwrap();
                msg_buf = Vec::new();
            }
            Ok(message) = rx.recv() => {
                match message {
                    Message::User(message) => {
                        if message.from_id != id {
                            let content = format!("[{}] {}", message.from_name, message.content);

                            writer.write_all(content.as_bytes()).await.unwrap();
                        }
                    }
                    Message::Sys(message) => {
                        if message.only {
                            if message.special == id {
                                writer.write_all(message.content.as_bytes()).await.unwrap();
                            }
                        } else {
                            if message.special != id {
                                writer.write_all(message.content.as_bytes()).await.unwrap();
                            }
                        }
                    }
                }
            }
        }
    }
}
