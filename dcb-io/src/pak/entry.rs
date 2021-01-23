//! A `.PAK` entry

// Modules
pub mod animation2d;
pub mod error;

// Exports
pub use animation2d::Animation2d;
pub use error::FromReaderError;

// Imports
use super::{header::Kind, Header};
use std::io;

/// A `.PAK` entry
#[derive(PartialEq, Eq, Clone, Debug)]
pub enum PakEntry {
	/// Unknown 0
	Unknown0,

	/// Unknown 1
	Unknown1,

	/// Game script, `MSCD`
	GameScript,

	/// 2D Animation
	Animation2d(Animation2d),

	/// File sub-header
	FileSubHeader,

	/// File contents
	FileContents,

	/// Audio `SEQ`
	AudioSeq,

	/// Audio `VH`
	AudioVh,

	/// Audio `VB`
	AudioVb,
}

impl PakEntry {
	/// Deserializes a `.PAK` file entry from a reader
	pub fn from_reader<R: io::Read>(reader: &mut R, header: Header) -> Result<Self, FromReaderError> {
		let kind = header.file_kind;
		let entry = match kind {
			Kind::Unknown0 => Self::Unknown0,
			Kind::Unknown1 => Self::Unknown1,
			Kind::GameScript => Self::GameScript,
			Kind::Animation2D => Self::Animation2d(Animation2d::deserialize(reader).map_err(FromReaderError::ParseAnimation2D)?),
			Kind::FileSubHeader => Self::FileSubHeader,
			Kind::FileContents => Self::FileContents,
			Kind::AudioSeq => Self::AudioSeq,
			Kind::AudioVh => Self::AudioVh,
			Kind::AudioVb => Self::AudioVb,
		};

		Ok(entry)
	}
}
