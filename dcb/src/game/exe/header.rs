//! Executable header

// Modules
pub mod error;

// Exports
pub use error::{FromBytesError, ToBytesError};

// Import
use crate::{
	util::{array_split, null_ascii_string::NullAsciiString},
	AsciiStrArr,
};
use byteorder::{ByteOrder, LittleEndian};
use dcb_bytes::Bytes;

/// The header of the executable.
#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Header {
	/// Initial program counter
	pub initial_pc: u32,

	/// Initial global pointer
	pub initial_gp: u32,

	/// Destination in memory for the executable
	pub dest: u32,

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
	pub marker: AsciiStrArr<0x7b4>,
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
			magic            : [0x8],
			_zero            : [0x8],
			initial_pc       : [0x4],
			initial_gp       : [0x4],
			dest             : [0x4],
			size             : [0x4],
			unknown20        : [0x4],
			unknown24        : [0x4],
			memfill_start    : [0x4],
			memfill_size     : [0x4],
			initial_sp_base  : [0x4],
			initial_sp_offset: [0x4],
			_zero2           : [0x13],
			marker           : [0x7b5],
		);

		// If the magic is wrong, return Err
		if bytes.magic != Self::MAGIC {
			return Err(FromBytesError::Magic { magic: *bytes.magic });
		}

		// TODO: Maybe check if `zero` and `zero2` are actually zero?

		Ok(Self {
			initial_pc:        LittleEndian::read_u32(bytes.initial_pc),
			initial_gp:        LittleEndian::read_u32(bytes.initial_gp),
			dest:              LittleEndian::read_u32(bytes.dest),
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
