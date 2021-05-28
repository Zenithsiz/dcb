//! Tim

// Modules
pub mod error;

// Exports
pub use error::{ColorsError, DeserializeError};

// Imports
use crate::{img::Colors, Clut, Color, Header, Img};
use dcb_bytes::Bytes;
use std::io;

/// `tim` file
#[derive(PartialEq, Clone, Debug)]
pub struct Tim {
	/// Clut
	pub clut: Option<Clut>,

	/// Image
	pub img: Img,
}

impl Tim {
	/// Deserializes a tim file
	pub fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, DeserializeError> {
		// Read and parse the header
		let mut header_bytes = [0u8; 0x8];
		reader
			.read_exact(&mut header_bytes)
			.map_err(DeserializeError::ReadHeader)?;
		let header = Header::from_bytes(&header_bytes).map_err(DeserializeError::ParseHeader)?;

		// If we have a clut, read it
		let clut = header
			.clut_present
			.then(|| Clut::deserialize(reader).map_err(DeserializeError::DeserializeClut))
			.transpose()?;

		let img = Img::deserialize(reader, header.bbp).map_err(DeserializeError::DeserializeImg)?;

		Ok(Self { clut, img })
	}

	/// Returns this image's size
	#[must_use]
	pub fn size(&self) -> [usize; 2] {
		let [width, height] = self.img.header.size.map(usize::from);
		let width = match &self.img.colors {
			Colors::Index4Bit(_) => width * 4,
			Colors::Index8Bit(_) => width * 2,
			Colors::Color16Bit(_) => width,
			Colors::Color24Bit(_) => todo!(),
		};

		[width, height]
	}

	/// Returns all colors
	// TODO: Index checking
	pub fn colors(&self) -> Result<Box<[[u8; 4]]>, ColorsError> {
		let colors: Vec<_> = match &self.img.colors {
			Colors::Index4Bit(idxs) => {
				let clut = self.clut.as_ref().ok_or(ColorsError::MissingClut)?;
				idxs.iter().map(|&idx| clut.colors[idx]).map(Color::to_rgba).collect()
			},
			Colors::Index8Bit(idxs) => {
				let clut = self.clut.as_ref().ok_or(ColorsError::MissingClut)?;
				idxs.iter().map(|&idx| clut.colors[idx]).map(Color::to_rgba).collect()
			},
			Colors::Color16Bit(colors) | Colors::Color24Bit(colors) => {
				colors.iter().copied().map(Color::to_rgba).collect()
			},
		};

		Ok(colors.into_boxed_slice())
	}
}
