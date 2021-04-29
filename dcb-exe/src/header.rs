//! Executable header

// Modules
pub mod error;

// Exports
pub use error::{FromBytesError, ToBytesError};

// Imports
use crate::Pos;
use byteorder::{ByteOrder, LittleEndian};
use dcb_bytes::Bytes;
use dcb_util::{array_split, array_split_mut, null_ascii_string::NullAsciiString, AsciiStrArr};

/// Executable header
#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Header {
	/// Initial value for `$pc`, the program counter.
	///
	/// This will be where the psx starts executing code.
	/// It is typically the start of some `start` function.
	pub pc0: u32,

	/// Initial value for `$gp`
	pub gp0: u32,

	/// Position of the executable in memory.
	///
	/// Must be a multiple of `2048`.
	pub start_pos: Pos,

	/// Size of the executable
	pub size: u32,

	/// Location to zero out
	pub memfill_start: u32,

	/// Number of bytes to zero out.
	pub memfill_size: u32,

	/// Initial `$sp` / `$fp`
	pub initial_sp_base: u32,

	/// Offset to add to `$sp` / `$fp`
	pub initial_sp_offset: u32,

	/// Marker for ascii text
	pub marker: AsciiStrArr<0x7b3>,
}

impl Header {
	/// Magic
	pub const MAGIC: &'static [u8; 8] = b"PS-X EXE";
}

impl Bytes for Header {
	type ByteArray = [u8; 0x800];
	type FromError = FromBytesError;
	type ToError = ToBytesError;

	fn from_bytes(bytes: &Self::ByteArray) -> Result<Self, Self::FromError> {
		let bytes = array_split!(bytes,
			magic            : [0x8],   // 0x0
			_zero            : [0x8],   // 0x8
			pc0              : [0x4],   // 0x10
			gp0              : [0x4],   // 0x14
			dest             : [0x4],   // 0x18
			size             : [0x4],   // 0x1c
			_zero2           : [0x8],   // 0x20
			memfill_start    : [0x4],   // 0x28
			memfill_size     : [0x4],   // 0x2c
			initial_sp_base  : [0x4],   // 0x30
			initial_sp_offset: [0x4],   // 0x34
			_zero3           : [0x14],  // 0x38
			marker           : [0x7b4], // 0x4c
		);

		// If the magic is wrong, return Err
		if bytes.magic != Self::MAGIC {
			return Err(FromBytesError::Magic { magic: *bytes.magic });
		}

		// If the size isn't aligned, return Err
		let size = LittleEndian::read_u32(bytes.size);
		if size % 0x800 != 0 {
			return Err(FromBytesError::SizeAlignment { size });
		}

		Ok(Self {
			pc0: LittleEndian::read_u32(bytes.pc0),
			gp0: LittleEndian::read_u32(bytes.gp0),
			start_pos: Pos(LittleEndian::read_u32(bytes.dest)),
			size,
			memfill_start: LittleEndian::read_u32(bytes.memfill_start),
			memfill_size: LittleEndian::read_u32(bytes.memfill_size),
			initial_sp_base: LittleEndian::read_u32(bytes.initial_sp_base),
			initial_sp_offset: LittleEndian::read_u32(bytes.initial_sp_offset),
			marker: NullAsciiString::read_string(bytes.marker).map_err(FromBytesError::Name)?,
		})
	}

	fn to_bytes(&self, bytes: &mut Self::ByteArray) -> Result<(), Self::ToError> {
		let bytes = array_split_mut!(bytes,
			magic            : [0x8],   // 0x0
			zero             : [0x8],   // 0x8
			pc0              : [0x4],   // 0x10
			gp0              : [0x4],   // 0x14
			dest             : [0x4],   // 0x18
			size             : [0x4],   // 0x1c
			zero2            : [0x8],   // 0x20
			memfill_start    : [0x4],   // 0x28
			memfill_size     : [0x4],   // 0x2c
			initial_sp_base  : [0x4],   // 0x30
			initial_sp_offset: [0x4],   // 0x34
			zero3            : [0x14],  // 0x38
			marker           : [0x7b4], // 0x4c
		);

		// Write the magic and zeroes
		*bytes.magic = *Self::MAGIC;
		bytes.zero.fill(0);
		bytes.zero2.fill(0);
		bytes.zero3.fill(0);

		LittleEndian::write_u32(bytes.pc0, self.pc0);
		LittleEndian::write_u32(bytes.gp0, self.gp0);
		LittleEndian::write_u32(bytes.dest, self.start_pos.0);
		LittleEndian::write_u32(bytes.size, self.size);
		LittleEndian::write_u32(bytes.memfill_start, self.memfill_start);
		LittleEndian::write_u32(bytes.memfill_size, self.memfill_size);
		LittleEndian::write_u32(bytes.initial_sp_base, self.initial_sp_base);
		LittleEndian::write_u32(bytes.initial_sp_offset, self.initial_sp_offset);
		bytes.marker.write_string(&self.marker);

		Ok(())
	}
}
