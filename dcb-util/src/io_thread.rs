//! Io thread

// Imports
use crate::MutexPoison;
use std::{
	cell::RefCell,
	convert::{TryFrom, TryInto},
	io::{self, Cursor, SeekFrom},
	mem,
	sync::Mutex,
};
use thread_local::ThreadLocal;

/// Io thread
///
/// This type allows a stream to be read / written to from multiple threads
/// each with their own seek pointer.
pub struct IoThread<T> {
	/// Inner
	inner: Mutex<T>,

	/// Each thread's state
	threads: ThreadLocal<RefCell<ThreadState>>,
}

impl<T> IoThread<T> {
	/// Default buffer size
	const DEFAULT_BUFFER_SIZE: usize = 8192;
	/// Max buffer size
	const MAX_BUFFER_SIZE: usize = 0x8000;

	/// Creates a new io thread
	pub fn new(inner: T) -> Self {
		Self {
			inner:   Mutex::new(inner),
			threads: ThreadLocal::new(),
		}
	}

	/// Returns the thread state associated with this thread
	fn state(&self) -> &RefCell<ThreadState> {
		self.threads.get_or_default()
	}
}

/// Thread state
#[derive(PartialEq, Clone, Default, Debug)]
struct ThreadState {
	/// Buffer base seek
	base_seek: u64,

	/// Offset from base seek
	offset: u64,

	/// Current buffer
	buffer: Vec<u8>,
}

impl ThreadState {
	/// Returns the base seek as a usize
	fn base_seek_usize(&self) -> usize {
		self.base_seek.try_into().expect("`u64` didn't fit into `usize`")
	}

	/// Returns the offset as a usize
	fn offset_usize(&self) -> usize {
		self.offset.try_into().expect("`u64` didn't fit into `usize`")
	}

	/// Returns the current buffer after the offset
	fn buffer_offset(&self) -> &[u8] {
		if self.offset_usize() > self.buffer.len() {
			return &[];
		}

		&self.buffer[self.offset_usize()..]
	}

	/// Returns the current buffer after the offset mutably
	fn buffer_offset_mut(&mut self) -> &mut [u8] {
		if self.offset_usize() > self.buffer.len() {
			return &mut [];
		}

		let offset = self.offset_usize();
		&mut self.buffer[offset..]
	}
}

impl<'a, T: io::Seek> io::Seek for &'a IoThread<T> {
	fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
		let mut state = self.state().borrow_mut();

		// TODO: If seek is nearby, don't discard buffer
		match pos {
			SeekFrom::Start(pos) => {
				state.base_seek = pos;
				state.offset = 0;
				state.buffer.clear();
			},
			SeekFrom::End(pos) => {
				let len = self.inner.lock_unwrap().stream_len()?;
				state.base_seek = crate::signed_offset(len, pos);
				state.offset = 0;
				state.buffer.clear();
			},
			SeekFrom::Current(offset) => {
				state.offset = crate::signed_offset(state.offset, offset);
			},
		};

		Ok(state.base_seek + state.offset)
	}
}

impl<'a, T: io::Seek + io::Read> io::Read for &'a IoThread<T> {
	fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
		let mut state = self.state().borrow_mut();

		// Read all we have in buffer
		let buffer = state.buffer_offset();
		let bytes_read = Cursor::new(buffer).read(buf)?;
		state.offset += u64::try_from(bytes_read).expect("Unable to get `usize` as `u64`");

		// If we read everything, return
		if bytes_read == buf.len() {
			return Ok(bytes_read);
		}

		// If reading the rest onto the buffer would exceed max size, clear the buffer
		// and set the base offset to the current position.
		if state.buffer.len() + buf.len() - bytes_read >= IoThread::<T>::MAX_BUFFER_SIZE {
			state.buffer.clear();
			state.base_seek += state.offset;
			state.offset = 0;
		}

		// Then read all the remaining bytes onto the buffer
		let remaining_bytes = usize::max(buf.len() - bytes_read, IoThread::<T>::DEFAULT_BUFFER_SIZE);
		let new_len = usize::min(state.offset_usize() + remaining_bytes, IoThread::<T>::MAX_BUFFER_SIZE);
		state.buffer.resize(new_len, 0);

		// TODO: Maybe just use `read`?
		let mut inner = self.inner.lock_unwrap();
		inner.seek(SeekFrom::Start(state.base_seek + state.offset))?;
		inner.read_exact(state.buffer_offset_mut())?;

		// Then recurse
		mem::drop(state);
		self.read(buf)
	}
}

impl<'a, T: io::Seek + io::Write> io::Write for &'a IoThread<T> {
	fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
		todo!()
	}

	fn flush(&mut self) -> io::Result<()> {
		todo!()
	}
}
