extern crate traildb;
extern crate prettytable;
use traildb::{Db, Event};
use std::path::Path;
use prettytable::Table;
use prettytable::Row;
use prettytable::Cell;

fn table_from_event(event: Event, header: Row, db: &Db) -> Table {
    let mut table = Table::new();
    table.add_row(header);
    let mut row_cells = Vec::new();
    row_cells.push(Cell::new(&format!("{}", event.timestamp)));
    for item in event.items {
        match db.get_item_value(*item) {
            Some(item) => {
                let cell = Cell::new(item);
                &row_cells.push(cell);
            },
            _ => {},
        }
    }
    table.add_row(Row::new(row_cells));
    table
}

fn header_from_db(db: &Db) -> Row {
    let mut cells = Vec::new();
    for i in 0..db.num_fields() as traildb::Field {
        cells.push(Cell::new(db.get_field_name(i).unwrap()));
    }
    Row::new(cells)
}

fn main() {
    // open the example db
    let db_path = Path::new("assets/wikipedia-history-small.tdb");
    let db = Db::open(db_path).unwrap();
    let header_row = header_from_db(&db);

    // iterate through trails (unique users)
    for trail in db.iter() {
        // iterate through the users wikipedia edits
        for event in trail {
            let table = table_from_event(event, header_row.clone(), &db);
            println!("{}", table);
        }
    }
}
