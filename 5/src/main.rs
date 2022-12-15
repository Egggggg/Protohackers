mod session;

use tokio::net::TcpListener;

use session::handle_session;

const ADDR: &'static str = "7YWHMfk9JZe0LM0g1ZauHuiSxhI";
const ENDPOINT: &'static str = "chat.protohackers.com:16963";

#[tokio::main]
async fn main() {
    let addr = "0.0.0.0:8080";
    let listener = TcpListener::bind(addr).await.unwrap();

    loop {
        let (mut socket, addr) = listener.accept().await.unwrap();

        tokio::spawn(async move {
            handle_session(socket).await;
        });
    }
}
