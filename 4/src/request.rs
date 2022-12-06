use crate::Database;

pub fn handle_request(buf: Vec<u8>, db: Database) -> Option<String> {
    let req = String::from_utf8(buf).map_err(|_| ()).unwrap();
    let req = req.trim();

    dbg!(&req);

    if req.contains('=') {
        // insertions are '{key}={value}', they always have an '='
        let (key, value) = req.split_once('=').unwrap();
        db.lock().unwrap().insert(key.to_owned(), value.to_owned());
        // insertions do not respond
        None
    } else {
        // all other requests are retrievals
        let db = db.lock().unwrap();

        if req == "version" {
            return Some("version=69 :)".to_owned());
        }

        let value = db.get(req);

        let value = match value {
            Some(s) => s,
            None => "",
        };

        let output = format!("{req}={value}");

        Some(output)
    }
}
