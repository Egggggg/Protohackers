use std::net::SocketAddr;

use regex::Regex;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
};

use crate::{ADDR, ENDPOINT};

pub async fn handle_session(mut user: TcpStream, addr: SocketAddr) {
    let mut server = TcpStream::connect(ENDPOINT).await.unwrap();

    let (user_reader, mut user_writer) = user.split();
    let (server_reader, mut server_writer) = server.split();

    let mut user_reader = BufReader::new(user_reader);
    let mut server_reader = BufReader::new(server_reader);

    let mut user_buf: Vec<u8> = Vec::new();
    let mut server_buf: Vec<u8> = Vec::new();
    let re = Regex::new(r"(^ ?(7[[:alnum:]]{25,34}))|((7[[:alnum:]]{25,34}) ?\n$)").unwrap();

    loop {
        // look for boguscoin address and replace with 7YWHMfk9JZe0LM0g1ZauHuiSxhI
        tokio::select! {
            _ = user_reader.read_until(b'\n', &mut user_buf) => {
                // user sent a message
                let Ok(msg) = String::from_utf8(user_buf.clone()) else {
                    return;
                };

                user_buf.clear();

                if msg.len() == 0 {
                    return;
                }

                println!("Message `{msg}` from `{addr}`");

                let msg = re.replace(&msg, ADDR);

                server_writer.write_all(&*msg.as_bytes()).await.unwrap();

            }
            _ = server_reader.read_until(b'\n', &mut server_buf) => {
                // server sent a message
                let Ok(msg) = String::from_utf8(server_buf.clone()) else {
                    return;
                };

                server_buf.clear();

                if msg.len() == 0 {
                    return;
                }

                println!("Message `{msg}` from server to `{addr}`");

                let msg = re.replace(&msg, ADDR);

                user_writer.write_all(&*msg.as_bytes()).await.unwrap();}
        }

        println!("");
    }
}
