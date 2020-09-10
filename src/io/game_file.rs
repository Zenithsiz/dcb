//! Abstraction over the game file.
//!
//! See [`GameFile`] for details

// Imports
use crate::io::address::{real, Data as DataAddress, Real as RealAddress};
use std::{
	convert::TryInto,
	io::{Read, Seek, SeekFrom, Write},
};

/// A type that abstracts over a the game reader.
///
/// # Game reader
/// The game file is a `.bin` file, of the type `MODE2/2352`.
///
/// This means that the file is divided into sectors of size
/// 2352 bytes, each with it's data structure.
///
/// For us the only thing that matters is the data section
/// of each sector, which is 2048 bytes long.
///
/// This type allows reading and writing in `DataAddress` addresses,
/// which are reader offsets in terms of the 2048 byte data section,
/// instead of the 2352 byte sectors.
///
/// # Parameters
/// `GameFile` is generic over `R`, this being any type that implements
/// `Read`, `Write` and `Seek`, thus being able to read from either a
/// reader, a buffer in memory or even some remote network location.
///
/// # Read/Write Strategy
/// The strategy this employs for reading and writing currently is to
/// get the current 2048 byte block and work on it until it is exhausted,
/// then to get a new 2048 byte block until the operation is complete.
/// This will require an `io` call for every single 2048 byte block instead
/// of an unique call for all of the block, but due to the invariants required,
/// this is the strategy employed.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Default, Hash, Debug)]
pub struct GameFile<R: Read + Write + Seek> {
	/// The type to read and write from
	reader: R,
}

/// Error type for [`GameFile::from_reader`]
#[derive(Debug, thiserror::Error)]
pub enum NewGameFileError {
	/// Unable to seek reader to data section
	#[error("Unable to seek reader to data section")]
	SeekData(#[source] std::io::Error),
}

// Constructors
impl<R: Read + Write + Seek> GameFile<R> {
	/// Constructs a `GameFile` given a reader
	pub fn from_reader(mut reader: R) -> Result<Self, NewGameFileError> {
		// Seek the reader to the beginning of the data section
		reader
			.seek(SeekFrom::Start(DataAddress::from_u64(0).to_real().as_u64()))
			.map_err(NewGameFileError::SeekData)?;

		Ok(Self { reader })
	}
}

/// `Read` for `GameFile`
///
/// # Implementation guarantees
/// Currently `Read` guarantees that if an error is returned, then
/// the buffer isn't modified, but this implementation cannot make
/// that guarantee.
impl<R: Read + Write + Seek> Read for GameFile<R> {
	fn read(&mut self, mut buf: &mut [u8]) -> std::io::Result<usize> {
		// Total length of the buffer to fill
		let total_buf_len = buf.len();

		// While the buffer isn't empty
		while !buf.is_empty() {
			// Get the current real address we're at in the reader
			// Note: If we can't get the position, we return immediately
			let cur_real_address = RealAddress::from(self.reader.stream_position()?);

			// Get the data section end
			let data_section_end = cur_real_address.data_section_end();

			// If we're at the end of the data section, seek to the next data section
			if cur_real_address == data_section_end {
				// Seek ahead by skipping the footer and next header
				self.reader.seek(SeekFrom::Current(
					(RealAddress::FOOTER_BYTE_SIZE + RealAddress::HEADER_BYTE_SIZE)
						.try_into()
						.expect("Sector offset didn't fit into `u64`"),
				))?;

				// And restart this loop
				continue;
			}

			// We always guarantee that the current address lies within the data sections
			// Note: We only check it here, because `cur_real_address` may be `data_section_end`
			//       during seeking.
			assert!(
				cur_real_address.in_data_section(),
				"Real offset {} [Sector {}, Offset {}] could not be read as it was not in the data section",
				cur_real_address,
				cur_real_address.sector(),
				cur_real_address.offset()
			);

			// Check how many bytes we can read
			// Note: Cannot overflow, max is `2048`, so an error means `cur_real_address` was past the data section end
			let mut bytes_to_read = (data_section_end - cur_real_address)
				.try_into()
				.expect("Current address is past end of data");

			// If we were to read more bytes than the buffer has, read less
			if bytes_to_read > buf.len() {
				bytes_to_read = buf.len();
			}

			// Read either until the end of the data section or until buffer is full
			// Note: If any fail, we immediately return Err
			let bytes_read = self.reader.read(&mut buf[0..bytes_to_read])?;

			// If 0 bytes were read, EOF was reached, so return with however many we've read
			if bytes_read == 0 {
				return Ok(total_buf_len - buf.len());
			}

			// Discard what we've already read
			buf = &mut buf[bytes_read..]; // If `bytes_to_read == buf.len()` this does not panic
		}

		// And return the bytes we read
		Ok(total_buf_len)
	}
}

/// Write for `GameFile`
///
/// # Implementation guarantees
/// Currently `Read` guarantees that if an error is returned, then
/// the buffer isn't modified, but this implementation cannot make
/// that guarantee.
impl<R: Read + Write + Seek> Write for GameFile<R> {
	fn write(&mut self, mut buf: &[u8]) -> std::io::Result<usize> {
		// Total length of the buffer to write
		let total_buf_len = buf.len();

		// While the buffer isn't empty
		while !buf.is_empty() {
			// Get the current real address we're at in the reader
			// Note: If we can't get the position, we return immediately
			let cur_real_address = RealAddress::from(self.reader.stream_position()?);

			// Get the data section end
			let data_section_end = cur_real_address.data_section_end();

			// If we're at the end of the data section, seek to the next data section
			if cur_real_address == data_section_end {
				// Seek ahead by skipping the footer and next header
				self.reader.seek(SeekFrom::Current(
					(RealAddress::FOOTER_BYTE_SIZE + RealAddress::HEADER_BYTE_SIZE)
						.try_into()
						.expect("Sector offset didn't fit into `u64`"),
				))?;

				// And restart this loop
				continue;
			}

			// We always guarantee that the current address lies within the data sections
			// Note: We only check it here, because `cur_real_address` may be `data_section_end`
			//       during seeking.
			assert!(
				cur_real_address.in_data_section(),
				"Real offset {} [Sector {}, Offset {}] could not be written as it was not in the data section",
				cur_real_address,
				cur_real_address.sector(),
				cur_real_address.offset()
			);

			// Check how many bytes we can write, up to the buffer's len
			let mut bytes_to_write = (data_section_end - cur_real_address)
				.try_into()
				.expect("Current address is past end of data");

			// If we were to write more bytes than the buffer has, write less
			if bytes_to_write > buf.len() {
				bytes_to_write = buf.len();
			}

			// Write either until the end of the data section or until buffer runs out
			// Note: If this fails, we immediately return Err
			let bytes_written = self.reader.write(&buf[0..bytes_to_write])?;

			// If 0 bytes were written, EOF was reached, so return with however many we've read
			if bytes_written == 0 {
				return Ok(total_buf_len - buf.len());
			}

			// Discard what we've already written
			buf = &buf[bytes_to_write..]; // If `bytes_to_write == buf.len()` this does not panic
		}

		// And return the bytes we read
		Ok(total_buf_len)
	}

	fn flush(&mut self) -> std::io::Result<()> {
		self.reader.flush()
	}
}

/// Error type for `Seek for GameFile`.
/// Returned when, after seeking, we ended up in a non-data section
#[derive(PartialEq, Eq, Clone, Copy, Debug, thiserror::Error)]
#[error("Reader seeked into a non-data section")]
pub struct SeekNonDataError(#[source] real::ToDataError);

impl<R: Read + Write + Seek> Seek for GameFile<R> {
	fn seek(&mut self, data_pos: SeekFrom) -> std::io::Result<u64> {
		// Imports
		use std::ops::Add;

		// Calculate the real position
		let real_pos = match data_pos {
			SeekFrom::Start(data_address) => SeekFrom::Start(
				// Parse the address as data, then convert it to real
				DataAddress::from(data_address).to_real().as_u64(),
			),
			SeekFrom::Current(data_offset) => SeekFrom::Start(
				// Get the real address, convert it to data, add the offset in data units, then convert it back into real
				RealAddress::from(self.reader.stream_position()?)
					.try_to_data()
					.map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, SeekNonDataError(err)))?
					.add(data_offset)
					.to_real()
					.as_u64(),
			),
			SeekFrom::End(_) => {
				todo!("`SeekFrom::End` seeking isn't currently implemented");
			},
		};

		// Seek to the real position and get where we are right now
		let cur_real_address = RealAddress::from(self.reader.seek(real_pos)?);

		// Get the data address
		let data_address = cur_real_address
			.try_to_data()
			.map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, SeekNonDataError(err)))?;

		// And return the new data address
		Ok(data_address.as_u64())
	}
}
