#[allow(non_camel_case_types, dead_code, non_snake_case, private_in_public)]
extern crate traildb_sys;

use std::ffi::CString;
use std::fmt;
use std::mem::{forget, transmute};
use std::path::Path;

use std::collections::HashMap;

#[derive(Debug, PartialEq)]
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
fn wrap_tdb_err<T>(err: traildb_sys::tdb_error, val: T) -> Result<T, Error> {
    match err {
        traildb_sys::tdb_error_TDB_ERR_OK => Ok(val),
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Item(pub u64);
/// TODO: Document me
pub type Value = u64;
/// TODO: Document me
pub type Field = u32;

/// A structure that represents a `TrailDB` constructor.
///
/// A constructor lives in RAM. All events are added to the constructor.
/// After being written to disk, the `TrailDB` is immutable.
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
    obj: *mut traildb_sys::tdb_cons,
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

        let ptr = unsafe { traildb_sys::tdb_cons_init() };
        let ret = unsafe {
            traildb_sys::tdb_cons_open(
                ptr,
                path_cstr(path).as_ptr(),
                field_ptrs.as_slice().as_ptr() as *mut *const i8,
                field_ptrs.len() as u64,
            )
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
            traildb_sys::tdb_cons_add(
                self.obj,
                uuid.as_ptr() as *mut u8,
                timestamp,
                val_ptrs.as_slice().as_ptr() as *mut *const i8,
                val_lens.as_slice().as_ptr() as *const u64,
            )
        };
        wrap_tdb_err(ret, ())
    }

    /// Close a constructor without writing it to disk.
    pub fn close(&mut self) {
        unsafe { traildb_sys::tdb_cons_close(self.obj) };
    }

    /// Write the TrailDB to disk and close it.
    pub fn finalize(&mut self) -> Result<(), Error> {
        let ret = unsafe { traildb_sys::tdb_cons_finalize(self.obj) };
        wrap_tdb_err(ret, ())
    }

    /// Combine an already finalized TrailDB with a constructor.
    pub fn append(&mut self, db: &Db) -> Result<(), Error> {
        let ret = unsafe { traildb_sys::tdb_cons_append(self.obj, transmute(db)) };
        wrap_tdb_err(ret, ())
    }
}

impl Drop for Constructor {
    fn drop(&mut self) {
        unsafe { traildb_sys::tdb_cons_close(self.obj) };
    }
}

pub struct Db<'a> {
    obj: &'a mut traildb_sys::tdb,
}

impl<'a> Db<'a> {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let ptr = unsafe { traildb_sys::tdb_init() };
        let ret = unsafe { traildb_sys::tdb_open(ptr, path_cstr(path).as_ptr()) };
        unsafe {
            wrap_tdb_err(
                ret,
                Db {
                    obj: transmute(ptr),
                },
            )
        }
    }

    pub fn close(&mut self) {
        unsafe {
            traildb_sys::tdb_close(self.obj);
        }
    }

    pub fn num_trails(&self) -> u64 {
        unsafe { traildb_sys::tdb_num_trails(self.obj) }
    }

    pub fn num_events(&self) -> u64 {
        unsafe { traildb_sys::tdb_num_events(self.obj) }
    }

    pub fn num_fields(&self) -> u64 {
        unsafe { traildb_sys::tdb_num_fields(self.obj) }
    }

    pub fn min_timestamp(&self) -> Timestamp {
        unsafe { traildb_sys::tdb_min_timestamp(self.obj) }
    }

    pub fn max_timestamp(&self) -> Timestamp {
        unsafe { traildb_sys::tdb_max_timestamp(self.obj) }
    }

    pub fn version(&self) -> Version {
        unsafe { traildb_sys::tdb_version(self.obj) }
    }

    pub fn will_need(&self) {
        unsafe { traildb_sys::tdb_willneed(self.obj) };
    }

    pub fn dont_need(&self) {
        unsafe { traildb_sys::tdb_dontneed(self.obj) };
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
            traildb_sys::tdb_get_trail_id(
                self.obj,
                uuid.as_ptr() as *mut u8,
                &mut id as *mut TrailId,
            )
        };
        match ret {
            traildb_sys::tdb_error_TDB_ERR_OK => Some(id),
            _ => None,
        }
    }

    pub fn get_uuid(&self, trail_id: TrailId) -> Option<&Uuid> {
        unsafe {
            let ptr = traildb_sys::tdb_get_uuid(self.obj, trail_id) as *const [u8; 16];
            ptr.as_ref()
        }
    }

    pub fn cursor(&'a self) -> Cursor<'a> {
        unsafe {
            let ptr = traildb_sys::tdb_cursor_new(self.obj);
            Cursor {
                obj: transmute(ptr),
            }
        }
    }

    pub fn iter(&'a self) -> DbIter<'a> {
        DbIter { pos: 0, db: self }
    }

    pub fn get_item_value(&'a self, item: Item) -> Option<&'a str> {
        unsafe {
            let mut len = 0u64;
            let ptr = traildb_sys::tdb_get_item_value(self.obj, transmute(item), &mut len);
            if len > 0 {
                let s = std::slice::from_raw_parts(ptr as *const u8, len as usize);
                Some(std::str::from_utf8_unchecked(s))
            } else {
                None
            }
        }
    }

    pub fn get_item(&'a self, field: Field, value: &str) -> Option<Item> {
        unsafe {
            let item = traildb_sys::tdb_get_item(
                self.obj,
                transmute(field),
                value.as_ptr() as *const i8,
                value.len() as u64,
            );

            if item == 0 {
                None
            } else {
                Some(Item(item))
            }
        }
    }

    pub fn get_field_name(&'a self, field: Field) -> Option<&'a str> {
        unsafe {
            let ptr = traildb_sys::tdb_get_field_name(self.obj, field);
            match std::ffi::CStr::from_ptr(ptr).to_str() {
                Ok(s) => Some(s),
                Err(_) => None,
            }
        }
    }

    pub fn lexicon_size(&'a self, field: Field) -> u64 {
        unsafe { traildb_sys::tdb_lexicon_size(self.obj, field) }
    }

    pub fn lexicon(&'a self, field: Field) -> Vec<&'a str> {
        let mut vec = Vec::with_capacity(self.lexicon_size(field) as usize);
        for i in 1..self.lexicon_size(field) {
            let value = unsafe {
                let mut len = 0u64;
                let ptr = traildb_sys::tdb_get_value(self.obj, field, i, &mut len);
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
        unsafe { traildb_sys::tdb_close(self.obj) };
    }
}

pub struct DbIter<'a> {
    pos: u64,
    db: &'a Db<'a>,
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

/// A cursor allows you to iterate over the events in a single trail,
/// decoding a batch from the optimized storage format when necessary.
///
/// A cursor can be re-used across trails, by calling
/// `cursor.get_trail()` with the `TrailId`. Initializing a cursor is
/// expensive and using `DbIter` which initializes a new cursor for
/// each trail will be much slower, than re-using a cursor.
pub struct Cursor<'a> {
    obj: &'a mut traildb_sys::tdb_cursor,
}

impl<'a> Cursor<'a> {
    pub fn get_trail(&mut self, trail_id: TrailId) -> Result<(), Error> {
        let ret = unsafe { traildb_sys::tdb_get_trail(self.obj, trail_id) };
        wrap_tdb_err(ret, ())
    }

    pub fn len(&mut self) -> u64 {
        unsafe { traildb_sys::tdb_get_trail_length(self.obj) }
    }

    pub fn set_filter(&mut self, filter: &EventFilter) -> Result<(), Error> {
        let ret = unsafe { traildb_sys::tdb_cursor_set_event_filter(self.obj, filter.obj) };
        wrap_tdb_err(ret, ())
    }

    pub fn unset_filter(&mut self) {
        unsafe { traildb_sys::tdb_cursor_unset_event_filter(self.obj) };
    }
}

impl<'a> Drop for Cursor<'a> {
    fn drop(&mut self) {
        unsafe { traildb_sys::tdb_cursor_free(self.obj) };
    }
}

impl<'a> Iterator for Cursor<'a> {
    type Item = Event<'a>;

    fn next(&mut self) -> Option<Event<'a>> {
        unsafe {
            let e = traildb_sys::tdb_cursor_next(self.obj);
            Event::from_tdb_event(e)
        }
    }
}

/// A `MultiCursor` allows you to iterate over multiple cursors at the
/// same time, even from different TrailDBs.
pub struct MultiCursor<'a> {
    obj: &'a mut traildb_sys::tdb_multi_cursor,
}

impl<'a> MultiCursor<'a> {
    /// Open a cursor in each of the Dbs passed. If you want multiple
    /// cursors for the same db, include it multiple times.
    pub fn new(cursors: &'a [Cursor<'a>]) -> MultiCursor<'a> {
        use std::iter::FromIterator;

        Self::from_iter(cursors)
    }

    unsafe fn from_raw(ptrs: &[*const traildb_sys::tdb_cursor]) -> Self {
        let ptr = traildb_sys::tdb_multi_cursor_new(
            ptrs.as_ptr() as *mut *mut traildb_sys::tdb_cursor,
            ptrs.len() as u64,
        );
        MultiCursor {
            obj: transmute(ptr),
        }
    }

    pub fn reset(&mut self) {
        unsafe { traildb_sys::tdb_multi_cursor_reset(self.obj) };
    }
}

impl<'a> std::iter::FromIterator<&'a Cursor<'a>> for MultiCursor<'a> {
    fn from_iter<I: IntoIterator<Item = &'a Cursor<'a>>>(iter: I) -> Self {
        let mut ptrs: Vec<*const traildb_sys::tdb_cursor> = vec![];
        for cursor in iter.into_iter() {
            let ptr: *const traildb_sys::tdb_cursor = cursor.obj;
            ptrs.push(ptr);
        }
        unsafe { Self::from_raw(&ptrs) }
    }
}

impl<'a> Drop for MultiCursor<'a> {
    fn drop(&mut self) {
        unsafe { traildb_sys::tdb_multi_cursor_free(self.obj) };
    }
}

impl<'a> Iterator for MultiCursor<'a> {
    type Item = MultiEvent<'a>;

    fn next(&mut self) -> Option<MultiEvent<'a>> {
        unsafe {
            let e = traildb_sys::tdb_multi_cursor_next(self.obj);
            MultiEvent::from_tdb_multi_event(e)
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

fn path_cstr<P: AsRef<Path>>(path: P) -> CString {
    CString::new(path.as_ref().to_str().unwrap()).unwrap()
}

#[derive(Debug)]
pub struct Event<'a> {
    pub timestamp: Timestamp,
    pub items: &'a [Item],
}

impl<'a> Event<'a> {
    fn from_tdb_event(e: *const traildb_sys::tdb_event) -> Option<Self> {
        unsafe {
            match e.as_ref() {
                None => None,
                Some(e) => Some(Event {
                    timestamp: e.timestamp,
                    items: std::slice::from_raw_parts(transmute(&e.items), e.num_items as usize),
                }),
            }
        }
    }
}

#[derive(Debug)]
pub struct MultiEvent<'a> {
    pub cursor_idx: usize,
    pub event: Event<'a>,
}

impl<'a> MultiEvent<'a> {
    fn from_tdb_multi_event(e: *const traildb_sys::tdb_multi_event) -> Option<Self> {
        unsafe {
            match e.as_ref() {
                None => None,
                Some(multi_event) => Some(MultiEvent {
                    event: Event::from_tdb_event(multi_event.event).unwrap(),
                    cursor_idx: multi_event.cursor_idx as usize,
                }),
            }
        }
    }
}

pub struct EventFilter<'b> {
    obj: &'b mut traildb_sys::tdb_event_filter,
}

impl<'b> EventFilter<'b> {
    pub fn new() -> EventFilter<'b> {
        let filter = unsafe { traildb_sys::tdb_event_filter_new() };
        EventFilter {
            obj: unsafe { transmute(filter) },
        }
    }

    pub fn all() -> EventFilter<'b> {
        let filter = unsafe { traildb_sys::tdb_event_filter_new_match_all() };
        EventFilter {
            obj: unsafe { transmute(filter) },
        }
    }

    pub fn none() -> EventFilter<'b> {
        let filter = unsafe { traildb_sys::tdb_event_filter_new_match_none() };
        EventFilter {
            obj: unsafe { transmute(filter) },
        }
    }

    pub fn or(&mut self, item: Item) -> &mut EventFilter<'b> {
        unsafe {
            traildb_sys::tdb_event_filter_add_term(self.obj, item.0, false as i32);
        };
        self
    }

    pub fn or_not(&mut self, item: Item) -> &mut EventFilter<'b> {
        unsafe {
            traildb_sys::tdb_event_filter_add_term(self.obj, item.0, true as i32);
        };
        self
    }

    pub fn and(&mut self) -> &mut EventFilter<'b> {
        let ret = wrap_tdb_err(
            unsafe { traildb_sys::tdb_event_filter_new_clause(self.obj) },
            (),
        );
        ret.expect("tdb_event_filter_new_clause failed");
        self
    }

    pub fn time_range(&mut self, start: u64, end: u64) -> &mut EventFilter<'b> {
        let ret = wrap_tdb_err(
            unsafe { traildb_sys::tdb_event_filter_add_time_range(self.obj, start, end) },
            (),
        );
        ret.expect("tdb_event_filter_add_time_range failed");
        self
    }

    pub fn num_clauses(&mut self) -> u64 {
        unsafe { traildb_sys::tdb_event_filter_num_clauses(self.obj) }
    }
}

impl<'b> Drop for EventFilter<'b> {
    fn drop(&mut self) {
        unsafe { traildb_sys::tdb_event_filter_free(self.obj) };
    }
}

#[cfg(test)]
mod tests {
    extern crate tempdir;
    extern crate uuid;
    use self::tempdir::TempDir;
    use super::{Constructor, Cursor, Db, EventFilter, MultiCursor, MultiEvent};
    use std::collections::HashSet;
    use std::iter::FromIterator;

    #[test]
    #[no_mangle]
    fn test_traildb() {
        // create a new constructor
        let field_names = ["field1", "field2"];

        let mut path = TempDir::new("traildb-tmp").unwrap().into_path();
        path.push("kitchen-sink-test");
        let mut cons = Constructor::new(&path, &field_names).unwrap();
        let field_vals = ["cats", "dogs"];

        // add an event
        let events_per_trail = 10;
        let mut trail_cnt = 0;
        let mut event_cnt = 0;
        let mut uuids = Vec::new();
        let mut timestamp = 1;
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
        let db = Db::open(&path).unwrap();

        // check number of fields
        let num_fields = db.num_fields();
        assert_eq!(num_fields, 1 + field_names.len() as u64);

        // check field names are correct
        let db_fields = db.fields();
        assert!(db_fields.contains_key("field1"));
        assert!(db_fields.contains_key("field2"));
        assert_eq!(None, db_fields.get("missing"));

        // check number of trails
        let num_trails = db.num_trails();
        assert_eq!(num_trails, trail_cnt);

        // check number of events
        let num_events = db.num_events();
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
                    assert!(item.is_some());
                    assert_eq!(item.unwrap(), *item_ref);
                }
            }
        }
    }

    #[test]
    fn test_multi_cursor() {
        let field_names = ["field1"];
        let mut path = TempDir::new("traildb-tmp").unwrap().into_path();
        path.push("multicursor-test");

        let mut cons = Constructor::new(&path, &field_names).unwrap();

        let uuid1 = *uuid::Uuid::new_v4().as_bytes();
        let uuid2 = *uuid::Uuid::new_v4().as_bytes();

        assert!(cons.add(&uuid1, 10, &["foo1"]).is_ok());
        assert!(cons.add(&uuid1, 11, &["foo2"]).is_ok());
        assert!(cons.add(&uuid1, 12, &["foo3"]).is_ok());

        assert!(cons.add(&uuid2, 20, &["bar1"]).is_ok());
        assert!(cons.add(&uuid2, 21, &["bar2"]).is_ok());
        assert!(cons.add(&uuid2, 22, &["bar3"]).is_ok());

        assert!(cons.finalize().is_ok());

        let db = Db::open(&path).unwrap();

        let mut cursors = vec![db.cursor(), db.cursor()];
        assert!(cursors[0].get_trail(0).is_ok());
        assert!(cursors[1].get_trail(1).is_ok());

        let mut multi_cursor: MultiCursor = cursors.iter().collect();
        multi_cursor.reset();

        let multi_events: Vec<MultiEvent> = multi_cursor.collect();
        assert_eq!(
            vec![10, 11, 12, 20, 21, 22],
            multi_events
                .iter()
                .map(|me| me.event.timestamp)
                .collect::<Vec<u64>>()
        );
        assert_eq!(
            HashSet::from_iter(vec![0, 0, 0, 1, 1, 1].into_iter()),
            multi_events
                .iter()
                .map(|me| me.cursor_idx)
                .collect::<HashSet<usize>>()
        );

        // TODO: Test dropping cursors, should not be allowed if multi
        // cursor is still around.
    }

    #[test]
    fn filters() {
        let mut path = TempDir::new("traildb-tmp").unwrap().into_path();
        path.push("filters");

        let mut cons = Constructor::new(&path, &vec!["field1", "field2"]).unwrap();

        let uuid = *uuid::Uuid::new_v4().as_bytes();

        assert!(cons.add(&uuid, 0, &vec!["a", "0"]).is_ok());
        assert!(cons.add(&uuid, 1, &vec!["b", "1"]).is_ok());
        assert!(cons.add(&uuid, 2, &vec!["a", "2"]).is_ok());
        assert!(cons.add(&uuid, 3, &vec!["c", "3"]).is_ok());
        assert!(cons.add(&uuid, 4, &vec!["a", "4"]).is_ok());
        assert!(cons.add(&uuid, 5, &vec!["d", "5"]).is_ok());
        assert!(cons.finalize().is_ok());

        // Return a Vec with timestamps of event returned when the
        // given filter is applied.
        fn timestamps(c: &mut Cursor, f: &EventFilter) -> Vec<u64> {
            assert!(c.get_trail(0).is_ok());
            assert!(c.set_filter(f).is_ok());

            let mut result = vec![];
            for event in c {
                result.push(event.timestamp);
            }
            result
        }

        let db = Db::open(&path).unwrap();
        let fields = db.fields();
        let field1 = fields.get("field1").unwrap();
        let field2 = fields.get("field2").unwrap();

        let mut cursor = db.cursor();

        let mut f = EventFilter::new();

        // Empty filter doesn't match any events
        assert_eq!(1, f.num_clauses());
        assert_eq!(0, timestamps(&mut cursor, &f).len());

        // Events with field1=a
        f.or(db.get_item(*field1, "a").unwrap());
        assert_eq!(vec![0, 2, 4], timestamps(&mut cursor, &f));

        // Calling '.or(...)' again adds another OR clause
        // the filter is now: field1=a OR field1=b
        f.or(db.get_item(*field1, "b").unwrap());
        assert_eq!(vec![0, 1, 2, 4], timestamps(&mut cursor, &f));

        drop(f);

        // field1=a OR field1=b with more ergonomic API
        let mut f = EventFilter::new();
        f.or(db.get_item(*field1, "a").unwrap())
            .or(db.get_item(*field1, "b").unwrap());
        assert_eq!(vec![0, 1, 2, 4], timestamps(&mut cursor, &f));
        drop(f);

        // NOT field1=a
        let mut f = EventFilter::new();
        f.or(db.get_item(*field1, "a").unwrap());
        assert_eq!(vec![0, 2, 4], timestamps(&mut cursor, &f));
        drop(f);

        // field1=a AND (field2=0 OR field2=2)
        let mut f = EventFilter::new();
        f.or(db.get_item(*field1, "a").unwrap())
            .and()
            .or(db.get_item(*field2, "0").unwrap())
            .or(db.get_item(*field2, "2").unwrap());
        assert_eq!(vec![0, 2], timestamps(&mut cursor, &f));
        drop(f);

        // start_time <= timestamp < end_time
        let mut f = EventFilter::new();
        f.time_range(2, 4);
        assert_eq!(vec![2, 3], timestamps(&mut cursor, &f));
        drop(f);

        // EventFilter::all() always matches all events.
        let mut f = EventFilter::all();
        assert_eq!(6, timestamps(&mut cursor, &f).len());
        // TODO: Adding a term to a match_all filter doesn't make
        // sense. Should it fail?
        f.or(db.get_item(*field1, "a").unwrap());
        assert_eq!(6, timestamps(&mut cursor, &f).len());
        drop(f);

        // EventFilter::none() always matches all events.
        let f = EventFilter::none();
        assert_eq!(0, timestamps(&mut cursor, &f).len());
    }
}
