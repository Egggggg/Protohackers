use std::{net::SocketAddr, sync::Arc};

use tokio::net::UdpSocket;

use crate::Database;

pub async fn handle_request(
    addr: SocketAddr,
    buf: Vec<u8>,
    socket: Arc<UdpSocket>,
    db: Database,
) -> Result<(), ()> {
    let req = String::from_utf8(buf).map_err(|_| ()).unwrap();

    dbg!(&req);

    if req.contains('=') {
        // insertions are '{key}={value}', they always have an '='
        let (key, value) = req.split_once('=').unwrap();
        db.lock().unwrap().insert(key.to_owned(), value.to_owned());
        // insertions do not respond
    } else {
        // all other requests are retrievals
        let db = db.lock().unwrap();
        let value = db.get(&req);

        let value = match value {
            Some(s) => s,
            None => "",
        };

        let output = format!("{req}={value}");

        socket.send_to(output.as_bytes(), addr).await.unwrap();
    }

    Ok(())
}
