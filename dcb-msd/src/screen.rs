//! Screens

/// Screens
#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub enum Screen {
	/// Player room
	PlayerRoom,

	/// Card list
	CardList,

	/// Choose Partner
	ChoosePartner,

	/// Edit partner
	EditPartner,

	/// Keyboard
	Keyboard,
}

impl Screen {
	/// Returns a string representing this screen
	#[must_use]
	pub const fn as_str(self) -> &'static str {
		match self {
			Screen::PlayerRoom => "Player's Room",
			Screen::CardList => "Card List",
			Screen::ChoosePartner => "Choose Partner",
			Screen::EditPartner => "Edit Partner",
			Screen::Keyboard => "Keyboard",
		}
	}
}
