//! 2D Animation

// Modules
mod error;
pub mod model;

// Exports
pub use error::FromReaderError;
pub use model::TmdModel;

// Imports
use byteorder::{ByteOrder, LittleEndian};
use std::{convert::TryFrom, io};
use zutil::{null_ascii_string::NullAsciiString, AsciiStrArr};

/// 3D Model set
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Model3dSet {
	/// File name
	pub file_name: AsciiStrArr<0xf>,

	/// TODO
	pub unknown0: [u8; 0x20],

	/// Models
	pub models: Vec<(u64, u64, [[i16; 2]; 3], TmdModel)>,
}

impl Model3dSet {
	/// Header size
	pub const HEADER_SIZE: usize = 0x34;
}

impl Model3dSet {
	/// Deserializes a 2d animation data
	pub fn from_reader<R: io::Read + io::Seek>(reader: &mut R) -> Result<Self, FromReaderError> {
		// Read the header
		let mut header_bytes = [0; Self::HEADER_SIZE];
		reader
			.read_exact(&mut header_bytes)
			.map_err(FromReaderError::ReadHeader)?;

		let header_bytes = zutil::array_split!(&header_bytes,
			file_name : [0x10],
			models_len: [0x4],
			unknown0  : [0x20],
		);

		let file_name = header_bytes
			.file_name
			.read_string()
			.map_err(FromReaderError::ParseName)?;
		let models_len = usize::try_from(LittleEndian::read_u32(header_bytes.models_len))
			.expect("Model length didn't fit into `usize`");

		// Read the unknown 1
		let mut unknown1_bytes = vec![0; models_len * 3 * 4];
		reader
			.read_exact(&mut unknown1_bytes)
			.map_err(FromReaderError::ReadUnknown1)?;

		// Read the models
		let mut models = vec![];
		for unknown1 in unknown1_bytes.array_chunks::<12>() {
			let unknown1 = zutil::array_split!(unknown1,
				x0: [0x2],
				y0: [0x2],
				x1: [0x2],
				y1: [0x2],
				x2: [0x2],
				y2: [0x2],
			);

			let unknown1 = [
				[LittleEndian::read_i16(unknown1.x0), LittleEndian::read_i16(unknown1.y0)],
				[LittleEndian::read_i16(unknown1.x1), LittleEndian::read_i16(unknown1.y1)],
				[LittleEndian::read_i16(unknown1.x2), LittleEndian::read_i16(unknown1.y2)],
			];

			let cur_pos = reader.stream_position().map_err(FromReaderError::GetPos)?;
			let model = TmdModel::from_reader(reader).map_err(FromReaderError::ReadModel)?;
			let after_pos = reader.stream_position().map_err(FromReaderError::GetPos)?;
			let size = after_pos - cur_pos;

			models.push((cur_pos, size, unknown1, model));
		}

		Ok(Self {
			file_name,
			unknown0: *header_bytes.unknown0,
			models,
		})
	}
}
