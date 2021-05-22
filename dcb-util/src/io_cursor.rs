//! Io cursor

// Imports
use crate::WriteTake;
use std::{
	convert::{TryFrom, TryInto},
	io::{self, Read, Seek, SeekFrom, Write},
};

/// Io cursor.
///
/// 'Slices' a piece of an inner cursor and only allows access
/// to it.
#[derive(Debug)]
pub struct IoCursor<T> {
	/// Inner value
	inner: T,

	/// Start position
	start_pos: u64,

	/// Size
	size: u64,
}

impl<T: Seek> IoCursor<T> {
	/// Creates a new cursor given it's starting position and size
	pub fn new(mut inner: T, start_pos: u64, size: u64) -> Result<Self, io::Error> {
		// Seek to the start
		inner.seek(SeekFrom::Start(start_pos))?;

		Ok(Self { inner, start_pos, size })
	}

	/// Returns the current position of the cursor
	pub fn cur_pos(&mut self) -> Result<u64, io::Error> {
		let inner_pos = self.inner.stream_position()?;

		Ok(inner_pos - self.start_pos)
	}
}

impl<T: Read + Seek> Read for IoCursor<T> {
	fn read(&mut self, buf: &mut [u8]) -> Result<usize, io::Error> {
		let len = u64::min(
			buf.len().try_into().expect("Buffer length didn't fit into a `u64`"),
			self.size - self.cur_pos()?,
		);

		self.inner.by_ref().take(len).read(buf)
	}
}

impl<T: Write + Seek> Write for IoCursor<T> {
	fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
		let len = u64::min(
			buf.len().try_into().expect("Buffer length didn't fit into a `u64`"),
			self.size - self.cur_pos()?,
		);

		WriteTake::new(&mut self.inner, len).write(buf)
	}

	fn flush(&mut self) -> io::Result<()> {
		self.inner.flush()
	}
}

impl<T: Read + Seek> Seek for IoCursor<T> {
	fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
		match pos {
			SeekFrom::Start(pos) => {
				let pos = u64::min(pos, self.size);
				let inner_pos = self.inner.seek(SeekFrom::Start(self.start_pos + pos))?;
				Ok(inner_pos - self.start_pos)
			},
			SeekFrom::End(pos) => {
				let pos = pos.clamp(-i64::try_from(self.size).expect("Size didn't fit into an `i64`"), 0);
				let inner_pos = self
					.inner
					.seek(SeekFrom::Start(crate::signed_offset(self.start_pos + self.size, pos)))?;
				Ok(inner_pos - self.start_pos)
			},
			SeekFrom::Current(offset) => {
				let offset = match offset.is_positive() {
					// If it's positive, check how much we have until the end
					true => {
						let until_end = i64::try_from(self.size - self.cur_pos()?)
							.expect("Remaining size didn't fit into an `i64`");
						i64::min(until_end, offset)
					},

					// Else it's negative, check how much we have until the start
					false => {
						let until_start =
							-i64::try_from(self.cur_pos()?).expect("Remaining size didn't fit into an `i64`");
						i64::max(until_start, offset)
					},
				};

				let inner_pos = self.inner.seek(SeekFrom::Current(offset))?;
				Ok(inner_pos - self.start_pos)
			},
		}
	}
}
