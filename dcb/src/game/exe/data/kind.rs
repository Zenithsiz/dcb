//! Data kind
//!
//! Every piece of data within the executable
//! may have a certain kind, an ascii string,
//! a table of words, a single byte, etc.

/// Data kind
#[derive(PartialEq, Eq, Clone, Hash, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(derive_more::Display)]
pub enum DataKind {
	/// Ascii string
	#[display(fmt = "str")]
	AsciiStr {
		/// String length
		len: u32,
	},

	/// Word
	#[display(fmt = "u32")]
	Word,

	/// Half-word
	#[display(fmt = "u16")]
	HalfWord,

	/// Byte
	#[display(fmt = "u8")]
	Byte,

	/// Array
	#[display(fmt = "[{ty}; {len}]")]
	Array {
		/// Array type
		ty: Box<DataKind>,

		/// Array length
		len: u32,
	},
}

impl DataKind {
	/// Returns the size of this data kind
	#[must_use]
	pub fn size(&self) -> u32 {
		match self {
			Self::AsciiStr { len } => len + 4 - (len % 4),
			Self::Word => 4,
			Self::HalfWord => 2,
			Self::Byte => 1,
			Self::Array { ty, len } => ty.size() * len,
		}
	}
}
