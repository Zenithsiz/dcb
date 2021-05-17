//! Effect arrow color

/// A card effect's arrow color
#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(strum::IntoStaticStr, strum::Display, strum::EnumIter)]
#[derive(dcb_bytes_derive::Discriminant)]
pub enum ArrowColor {
	/// Red
	Red   = 1,
	/// Green
	Green = 2,
	/// Blue
	Blue  = 3,
}

impl ArrowColor {
	/// Returns a string representing this color
	#[must_use]
	pub fn as_str(self) -> &'static str {
		self.into()
	}
}
