use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();

    loop {
        let (mut socket, _) = listener.accept().await.unwrap();

        println!("nice");

        tokio::spawn(async move {
            let (mut reader, mut writer) = socket.split();

            tokio::io::copy(&mut reader, &mut writer).await.unwrap();
        });
    }
}
