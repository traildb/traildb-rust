#[allow(non_camel_case_types,dead_code,non_snake_case,private_in_public)]
mod ffi;
use std::path::Path;
use std::ffi::CString;
use std::fmt;
use std::mem::{transmute, forget};


use std::collections::HashMap;

#[derive(Debug)]
#[derive(PartialEq)]
#[repr(C)]
pub enum Error {
    Nomem = -2,
    PathTooLong = -3,
    UnknownField = -4,
    UnknownUuid = -5,
    InvalidTrailId = -6,
    HandleIsNull = -7,
    HandleAlreadyOpened = -8,
    UnknownOption = -9,
    InvalidOptionValue = -10,
    InvalidUuid = -11,
    IoOpen = -65,
    IoClose = -66,
    IoWrite = -67,
    IoRead = -68,
    IoTruncate = -69,
    IoPackage = -70,
    InvalidInfoFile = -129,
    InvalidVersionFile = -130,
    IncompatibleVersion = -131,
    InvalidFieldsFile = -132,
    InvalidUuidsFile = -133,
    InvalidCodebookFile = -134,
    InvalidTrailsFile = -135,
    InvalidLexiconFile = -136,
    InvalidPackage = -137,
    TooManyFields = -257,
    DuplicateFields = -258,
    InvalidFieldname = -259,
    TooManyTrails = -260,
    ValueTooLong = -261,
    AppendFieldsMismatch = -262,
    LexiconTooLarge = -263,
    TimestampTooLarge = -264,
    TrailTooLong = -265,
    OnlyDiffFilter = -513,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match *self {
            Error::Nomem => "Nomem",
            Error::PathTooLong => "PathTooLong",
            Error::UnknownField => "UnknownField",
            Error::UnknownUuid => "UnknownUuid",
            Error::InvalidTrailId => "InvalidTrailId",
            Error::HandleIsNull => "HandleIsNull",
            Error::HandleAlreadyOpened => "HandleAlreadyOpened",
            Error::UnknownOption => "UnknownOption",
            Error::InvalidOptionValue => "InvalidOptionValue",
            Error::InvalidUuid => "InvalidUuid",
            Error::IoOpen => "IoOpen",
            Error::IoClose => "IoClose",
            Error::IoWrite => "IoWrite",
            Error::IoRead => "IoRead",
            Error::IoTruncate => "IoTruncate",
            Error::IoPackage => "IoPackage",
            Error::InvalidInfoFile => "InvalidInfoFile",
            Error::InvalidVersionFile => "InvalidVersionFile",
            Error::IncompatibleVersion => "IncompatibleVersion",
            Error::InvalidFieldsFile => "InvalidFieldsFile",
            Error::InvalidUuidsFile => "InvalidUuidsFile",
            Error::InvalidCodebookFile => "InvalidCodebookFile",
            Error::InvalidTrailsFile => "InvalidTrailsFile",
            Error::InvalidLexiconFile => "InvalidLexiconFile",
            Error::InvalidPackage => "InvalidPackage",
            Error::TooManyFields => "TooManyFields",
            Error::DuplicateFields => "DuplicateFields",
            Error::InvalidFieldname => "InvalidFieldname",
            Error::TooManyTrails => "TooManyTrails",
            Error::ValueTooLong => "ValueTooLong",
            Error::AppendFieldsMismatch => "AppendFieldsMismatch",
            Error::LexiconTooLarge => "LexiconTooLarge",
            Error::TimestampTooLarge => "TimestampTooLarge",
            Error::TrailTooLong => "TrailTooLong",
            Error::OnlyDiffFilter => "OnlyDiffFilter",
        };
        write!(f, "Error::{}", s)
    }
}

/// Convert a `tdb_error` either to either a `Ok(T)` or `Err(Error)`
fn wrap_tdb_err<T>(err: ffi::tdb_error, val: T) -> Result<T, Error> {
    match err {
        ffi::tdb_error::TDB_ERR_OK => Ok(val),
        _ => Err(unsafe { transmute(err) }),
    }
}

/// A timestamp must provided with added events.
pub type Timestamp = u64;
/// The type returned by `Db::version`.
pub type Version = u64;
/// An integer type that identifies an individual traul in a `Db`.
pub type TrailId = u64;
/// A [UUID](https://en.wikipedia.org/wiki/Universally_unique_identifier)
/// must be included with all added events.
pub type Uuid = [u8; 16];

/// TODO: Document me
#[derive(Debug,Clone,Copy,PartialEq,Eq,Hash)]
pub struct Item(pub u64);
/// TODO: Document me
pub type Value = u64;
/// TODO: Document me
pub type Field = u32;



/// A structure that represents a `TrailDB` constructor.
///
/// A constructor lives in RAM. All events are added to the constructor.
/// After being written to disk, it the `TrailDB` is immutable.
///
/// # Examples
///
/// ```
/// use traildb::{Constructor, Uuid};
/// use std::path::Path;
///
/// // Names relevent to our event type
/// let db_fields = ["user", "action"];
/// // Where to write our dabase to disk when we're done adding events to it
/// let db_path = Path::new("my_traildb");
/// // Create a constructor
/// let mut cons = Constructor::new(db_path, &db_fields).unwrap();
///
/// // Let's gather necessary data to create and event
/// // Time is stored as a `u64`. What that represents (e.g. UNIX time) is up to you
/// let timestamp: u64 = 0;
/// // Every trail need a UUID
/// let uuid: Uuid = [0u8;16];
/// // The values for for fields `"user"` and `"action"`
/// let event_vals = ["Alice", "login"];
///
/// // Now lets add our event data to the constructor
/// assert!(cons.add(&uuid, timestamp, &event_vals).is_ok());
///
/// // Finally, let's write our database to disk by calling `finalize`
/// assert!(cons.finalize().is_ok());
/// ```
pub struct Constructor {
    obj: *mut ffi::tdb_cons,
}

impl Constructor {
    /// Create a new TrailDB constructor.
    pub fn new(path: &Path, fields: &[&str]) -> Result<Self, Error> {

        let mut field_ptrs = Vec::new();
        for f in fields.iter() {
            let s = CString::new(*f).unwrap();
            field_ptrs.push(s.as_ptr());
            forget(s);
        }

        let ptr = unsafe { ffi::tdb_cons_init() };
        let ret = unsafe {
            ffi::tdb_cons_open(ptr,
                               path_cstr(path).as_ptr(),
                               field_ptrs.as_slice().as_ptr() as *mut *const i8,
                               field_ptrs.len() as u64)
        };
        wrap_tdb_err(ret, Constructor { obj: ptr })
    }

    /// Add an event to the constructor.
    pub fn add(&mut self, uuid: &Uuid, timestamp: Timestamp, values: &[&str]) -> Result<(), Error> {
        let mut val_ptrs = Vec::new();
        let mut val_lens = Vec::new();
        for v in values.iter() {
            val_ptrs.push(v.as_ptr());
            val_lens.push(v.len() as u64);
        }
        let ret = unsafe {
            ffi::tdb_cons_add(self.obj,
                              uuid.as_ptr() as *mut u8,
                              timestamp,
                              val_ptrs.as_slice().as_ptr() as *mut *const i8,
                              val_lens.as_slice().as_ptr() as *const u64)
        };
        wrap_tdb_err(ret, ())
    }

    /// Close a constructor without writing it to disk.
    pub fn close(&mut self) {
        unsafe { ffi::tdb_cons_close(self.obj) };
    }

    /// Write the TrailDB to disk and close it.
    pub fn finalize(&mut self) -> Result<(), Error> {
        let ret = unsafe { ffi::tdb_cons_finalize(self.obj) };
        wrap_tdb_err(ret, ())
    }

    /// Combine an alread finalized TrailDB with a constructor.
    pub fn append(&mut self, db: &Db) -> Result<(), Error> {
        let ret = unsafe { ffi::tdb_cons_append(self.obj, transmute(db)) };
        wrap_tdb_err(ret, ())
    }
}




pub struct Db<'a> {
    obj: &'a mut ffi::tdb
}

impl<'a> Db<'a> {
    pub fn open(path: &Path) -> Result<Self, Error> {
        let ptr = unsafe { ffi::tdb_init() };
        let ret = unsafe { ffi::tdb_open(ptr, path_cstr(path).as_ptr()) };
        unsafe { wrap_tdb_err(ret, Db { obj: transmute(ptr) }) }
    }

    pub fn close(&mut self) {
        unsafe {
            ffi::tdb_close(self.obj);
        }
    }

    pub fn num_trails(&self) -> u64 {
        unsafe { ffi::tdb_num_trails(self.obj) }
    }

    pub fn num_events(&self) -> u64 {
        unsafe { ffi::tdb_num_events(self.obj) }
    }

    pub fn num_fields(&self) -> u64 {
        unsafe { ffi::tdb_num_fields(self.obj) }
    }

    pub fn min_timestamp(&self) -> Timestamp {
        unsafe { ffi::tdb_min_timestamp(self.obj) }
    }

    pub fn max_timestamp(&self) -> Timestamp {
        unsafe { ffi::tdb_max_timestamp(self.obj) }
    }

    pub fn version(&self) -> Version {
        unsafe { ffi::tdb_version(self.obj) }
    }

    pub fn will_need(&self) {
        unsafe { ffi::tdb_willneed(self.obj) };
    }

    pub fn dont_need(&self) {
        unsafe { ffi::tdb_dontneed(self.obj) };
    }

    pub fn get_trail(&self, trail_id: TrailId) -> Option<Trail> {
        let mut cursor = self.cursor();
        if cursor.get_trail(trail_id).is_err() {
            return None;
        };
        Some(Trail {
            id: trail_id,
            cursor: cursor,
        })
    }

    pub fn get_trail_id(&self, uuid: &Uuid) -> Option<TrailId> {
        let mut id: TrailId = 0;
        let ret = unsafe {
            ffi::tdb_get_trail_id(self.obj, uuid.as_ptr() as *mut u8, &mut id as *mut TrailId)
        };
        match ret {
            ffi::tdb_error::TDB_ERR_OK => Some(id),
            _ => None,
        }
    }

    pub fn get_uuid(&self, trail_id: TrailId) -> Option<&Uuid> {
        unsafe {
            let ptr = ffi::tdb_get_uuid(self.obj, trail_id) as *const [u8; 16];
            ptr.as_ref()
        }
    }

    pub fn cursor(&'a self) -> Cursor<'a> {
        unsafe {
            let ptr = ffi::tdb_cursor_new(self.obj);
            Cursor { obj: transmute(ptr) }
        }
    }

    pub fn iter(&'a self) -> DbIter<'a> {
        DbIter { pos: 0, db: self }
    }

    pub fn get_item_value(&'a self, item: Item) -> &'a str {
        unsafe {
            let mut len = 0u64;
            let ptr = ffi::tdb_get_item_value(self.obj, transmute(item), &mut len);
            let s = std::slice::from_raw_parts(ptr as *const u8, len as usize);
            std::str::from_utf8_unchecked(s)
        }
    }

    pub fn get_item(&'a self, field: Field, value: &str) -> Option<Item> {
        unsafe {
            let item = ffi::tdb_get_item(self.obj,
                                         transmute(field),
                                         value.as_ptr() as *const i8,
                                         value.len() as u64);

            if item == 0 {
                None
            } else {
                Some(Item(item))
            }
        }
    }

    pub fn get_field_name(&'a self, field: Field) -> Option<&'a str> {
        unsafe {
            let ptr = ffi::tdb_get_field_name(self.obj, field);
            match std::ffi::CStr::from_ptr(ptr).to_str() {
                Ok(s) => Some(s),
                Err(_) => None,
            }
        }
    }

    pub fn lexicon_size(&'a self, field: Field) -> u64 {
        unsafe { ffi::tdb_lexicon_size(self.obj, field) }
    }

    pub fn lexicon(&'a self, field: Field) -> Vec<&'a str> {
        let mut vec = Vec::with_capacity(self.lexicon_size(field) as usize);
        for i in 1..self.lexicon_size(field) {
            let value = unsafe {
                let mut len = 0u64;
                let ptr = unsafe { ffi::tdb_get_value(self.obj, field, i, &mut len) };
                let s = std::slice::from_raw_parts(ptr as *const u8, len as usize);
                std::str::from_utf8_unchecked(s)
            };
            vec.push(value);
        }
        vec
    }

    pub fn fields(&'a self) -> HashMap<&str, Field> {
        let num_fields = self.num_fields();
        let mut fields: HashMap<&'a str, Field> = HashMap::with_capacity(num_fields as usize);

        for i in 1..num_fields {
            let field: Field = i as u32;
            let name = self.get_field_name(field).unwrap().clone();
            fields.insert(name, field);
        }
        fields
    }
}


impl<'a> Drop for Db<'a> {
    fn drop(&mut self) {
        unsafe {
            ffi::tdb_close(self.obj)
        };
    }
}




pub struct DbIter<'a> {
    pos: u64,
    db: &'a Db<'a>
}

impl<'a> Iterator for DbIter<'a> {
    type Item = Trail<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let id = self.pos;
        self.pos += 1;
        let mut cursor = self.db.cursor();
        match cursor.get_trail(id) {
            Err(_) => None,
            Ok(()) => {
                let trail = Trail {
                    id: id,
                    cursor: cursor,
                };
                Some(trail)
            }
        }
    }
}




pub struct Cursor<'a> {
    obj: &'a mut ffi::tdb_cursor,
}

impl<'a> Cursor<'a> {
    pub fn get_trail(&mut self, trail_id: TrailId) -> Result<(), Error> {
        let ret = unsafe { ffi::tdb_get_trail(self.obj, trail_id) };
        wrap_tdb_err(ret, ())
    }

    pub fn len(&mut self) -> u64 {
        unsafe { ffi::tdb_get_trail_length(self.obj) }
    }
}

impl<'a> Drop for Cursor<'a> {
    fn drop(&mut self) {
        unsafe { ffi::tdb_cursor_free(self.obj) };
    }
}

impl<'a> Iterator for Cursor<'a> {
    type Item = Event<'a>;

    fn next(&mut self) -> Option<Event<'a>> {
        unsafe {
            let e = ffi::tdb_cursor_next(self.obj);
            Event::from_tdb_event(e)
        }
    }
}




pub struct Trail<'a> {
    pub id: TrailId,
    cursor: Cursor<'a>,
}

impl<'a> Iterator for Trail<'a> {
    type Item = Event<'a>;

    fn next(&mut self) -> Option<Event<'a>> {
        self.cursor.next()
    }
}




fn path_cstr(path: &Path) -> CString {
    CString::new(path.to_str().unwrap()).unwrap()
}




pub struct Event<'a> {
    pub timestamp: Timestamp,
    pub items: &'a [Item],
}

impl<'a> Event<'a> {
    fn from_tdb_event(e: *const ffi::tdb_event) -> Option<Self> {
        unsafe {
            match e.as_ref() {
                None => None,
                Some(e) => {
                    Some(Event {
                        timestamp: e.timestamp,
                        items: std::slice::from_raw_parts(transmute(&e.items),
                                                          e.num_items as usize),
                    })
                }
            }
        }
    }
}




#[cfg(test)]
mod test_traildb {
    extern crate uuid;
    use super::{Constructor, Db, Field};
    use std::path::Path;

    #[test]
    #[no_mangle]
    fn test_traildb() {
        // create a new constructor
        let field_names = ["field1", "field2"];
        let db_path = Path::new("test");
        let mut cons = Constructor::new(db_path, &field_names).unwrap();
        let field_vals = ["cats", "dogs"];

        // add an event
        let events_per_trail = 10;
        let mut trail_cnt = 0;
        let mut event_cnt = 0;
        let mut uuids = Vec::new();
        let mut timestamp = 0;
        let mut timestamps = Vec::new();
        for _ in 0..10 {
            let uuid = *uuid::Uuid::new_v4().as_bytes();
            for _ in 0..events_per_trail {
                assert!(&cons.add(&uuid, timestamp, &field_vals).is_ok());
                timestamps.push(timestamp);
                event_cnt += 1;
                timestamp += 1;
            }
            uuids.push(uuid);
            trail_cnt += 1;
        }


        // finalize db (saves it to disk)
        assert!(cons.finalize().is_ok());

        // open test database
        let db_path = Path::new("test");
        let db = Db::open(db_path).unwrap();

        // check number of fields
        let num_fields = db.num_fields();
        println!("Num fields: {}", num_fields);
        assert_eq!(num_fields, 1 + field_names.len() as u64);

        // check field names are correct
        let db_fields = db.fields();
        assert!(db_fields.contains_key("field1"));
        assert!(db_fields.contains_key("field2"));
        assert_eq!(None, db_fields.get("missing"));

        // check number of trails
        let num_trails = db.num_trails();
        println!("Num trails: {}", num_trails);
        assert_eq!(num_trails, trail_cnt);

        // check number of events
        let num_events = db.num_events();
        println!("Num events: {}", num_events);
        assert_eq!(num_events, event_cnt);

        // Check round-trip get_uuid/get_trail_id
        for uuid in &uuids {
            let trail_id = db.get_trail_id(&uuid).unwrap();
            let uuid_rt = db.get_uuid(trail_id).unwrap();
            assert_eq!(&uuid, &uuid_rt);
        }

        // check max/min timestamp
        let min_timestamp = *timestamps.iter().min().unwrap();
        let max_timestamp = *timestamps.iter().max().unwrap();
        println!("Mix/Max timestamp: {}/{}", min_timestamp, max_timestamp);
        assert_eq!(db.min_timestamp(), min_timestamp);
        assert_eq!(db.max_timestamp(), max_timestamp);

        // test cursor
        let mut cursor = db.cursor();
        for uuid in &uuids {
            let trail_id = db.get_trail_id(&uuid).unwrap();
            cursor.get_trail(trail_id).unwrap();
            assert_eq!(events_per_trail, cursor.len());
        }

        // test db iterator
        for trail in db.iter() {
            // test trail iterator
            for event in trail {
                // check that inserted event values match read values
                for (item, item_ref) in event.items.into_iter().zip(field_vals.iter()) {
                    let item = db.get_item_value(*item);
                    assert_eq!(item, *item_ref);
                }
            }
        }
    }
}
