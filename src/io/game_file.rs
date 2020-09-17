//! Abstraction over the game file.
//!
//! See [`GameFile`] for details

// Modules
#[cfg(test)]
mod test;

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
///
/// # Seek
/// All seeks are done in data addresses, the stream position is also in data addresses.
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
	///
	/// This seeks the reader to the start of the data section on the first sector
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

		// Current address
		let mut cur_address = RealAddress::from_u64(self.reader.stream_position()?);

		// While the buffer isn't empty
		while !buf.is_empty() {
			// If we're at the end of the current data section, seek to the start of the next data data section and restart
			if cur_address == cur_address.cur_sector_data_section_end() {
				cur_address = RealAddress::from_u64(
					self.reader.seek(SeekFrom::Start(
						cur_address
							.next_sector_data_section_start()
							.try_into()
							.expect("Sector offset didn't fit into `i64`"),
					))?,
				);
				continue;
			}

			// Get how many bytes we can read, The minimum between the end of the data section and the size of the buffer
			// Note: Can't overflow, max is `2048`
			// Note: At this point, `cur_address` must be within the data section
			let bytes_to_read = cur_address
				.try_to_data()
				.expect("Address wasn't in data section")
				.remaining_bytes()
				.min(buf.len().try_into().expect("Unable to convert `usize` to `u64`"))
				.try_into()
				.expect("Unable to convert number 0..2048 to `i64`");

			// Read either until the end of the data section or until buffer is full
			// Note: If any fail, we immediately return Err
			let bytes_read = self.reader.read(&mut buf[0..bytes_to_read])?;

			// If 0 bytes were read, EOF was reached, so return with however many we've read
			if bytes_read == 0 {
				return Ok(total_buf_len - buf.len());
			}

			// Else seek into the next data section start
			cur_address = RealAddress::from_u64(self.reader.seek(SeekFrom::Start(cur_address.next_sector_data_section_start().as_u64()))?);

			// And discard what we've already read
			// Note: This slice can't panic, as `bytes_read` is at most `buf.len()`
			buf = &mut buf[bytes_read..];
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

		// Current address
		let mut cur_address = RealAddress::from_u64(self.reader.stream_position()?);

		// While the buffer isn't empty
		while !buf.is_empty() {
			// If we're at the end of the current data section, seek to the start of the next data data section and restart
			if cur_address == cur_address.cur_sector_data_section_end() {
				cur_address = RealAddress::from_u64(
					self.reader.seek(SeekFrom::Start(
						cur_address
							.next_sector_data_section_start()
							.try_into()
							.expect("Sector offset didn't fit into `i64`"),
					))?,
				);
				continue;
			}

			// Get how many bytes we can write, The minimum between the end of the data section and the size of the buffer
			// Note: Can't overflow, max is `2048`
			// Note: At this point, `cur_address` must be within the data section
			let bytes_to_write = cur_address
				.try_to_data()
				.expect("Address wasn't in data section")
				.remaining_bytes()
				.min(buf.len().try_into().expect("Unable to convert `usize` to `u64`"))
				.try_into()
				.expect("Unable to convert number 0..2048 to `i64`");

			// Write either until the end of the data section or until buffer is full
			// Note: If any fail, we immediately return Err
			let bytes_written = self.reader.write(&buf[0..bytes_to_write])?;

			// If 0 bytes were read, EOF was reached, so return with however many we've read
			if bytes_written == 0 {
				return Ok(total_buf_len - buf.len());
			}

			// Else seek into the next data section start
			cur_address = RealAddress::from_u64(self.reader.seek(SeekFrom::Start(cur_address.next_sector_data_section_start().as_u64()))?);

			// And discard what we've already written
			// Note: This slice can't panic, as `bytes_written` is at most `buf.len()`
			buf = &buf[bytes_written..];
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
	fn seek(&mut self, data_seek: SeekFrom) -> std::io::Result<u64> {
		// Imports
		use std::ops::Add;

		// Calculate the real seek
		let real_seek = match data_seek {
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
		let cur_real_address = RealAddress::from(self.reader.seek(real_seek)?);

		// Get the data address
		let data_address = cur_real_address
			.try_to_data()
			.map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, SeekNonDataError(err)))?;

		// And return the new data address
		Ok(data_address.as_u64())
	}
}
