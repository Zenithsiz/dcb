//! Digimon level

/// A digimon's level
#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(strum::IntoStaticStr, strum::Display, strum::EnumIter)]
#[derive(dcb_bytes_derive::Discriminant)]
pub enum Level {
	/// Rookie
	Rookie   = 0,

	/// Armor
	Armor    = 1,

	/// Champion
	Champion = 2,

	/// Ultimate
	Ultimate = 3,
}


impl Level {
	/// Returns a string representing this level
	#[must_use]
	pub fn as_str(self) -> &'static str {
		self.into()
	}
}
