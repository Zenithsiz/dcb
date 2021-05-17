//! Effect operation

/// A card's effect operation
#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(strum::IntoStaticStr, strum::Display, strum::EnumIter)]
#[derive(dcb_bytes_derive::Discriminant)]
pub enum EffectOperation {
	/// Addition
	Addition       = 0,

	/// Subtraction
	Subtraction    = 1,

	/// Multiplication
	Multiplication = 2,

	/// Division
	Division       = 3,
}

impl EffectOperation {
	/// Returns a string representing this operation
	#[must_use]
	pub fn as_str(self) -> &'static str {
		self.into()
	}

	/// Returns the operator string of this operation
	#[must_use]
	pub const fn operator_str(self) -> &'static str {
		match self {
			Self::Addition => "+",
			Self::Subtraction => "-",
			Self::Multiplication => "*",
			Self::Division => "/",
		}
	}
}
