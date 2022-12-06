mod request;

use std::{
    collections::HashMap,
    env, io,
    sync::{Arc, Mutex},
};

use tokio::net::UdpSocket;

use request::handle_request;

pub type Database = Arc<Mutex<HashMap<String, String>>>;

#[tokio::main]
async fn main() -> io::Result<()> {
    let addr = if env::var("STAGE").unwrap_or("none".to_owned()) == "dev" {
        "127.0.0.1:8080"
    } else {
        "0.0.0.0:8080"
    };

    let socket = Arc::new(UdpSocket::bind(addr).await?);
    let db: Database = Arc::new(Mutex::new(HashMap::new()));

    loop {
        // wait for the socket to be readable
        socket.readable().await?;

        let mut buf = vec![0u8; 999];
        let (len, addr) = socket.recv_from(&mut buf).await?;

        // only allow requests under 1000 bytes
        if len >= 1000 {
            continue;
        }

        // wait for the socket to be writable
        socket.writable().await?;

        let socket_clone = socket.clone();
        let db_clone = db.clone();
        let buf = (&buf[..len]).to_vec();

        // spawn a thread to handle the request
        tokio::spawn(async move {
            if let Some(res) = handle_request(buf, db_clone) {
                let res = res.as_bytes();

                let sent = socket_clone.send_to(res, addr).await;

                dbg!(sent);
            }
        });
    }
}
