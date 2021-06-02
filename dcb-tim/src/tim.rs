#![doc(include = "tim.md")]

// Modules
pub mod error;

// Exports
pub use error::{ColorsError, DeserializeError};

// Imports
use crate::{img::Pixels, Clut, Color, Header, Img};
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
		let header = Header::deserialize_bytes(&header_bytes).map_err(DeserializeError::ParseHeader)?;

		// If we have a clut, read it
		let clut = header
			.clut_present
			.then(|| Clut::deserialize(reader))
			.transpose()
			.map_err(DeserializeError::DeserializeClut)?;

		let img = Img::deserialize(reader, header.bpp).map_err(DeserializeError::DeserializeImg)?;

		// If this is indexed, return `Err` if we don't have a clut table
		if img.is_indexed() && clut.is_none() {
			return Err(DeserializeError::IndexMissingClut);
		}

		Ok(Self { clut, img })
	}

	/// Returns this image's size
	#[must_use]
	pub fn size(&self) -> [usize; 2] {
		let [width, height] = self.img.header.size.map(usize::from);
		let width = match &self.img.pixels {
			Pixels::Index4Bit(_) => width * 4,
			Pixels::Index8Bit(_) => width * 2,
			Pixels::Color16Bit(_) => width,
			Pixels::Color24Bit(_) => todo!(),
		};

		[width, height]
	}

	/// Returns the number of pallettes
	#[must_use]
	pub fn pallettes(&self) -> usize {
		match &self.img.pixels {
			Pixels::Index4Bit(_) => self.clut.as_ref().map_or(0, |clut| (clut.colors.len() + 15) / 16),
			Pixels::Index8Bit(_) => self.clut.as_ref().map_or(0, |clut| (clut.colors.len() + 255) / 256),
			Pixels::Color16Bit(_) | Pixels::Color24Bit(_) => 0,
		}
	}

	/// Returns all colors
	pub fn colors(&self, pallette: Option<usize>) -> Result<Box<[[u8; 4]]>, ColorsError> {
		// If the pallette is invalid, return Err
		let pallette = match pallette {
			Some(pallette) if pallette >= self.pallettes() => return Err(ColorsError::InvalidPallette),
			_ => pallette.unwrap_or(0),
		};

		let colors: Vec<_> = match &self.img.pixels {
			Pixels::Index4Bit(idxs) => {
				let clut = self.clut.as_ref().ok_or(ColorsError::MissingClut)?;
				idxs.iter()
					.map(|&idx| clut.colors.get(16 * pallette + idx).copied())
					.collect::<Option<Vec<_>>>()
					.ok_or(ColorsError::ColorOutOfBounds)?
					.into_iter()
					.map(Color::to_rgba)
					.collect()
			},
			Pixels::Index8Bit(idxs) => {
				let clut = self.clut.as_ref().ok_or(ColorsError::MissingClut)?;
				idxs.iter()
					.map(|&idx| clut.colors.get(256 * pallette + idx).copied())
					.collect::<Option<Vec<_>>>()
					.ok_or(ColorsError::ColorOutOfBounds)?
					.into_iter()
					.map(Color::to_rgba)
					.collect()
			},
			Pixels::Color16Bit(colors) | Pixels::Color24Bit(colors) => {
				colors.iter().copied().map(Color::to_rgba).collect()
			},
		};

		Ok(colors.into_boxed_slice())
	}
}
