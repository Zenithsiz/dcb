//! Tests

use crate::FilePtr;

// Imports
use super::*;
use chrono::NaiveDateTime;
use zutil::AsciiStrArr;

#[test]
fn write_read_entries() {
	let mut buffer = io::Cursor::new(vec![0; 0x16 * 2]);

	let dir = DirPtr::root();

	let entries = vec![
		DirEntry {
			name: AsciiStrArr::from_bytes("dir-1").expect("Invalid string"),
			date: NaiveDateTime::from_timestamp(123, 0),
			kind: DirEntryKind::Dir { ptr: DirPtr::new(123) },
		},
		DirEntry {
			name: AsciiStrArr::from_bytes("file-1").expect("Invalid string"),
			date: NaiveDateTime::from_timestamp(123, 0),
			kind: DirEntryKind::File {
				ptr:       FilePtr::new(123, 456),
				extension: AsciiStrArr::from_bytes("ext").expect("Invalid string"),
			},
		},
	];

	dir.write_entries(&mut buffer, entries.iter().cloned())
		.expect("Unable to write entries");

	let read_entries = dir
		.read_entries(&mut buffer)
		.expect("Unable to read entries")
		.collect::<Result<Vec<_>, _>>()
		.expect("Unable to read all entries");

	assert_eq!(entries, read_entries);
}
