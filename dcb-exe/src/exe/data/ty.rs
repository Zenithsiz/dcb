//! Data types

/// Data types
#[derive(PartialEq, Eq, Clone, Hash, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub enum DataType {
	/// Ascii string
	AsciiStr {
		/// String length
		len: usize,
	},

	/// Word
	#[serde(rename = "u32")]
	Word,

	/// Half-word
	#[serde(rename = "u16")]
	HalfWord,

	/// Byte
	#[serde(rename = "u8")]
	Byte,

	/// Array
	Array {
		/// Array type
		ty: Box<DataType>,

		/// Array length
		len: usize,
	},
}

impl DataType {
	/// Returns the size of this data kind
	#[must_use]
	pub fn size(&self) -> usize {
		match self {
			Self::Word => 4,
			Self::HalfWord => 2,
			Self::Byte => 1,
			Self::AsciiStr { len } => *len,
			Self::Array { ty, len } => len * ty.size(),
		}
	}
}
