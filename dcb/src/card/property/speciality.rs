//! Digimon speciality

/// A digimon's speciality
#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(strum::IntoStaticStr, strum::Display, strum::EnumIter)]
#[derive(dcb_bytes_derive::Discriminant)]
pub enum Speciality {
	/// Fire
	Fire     = 0,

	/// Ice
	Ice      = 1,

	/// Nature
	Nature   = 2,

	/// Darkness
	Darkness = 3,

	/// Rare
	Rare     = 4,
}

impl Speciality {
	/// Returns a string representing this speciality
	#[must_use]
	pub fn as_str(self) -> &'static str {
		self.into()
	}
}
