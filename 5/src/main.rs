use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let addr = "0.0.0.0:8080";
    let listener = TcpListener::bind(addr).await.unwrap();
}
