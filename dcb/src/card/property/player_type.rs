//! Player type

/// A player type
#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(strum::IntoStaticStr, strum::Display, strum::EnumIter)]
#[derive(dcb_bytes_derive::Discriminant)]
pub enum PlayerType {
	/// Opponent
	Opponent = 0,

	/// Player
	Player   = 1,
}

impl PlayerType {
	/// Returns a string representing this player type
	#[must_use]
	pub fn as_str(self) -> &'static str {
		self.into()
	}
}
