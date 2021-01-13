//! Data types

/// Data types
#[derive(PartialEq, Eq, Clone, Hash, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(derive_more::Display)]
pub enum DataType {
	/// Ascii string
	#[display(fmt = "Str<{len}>")]
	AsciiStr {
		/// String length
		len: usize,
	},

	/// Word
	#[display(fmt = "u32")]
	#[serde(rename = "u32")]
	Word,

	/// Half-word
	#[display(fmt = "u16")]
	#[serde(rename = "u16")]
	HalfWord,

	/// Byte
	#[display(fmt = "u8")]
	#[serde(rename = "u8")]
	Byte,

	/// Array
	#[display(fmt = "Arr<{ty}, {len}>")]
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
			// Round strings to the nearest word
			Self::AsciiStr { len } => len + 4 - (len % 4),
			Self::Array { ty, len } => len * ty.size(),
		}
	}

	/// Returns the alignment of this data kind
	#[must_use]
	#[allow(clippy::match_same_arms)] // Looks better like this
	pub fn align(&self) -> usize {
		match self {
			Self::Word => 4,
			Self::HalfWord => 2,
			Self::Byte => 1,
			Self::AsciiStr { .. } => 4,
			Self::Array { ty, .. } => ty.align(),
		}
	}
}
