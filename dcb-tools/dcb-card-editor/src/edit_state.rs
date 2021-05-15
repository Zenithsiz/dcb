//! Card editing state

// Imports
use anyhow::Context;
use dcb_util::AsciiStrArr;
use std::{
	collections::hash_map::DefaultHasher,
	hash::{Hash, Hasher},
	str::FromStr,
};

/// Helper state for managing each card
#[derive(PartialEq, Clone, Hash, Debug)]
pub enum CardEditState {
	Digimon(DigimonEditState),
	Item,
	Digivolve,
}

impl CardEditState {
	/// Creates an edit state from a digimon
	pub fn digimon(digimon: &dcb::Digimon) -> Self {
		Self::Digimon(DigimonEditState::new(digimon))
	}

	/// Returns this card as digimon
	pub fn as_digimon_mut(&mut self) -> Option<&mut DigimonEditState> {
		match self {
			Self::Digimon(state) => Some(state),
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

	/// Calculates the hash of this state
	pub fn calc_hash(&self) -> u64 {
		let mut state = DefaultHasher::new();
		self.hash(&mut state);
		state.finish()
	}
}
