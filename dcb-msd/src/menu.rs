//! Menus

/// Combo box
#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub enum ComboBox {
	/// Small combo box with 3 buttons
	Small,

	/// Large combo box with 5 buttons
	Large,
}

impl ComboBox {
	/// Returns a string representing this combo box
	#[must_use]
	pub const fn as_str(self) -> &'static str {
		match self {
			ComboBox::Small => "small",
			ComboBox::Large => "large",
		}
	}

	/// Returns if a button is allowed in this box
	#[must_use]
	pub const fn button_allowed(self, button: ComboBoxButton) -> bool {
		#[allow(clippy::enum_glob_use)] // It's local to shit small function
		use ComboBoxButton::*;

		match self {
			Self::Small => matches!(button, Talk | Battle | DeckData | Save | Yes | No | Cards | Partner),
			Self::Large => matches!(
				button,
				PlayerRoom |
					Menu | BattleCafe | BattleArena |
					ExtraArena | BeetArena |
					HauntedArena | FusionShop |
					Yes | No
			),
		}
	}

	/// Returns a button of this box given it's value
	#[must_use]
	pub const fn parse_button(self, value: u16) -> Option<ComboBoxButton> {
		#[allow(clippy::match_same_arms)] // We want it in ascending order
		let button = match (self, value) {
			(Self::Large, 0x0) => ComboBoxButton::PlayerRoom,
			(Self::Large, 0x1) => ComboBoxButton::Menu,
			(Self::Large, 0x2) => ComboBoxButton::BattleCafe,
			(Self::Large, 0x3) => ComboBoxButton::BattleArena,
			(Self::Large, 0x4) => ComboBoxButton::ExtraArena,
			(Self::Large, 0x5) => ComboBoxButton::BeetArena,
			(Self::Large, 0x6) => ComboBoxButton::HauntedArena,
			(Self::Large, 0x7) => ComboBoxButton::FusionShop,
			(Self::Large, 0x8) => ComboBoxButton::Yes,
			(Self::Large, 0x9) => ComboBoxButton::No,
			(Self::Small, 0x0c) => ComboBoxButton::Talk,
			(Self::Small, 0x0d) => ComboBoxButton::Battle,
			(Self::Small, 0x0e) => ComboBoxButton::DeckData,
			(Self::Small, 0x0f) => ComboBoxButton::Save,
			(Self::Small, 0x10) => ComboBoxButton::Yes,
			(Self::Small, 0x11) => ComboBoxButton::No,
			(Self::Small, 0x12) => ComboBoxButton::Cards,
			(Self::Small, 0x13) => ComboBoxButton::Partner,
			_ => return None,
		};
		Some(button)
	}
}

/// Combo box buttons
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum ComboBoxButton {
	/// "Player's Room"
	PlayerRoom   = 0x0,

	/// "Menu"
	Menu         = 0x1,

	/// "Battle Cafe"
	BattleCafe   = 0x2,

	/// "Battle Arena"
	BattleArena  = 0x3,

	/// "Extra Arena"
	ExtraArena   = 0x4,

	/// "Beet Arena"
	BeetArena    = 0x5,

	/// "Haunted Arena"
	HauntedArena = 0x6,

	/// "Fusion shop"
	FusionShop   = 0x7,

	/// "Yes"
	Yes          = 0x8,

	/// "No"
	No           = 0x9,

	/// "Talk"
	Talk         = 0x0c,

	/// "Battle"
	Battle       = 0x0d,

	/// "Deck Data"
	DeckData     = 0x0e,

	/// "Save"
	Save         = 0x0f,

	/// "Cards"
	Cards        = 0x12,

	/// "Partner"
	Partner      = 0x13,
}

impl ComboBoxButton {
	/// Returns a string representing this button
	#[must_use]
	pub const fn as_str(self) -> &'static str {
		match self {
			Self::PlayerRoom => "Player's room",
			Self::Menu => "Menu",
			Self::BattleCafe => "Battle Cafe",
			Self::BattleArena => "Battle Arena",
			Self::ExtraArena => "Extra Arena",
			Self::BeetArena => "Beet Arena",
			Self::HauntedArena => "Haunted Arena",
			Self::FusionShop => "Fusion shop",
			Self::Yes => "Yes",
			Self::No => "No",
			Self::Talk => "Talk",
			Self::Battle => "Battle",
			Self::DeckData => "DeckData",
			Self::Save => "Save",
			Self::Cards => "Cards",
			Self::Partner => "Partner",
		}
	}

	/// Returns the value of this button
	#[allow(clippy::as_conversions)] // We're just getting the value, as above defined
	#[must_use]
	pub const fn as_u16(self) -> u16 {
		self as u16
	}
}
