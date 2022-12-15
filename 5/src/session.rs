use tokio::{
    io::{AsyncBufReadExt, BufReader},
    net::TcpStream,
};

use crate::ENDPOINT;

pub async fn handle_session(mut user: TcpStream) {
    let mut server = TcpStream::connect(ENDPOINT).await.unwrap();

    let (user_reader, mut user_writer) = user.split();
    let (server_reader, mut server_writer) = server.split();

    let mut user_reader = BufReader::new(user_reader);
    let mut server_reader = BufReader::new(server_reader);

    let mut user_buf: Vec<u8> = Vec::new();
    let mut server_buf: Vec<u8> = Vec::new();

    loop {
        tokio::select! {
            _ = user_reader.read_until(b'\n', &mut user_buf) => {

            }
            _ = server_reader.read_until(b'\n', &mut server_buf) => {}
        }
    }
}
