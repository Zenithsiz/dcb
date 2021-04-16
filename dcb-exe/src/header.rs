//! Executable header

// Modules
pub mod error;

// Exports
pub use error::{FromBytesError, ToBytesError};

// Imports
use crate::Pos;
use byteorder::{ByteOrder, LittleEndian};
use dcb_bytes::Bytes;
use dcb_util::{array_split, null_ascii_string::NullAsciiString, AsciiStrArr};
use std::fmt;

/// The header of the executable.
#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Header {
	/// Initial program counter
	pub pc0: u32,

	/// Initial global pointer
	pub gp0: u32,

	/// Starting position, in memory, of the executable.
	pub start_pos: Pos,

	/// Size of the executable
	pub size: u32,

	/// Unknown at `0x20`
	pub unknown20: u32,

	/// Unknown at `0x24`
	pub unknown24: u32,

	/// Where to start mem filling
	pub memfill_start: u32,

	/// Size to mem fill
	pub memfill_size: u32,

	/// Initial stack pointer
	pub initial_sp_base: u32,

	/// Offset from initial stack pointer
	pub initial_sp_offset: u32,

	/// Executable region marker
	pub marker: AsciiStrArr<0x7b3>,
}

impl fmt::Display for Header {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let Self {
			ref pc0,
			ref gp0,
			ref start_pos,
			ref size,
			ref memfill_start,
			ref memfill_size,
			ref initial_sp_base,
			ref initial_sp_offset,
			ref marker,
			..
		} = self;

		writeln!(f, "PC: {pc0:#x}")?;
		writeln!(f, "GP: {gp0:#x}")?;
		writeln!(f, "Memory position: {start_pos} / size: {size:#x}")?;
		writeln!(f, "Memfill: {memfill_start:#X} / size: {memfill_size:#x}")?;
		writeln!(f, "SP: {initial_sp_base:#x} / offset: {initial_sp_offset:#x}")?;
		writeln!(f, "Marker: {marker:?}")
	}
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
			unknown20        : [0x4],   // 0x20
			unknown24        : [0x4],   // 0x24
			memfill_start    : [0x4],   // 0x28
			memfill_size     : [0x4],   // 0x2c
			initial_sp_base  : [0x4],   // 0x30
			initial_sp_offset: [0x4],   // 0x34
			_zero2           : [0x14],  // 0x38
			marker           : [0x7b4], // 0x4c
		);

		// If the magic is wrong, return Err
		if bytes.magic != Self::MAGIC {
			return Err(FromBytesError::Magic { magic: *bytes.magic });
		}

		// TODO: Maybe check if `zero` and `zero2` are actually zero?

		Ok(Self {
			pc0:               LittleEndian::read_u32(bytes.pc0),
			gp0:               LittleEndian::read_u32(bytes.gp0),
			start_pos:         Pos(LittleEndian::read_u32(bytes.dest)),
			size:              LittleEndian::read_u32(bytes.size),
			unknown20:         LittleEndian::read_u32(bytes.unknown20),
			unknown24:         LittleEndian::read_u32(bytes.unknown24),
			memfill_start:     LittleEndian::read_u32(bytes.memfill_start),
			memfill_size:      LittleEndian::read_u32(bytes.memfill_size),
			initial_sp_base:   LittleEndian::read_u32(bytes.initial_sp_base),
			initial_sp_offset: LittleEndian::read_u32(bytes.initial_sp_offset),
			marker:            NullAsciiString::read_string(bytes.marker).map_err(FromBytesError::Name)?,
		})
	}

	fn to_bytes(&self, _bytes: &mut Self::ByteArray) -> Result<(), Self::ToError> {
		todo!()
	}
}
