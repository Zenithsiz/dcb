//! A `.PAK` entry

// Modules
pub mod animation2d;
pub mod error;

// Exports
pub use animation2d::Animation2d;
pub use error::DeserializeError;

// Imports
use super::{header::Kind, Header};

/// A `.PAK` entry
#[derive(PartialEq, Eq, Clone, Debug)]
pub enum PakEntry {
	/// Unknown 0
	Unknown0(Vec<u8>),

	/// Unknown 1
	Unknown1(Vec<u8>),

	/// Game script, `MSCD`
	GameScript(Vec<u8>),

	/// 2D Animation
	Animation2d(Animation2d),

	/// File sub-header
	FileSubHeader(Vec<u8>),

	/// File contents
	FileContents(Vec<u8>),

	/// Audio `SEQ`
	AudioSeq(Vec<u8>),

	/// Audio `VH`
	AudioVh(Vec<u8>),

	/// Audio `VB`
	AudioVb(Vec<u8>),
}

impl PakEntry {
	/// Deserializes a `.PAK` file entry from it's header and ata
	pub fn deserialize(header: Header, data: Vec<u8>) -> Result<Self, DeserializeError> {
		let kind = header.file_kind;
		let entry = match kind {
			Kind::Unknown0 => Self::Unknown0(data),
			Kind::Unknown1 => Self::Unknown1(data),
			Kind::GameScript => Self::GameScript(data),
			Kind::Animation2D => Self::Animation2d(Animation2d::deserialize(std::io::Cursor::new(data)).map_err(DeserializeError::ParseAnimation2D)?),
			Kind::FileSubHeader => Self::FileSubHeader(data),
			Kind::FileContents => Self::FileContents(data),
			Kind::AudioSeq => Self::AudioSeq(data),
			Kind::AudioVh => Self::AudioVh(data),
			Kind::AudioVb => Self::AudioVb(data),
		};

		Ok(entry)
	}
}