use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();

    loop {
        let (listener, _) = listener.accept().await.unwrap();

        tokio::spawn(async move {
            let (mut reader, mut writer) = tokio::io::split(listener);

            tokio::io::copy(&mut reader, &mut writer).await.unwrap();
        });
    }
}
