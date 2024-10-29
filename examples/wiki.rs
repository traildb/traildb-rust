extern crate traildb;
use traildb::Db;
use std::path::Path;

const SESSION_LIMIT: u64 = 30 * 60; // at least 30 minutes between edits

fn main() {
    // open the example db
    let db_path = Path::new("assets/wikipedia-history-small.tdb");
    let db = Db::open(db_path).unwrap();

    // iterate through some of the events
    for mut trail in db.iter() {
        let id = trail.id;
        if let Some(event) = trail.next() {
            let mut prev_time = event.timestamp;
            let mut session_cnt = 1;
            let mut event_cnt = 1;
            for event in trail {
                if event.timestamp - prev_time > SESSION_LIMIT {
                    session_cnt += 1;
                }
                prev_time = event.timestamp;
                event_cnt += 1;
            }
            println!("Trail[{}] Number of Sessions: {} Number of Events: {}", id, session_cnt, event_cnt);
        }
    }
}
