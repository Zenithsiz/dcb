//! Tests

// Lints
// TODO: Deal with these conversions
#![allow(clippy::as_conversions, clippy::cast_possible_truncation)]

// Imports
use super::*;
use itertools::Itertools;
use std::{convert::TryFrom, io::Cursor};

/// Buffer size for all tests
/// Equal to 5 sectors
pub const BUF_SIZE: u64 = RealAddress::SECTOR_BYTE_SIZE * 5;

#[test]
fn seek_start() {
	let buf = vec![0; BUF_SIZE as usize];
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
	let buf = vec![0; BUF_SIZE as usize];
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

#[test]
fn read() {
	let buf: Vec<_> = (0..255).cycle().take(BUF_SIZE as usize).collect();
	let cursor = Cursor::new(buf);
	let mut game_file = GameFile::from_reader(cursor).expect("Unable to create game file");

	let read_expected: Vec<_> = (0..255)
		.cycle()
		.chunks(RealAddress::SECTOR_BYTE_SIZE as usize)
		.into_iter()
		.flat_map(|chunk| {
			chunk
				.skip(RealAddress::HEADER_BYTE_SIZE as usize)
				.take(RealAddress::DATA_BYTE_SIZE as usize)
		})
		.take(data_bytes(BUF_SIZE) as usize)
		.collect();

	let mut read = Vec::with_capacity(read_expected.len());
	assert_eq!(game_file.read_to_end(&mut read).expect("Unable to read"), read_expected.len());
	assert_eq!(read.len(), read_expected.len());

	// Note: We use `zip(0..)` instead of `enumerate` because we need a `u64`.
	for ((lhs, rhs), n) in read.into_iter().zip(read_expected.into_iter()).zip(0..) {
		if lhs != rhs {
			panic!("Data differed at data address {}, ({lhs} != {rhs})", DataAddress::from_u64(n));
		}
	}
}

#[test]
fn write() {
	let buf: Vec<_> = vec![0; BUF_SIZE as usize];
	let cursor = Cursor::new(buf);
	let mut game_file = GameFile::from_reader(cursor).expect("Unable to create game file");

	let written_expected: Vec<_> = (0..255).cycle().take(data_bytes(BUF_SIZE) as usize).collect();

	assert_eq!(game_file.write(&written_expected).expect("Unable to write"), written_expected.len());
	let written: Vec<_> = game_file
		.reader
		.into_inner()
		.into_iter()
		.chunks(RealAddress::SECTOR_BYTE_SIZE as usize)
		.into_iter()
		.flat_map(|chunk| {
			chunk
				.skip(RealAddress::HEADER_BYTE_SIZE as usize)
				.take(RealAddress::DATA_BYTE_SIZE as usize)
		})
		.collect();
	assert_eq!(written.len(), written_expected.len());

	// Note: We use `zip(0..)` instead of `enumerate` because we need a `u64`.
	for ((lhs, rhs), n) in written_expected.into_iter().zip(written.into_iter()).zip(0..) {
		if lhs != rhs {
			panic!("Data differed at data address {}, ({lhs} != {rhs})", DataAddress::from_u64(n));
		}
	}
}


/// Returns the number of data bytes within a number of real bytes
fn data_bytes(real_size: u64) -> u64 {
	let data_sectors = real_size / RealAddress::SECTOR_BYTE_SIZE;

	let data_offset = match real_size % RealAddress::SECTOR_BYTE_SIZE {
		0..RealAddress::DATA_START => 0,
		offset @ RealAddress::DATA_START..RealAddress::DATA_END => offset,
		RealAddress::DATA_END..RealAddress::SECTOR_BYTE_SIZE => 0,
		offset => unreachable!("Offset was {}", offset),
	};

	data_sectors * RealAddress::DATA_BYTE_SIZE + data_offset
}
