//! Tmd model

// Modules
pub mod error;
pub mod obj;

// Exports
pub use error::FromReaderError;
pub use obj::Obj;

// Imports
use byteorder::{ByteOrder, LittleEndian};
use dcb_bytes::Bytes;
use dcb_util::array_split;
use std::{convert::TryFrom, io};

/// A `.TMD` model.
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct TmdModel {
	/// All objects
	objs: Vec<Obj>,
}

impl TmdModel {
	/// Header size
	pub const HEADER_SIZE: usize = 0xc;
	/// Magic
	pub const MAGIC: [u8; 4] = [0x41, 0x0, 0x0, 0x0];
}

impl TmdModel {
	/// Parses a tmd model from a reader
	pub fn from_reader<R: io::Read + io::Seek>(reader: &mut R) -> Result<Self, FromReaderError> {
		// Read the header
		let mut header_bytes = [0; Self::HEADER_SIZE];
		reader.read_exact(&mut header_bytes).map_err(FromReaderError::ReadHeader)?;
		let header_bytes = array_split!(&header_bytes,
			magic   : [0x4],
			flags   : [0x4],
			objs_len: [0x4],
		);

		// If the magic doesn't match, return Err
		if *header_bytes.magic != Self::MAGIC {
			return Err(FromReaderError::InvalidMagic(*header_bytes.magic));
		}

		// TODO: Support more than just `0x0` flags
		let flags = LittleEndian::read_u32(header_bytes.flags);
		if flags != 0x0 {
			todo!("Flags other than `0x0` ({:#x}) are not supported", flags);
		}

		let objs_len = usize::try_from(LittleEndian::read_u32(header_bytes.objs_len)).expect("Unable to get object number as `usize`");

		// TODO: Support more than 1 object
		if objs_len != 1 {
			todo!("Only 1 object is currently supported per model");
		}

		let mut objs = vec![];
		for _ in 0..objs_len {
			let mut obj_bytes = [0; 0x1c];
			reader.read_exact(&mut obj_bytes).map_err(FromReaderError::ReadObj)?;
			let obj = Obj::from_bytes(&obj_bytes).into_ok();
			objs.push(obj);
		}

		// Skip past the rest of the model
		// TODO: Read everything
		// TODO: Calculate end some other way
		// Note: `- 0x1c` is because of the object we have to read.
		let end = objs[0].normal_pos + 0x8 * objs[0].normal_len;
		reader
			.seek(io::SeekFrom::Current(i64::from(end - 0x1c)))
			.map_err(FromReaderError::SeekPastModel)?;

		Ok(Self { objs })
	}
}
