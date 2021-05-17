//! Digimon attack type

/// A digimon's attack type
#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(strum::IntoStaticStr, strum::Display, strum::EnumIter)]
#[derive(dcb_bytes_derive::Discriminant)]
pub enum AttackType {
	/// Circle
	Circle   = 0,

	/// Triangle
	Triangle = 1,

	/// Cross
	Cross    = 2,
}

impl AttackType {
	/// Returns a string representing this attack type
	#[must_use]
	pub fn as_str(self) -> &'static str {
		self.into()
	}
}
