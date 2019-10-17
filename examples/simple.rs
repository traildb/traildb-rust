// Run this with cargo run --example simple

extern crate traildb;
extern crate uuid;

use std::path::Path;


fn main() {
    let path = Path::new("tiny");
    let fields = ["username", "action"];
    let mut cons = traildb::Constructor::new(path, &fields).expect("Could not create constructor");

    let events = vec!["open".to_owned(), "save".to_owned(), "close".to_owned()];

    for i in 0..3 {
        let uuid = uuid::Uuid::new_v4();
        let username = format!("user{}", i);
        for (day, action) in events.clone().into_iter().enumerate() {
            let timestamp = (i * 10) + day;
            let values = vec![username.as_str(), action.as_str()];
            cons.add(uuid.as_bytes(), timestamp as u64, &values).expect("Could not add")
        }
    }
    cons.finalize().expect("Could not finalize");

    let db = traildb::Db::open(path).expect("Could not open for reading");
    let mut cursor = db.cursor();
    for i in 0..db.num_trails() {
        let uuid = db.get_uuid(i).expect("Could not read uuid");
        let uuid = uuid::Uuid::from_bytes(*uuid);

        cursor.get_trail(i).expect("Could not read trail");

        while let Some(event) = cursor.next() {
            print!("{:?}, [ timestamp={}", uuid, event.timestamp);
            for (j, item) in event.items.iter().enumerate() {
                let value = db.get_item_value(*item).unwrap();
                print!(" {}={}", fields[j], value)
            }
            println!(" ]");
        }
    }
}
