//! Card slots

/// A player's card slots
#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(strum::IntoStaticStr, strum::Display, strum::EnumIter)]
#[derive(dcb_bytes_derive::Discriminant)]
pub enum Slot {
	/// Hand
	Hand    = 0,

	/// Dp
	Dp      = 1,

	/// Online deck
	Online  = 2,

	/// Offline deck
	Offline = 3,
}

impl Slot {
	/// Returns a string representing this slot
	#[must_use]
	pub fn as_str(self) -> &'static str {
		self.into()
	}
}
