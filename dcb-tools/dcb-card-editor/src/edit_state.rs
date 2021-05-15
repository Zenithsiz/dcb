//! Card editing state

// Imports
use anyhow::Context;
use dcb_util::AsciiStrArr;
use std::str::FromStr;

/// Helper state for managing each card
#[derive(PartialEq, Clone, Hash, Debug)]
pub enum CardEditState {
	Digimon(DigimonEditState),
	Item(ItemEditState),
	Digivolve(DigivolveEditState),
}

impl CardEditState {
	/// Creates an edit state from a digimon
	pub fn digimon(digimon: &dcb::Digimon) -> Self {
		Self::Digimon(DigimonEditState::new(digimon))
	}

	/// Creates an edit state from an item
	pub fn item(item: &dcb::Item) -> Self {
		Self::Item(ItemEditState::new(item))
	}

	/// Creates an edit state from a digivolve
	pub fn digivolve(digivolve: &dcb::Digivolve) -> Self {
		Self::Digivolve(DigivolveEditState::new(digivolve))
	}

	/// Returns this card as digimon
	pub fn as_digimon_mut(&mut self) -> Option<&mut DigimonEditState> {
		match self {
			Self::Digimon(state) => Some(state),
			_ => None,
		}
	}

	pub fn as_item_mut(&mut self) -> Option<&mut ItemEditState> {
		match self {
			Self::Item(state) => Some(state),
			_ => None,
		}
	}

	pub fn as_digivolve_mut(&mut self) -> Option<&mut DigivolveEditState> {
		match self {
			Self::Digivolve(state) => Some(state),
			_ => None,
		}
	}
}

/// Digimon card edit state
#[derive(PartialEq, Clone, Hash, Debug)]
pub struct DigimonEditState {
	pub name:               String,
	pub move_circle_name:   String,
	pub move_triangle_name: String,
	pub move_cross_name:    String,
	pub effect_description: [String; 4],
}

impl DigimonEditState {
	/// Creates an edit state from a digimon
	pub fn new(digimon: &dcb::Digimon) -> Self {
		Self {
			name:               digimon.name.to_string(),
			move_circle_name:   digimon.move_circle.name.to_string(),
			move_triangle_name: digimon.move_triangle.name.to_string(),
			move_cross_name:    digimon.move_cross.name.to_string(),
			effect_description: digimon.effect_description.map(|s| s.to_string()),
		}
	}

	/// Applies this state to a digimon
	pub fn apply(&self, digimon: &mut dcb::Digimon) -> Result<(), anyhow::Error> {
		digimon.name = AsciiStrArr::from_str(&self.name).context("Unable to set name")?;
		digimon.move_circle.name =
			AsciiStrArr::from_str(&self.move_circle_name).context("Unable to set circle move name")?;
		digimon.move_triangle.name =
			AsciiStrArr::from_str(&self.move_triangle_name).context("Unable to set triangle move name")?;
		digimon.move_cross.name =
			AsciiStrArr::from_str(&self.move_cross_name).context("Unable to set cross move name")?;

		for (idx, effect_description) in self.effect_description.iter().enumerate() {
			digimon.effect_description[idx] =
				AsciiStrArr::from_str(effect_description).context("Unable to set effect description")?;
		}

		Ok(())
	}
}

/// Item card edit state
#[derive(PartialEq, Clone, Hash, Debug)]
pub struct ItemEditState {
	pub name:               String,
	pub effect_description: [String; 4],
}

impl ItemEditState {
	/// Creates an edit state from an item
	pub fn new(item: &dcb::Item) -> Self {
		Self {
			name:               item.name.to_string(),
			effect_description: item.effect_description.map(|s| s.to_string()),
		}
	}

	/// Applies this state to an item
	pub fn apply(&self, item: &mut dcb::Item) -> Result<(), anyhow::Error> {
		item.name = AsciiStrArr::from_str(&self.name).context("Unable to set name")?;

		for (idx, effect_description) in self.effect_description.iter().enumerate() {
			item.effect_description[idx] =
				AsciiStrArr::from_str(effect_description).context("Unable to set effect description")?;
		}

		Ok(())
	}
}

/// Digivolve card edit state
#[derive(PartialEq, Clone, Hash, Debug)]
pub struct DigivolveEditState {
	pub name:               String,
	pub effect_description: [String; 4],
}

impl DigivolveEditState {
	/// Creates an edit state from a digivolve
	pub fn new(digivolve: &dcb::Digivolve) -> Self {
		Self {
			name:               digivolve.name.to_string(),
			effect_description: digivolve.effect_description.map(|s| s.to_string()),
		}
	}

	/// Applies this state to a digivolve
	pub fn apply(&self, digivolve: &mut dcb::Digivolve) -> Result<(), anyhow::Error> {
		digivolve.name = AsciiStrArr::from_str(&self.name).context("Unable to set name")?;

		for (idx, effect_description) in self.effect_description.iter().enumerate() {
			digivolve.effect_description[idx] =
				AsciiStrArr::from_str(effect_description).context("Unable to set effect description")?;
		}

		Ok(())
	}
}
