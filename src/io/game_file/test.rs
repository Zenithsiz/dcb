//! Tests

// Imports
use super::*;
use std::{convert::TryFrom, io::Cursor};

#[test]
fn seek_start() {
	let buf = vec![0; 10_000];
	let cursor = Cursor::new(buf);
	let mut game_file = GameFile::from_reader(cursor).expect("Unable to create game file");

	// Check initial position
	assert_eq!(game_file.stream_position().expect("Unable to get stream position"), 0);

	// Check other seeks
	for &seek_pos in &[1, 2047, 2048, 2049, 4095, 4096, 4097] {
		game_file.seek(SeekFrom::Start(seek_pos)).expect("Unable to seek");

		assert_eq!(game_file.stream_position().expect("Unable to get stream position"), seek_pos);
	}
}

#[test]
fn seek_cur() {
	let buf = vec![0; 10_000];
	let cursor = Cursor::new(buf);
	let mut game_file = GameFile::from_reader(cursor).expect("Unable to create game file");

	// Check initial position
	assert_eq!(game_file.stream_position().expect("Unable to get stream position"), 0);

	// Check other seeks
	let mut cur_pos = 0u64;
	for &seek_offset in &[0, 1, 2046, 1, 0, 1, 2048, 2047, 2049] {
		game_file
			.seek(SeekFrom::Current(i64::try_from(seek_offset).expect("Unable to convert to `i64`")))
			.expect("Unable to seek");
		cur_pos += seek_offset;

		assert_eq!(game_file.stream_position().expect("Unable to get stream position"), cur_pos);
	}
}

// TODO: Read and write tests.
