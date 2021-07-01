#![doc = include_str!("digivolve.md")]

// Modules
mod diff;
mod error;

// Exports
pub use diff::{DiffKind, DiffVisitor};
pub use error::DeserializeBytesError;

// Imports
use crate::card::property::DigivolveEffect;
use dcb_bytes::Bytes;
use std::{iter, ops::Try};
use zutil::{null_ascii_string::NullAsciiString, AsciiStrArr};

/// A digivolve card
///
/// Contains all information about each digivolve card stored in the [`Card Table`](crate::card::table::Table)
#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Digivolve {
	/// The item's name
	pub name: AsciiStrArr<0x14>,

	/// The effect's description.
	///
	/// The description is split along 4 lines
	pub effect_description: [AsciiStrArr<0x14>; 4],

	/// Effect
	pub effect: DigivolveEffect,
}

impl Digivolve {
	/// Lists the differences between two items
	pub fn diff<V: DiffVisitor>(&self, rhs: &Self, visitor: &mut V) -> V::Result {
		let lhs = self;

		if lhs.name != rhs.name {
			visitor.visit_name(&lhs.name, &rhs.name)?;
		}
		for (idx, (lhs_desc, rhs_desc)) in
			iter::zip(lhs.effect_description.each_ref(), rhs.effect_description.each_ref()).enumerate()
		{
			if lhs_desc != rhs_desc {
				visitor.visit_effect_description(idx, lhs_desc, rhs_desc)?;
			}
		}
		if lhs.effect != rhs.effect {
			visitor.visit_effect(lhs.effect, rhs.effect)?;
		}

		<V::Result as Try>::from_output(())
	}
}

impl Bytes for Digivolve {
	type ByteArray = [u8; 0x6c];
	type DeserializeError = DeserializeBytesError;
	type SerializeError = !;

	fn deserialize_bytes(bytes: &Self::ByteArray) -> Result<Self, Self::DeserializeError> {
		// Split bytes
		let bytes = zutil::array_split!(bytes,
			name                : [0x15],
			effect              : [0x3],
			effect_description_0: [0x15],
			effect_description_1: [0x15],
			effect_description_2: [0x15],
			effect_description_3: [0x15],
		);

		Ok(Self {
			// Name
			name: bytes.name.read_string().map_err(DeserializeBytesError::Name)?,

			// Effect
			effect_description: [
				bytes
					.effect_description_0
					.read_string()
					.map_err(DeserializeBytesError::EffectDescription1)?,
				bytes
					.effect_description_1
					.read_string()
					.map_err(DeserializeBytesError::EffectDescription2)?,
				bytes
					.effect_description_2
					.read_string()
					.map_err(DeserializeBytesError::EffectDescription3)?,
				bytes
					.effect_description_3
					.read_string()
					.map_err(DeserializeBytesError::EffectDescription4)?,
			],

			// Unknown
			effect: DigivolveEffect::deserialize_bytes(bytes.effect).map_err(DeserializeBytesError::Effect)?,
		})
	}

	fn serialize_bytes(&self, bytes: &mut Self::ByteArray) -> Result<(), Self::SerializeError> {
		// Split bytes
		let bytes = zutil::array_split_mut!(bytes,
			name                : [0x15],
			effect              : [0x3],
			effect_description_0: [0x15],
			effect_description_1: [0x15],
			effect_description_2: [0x15],
			effect_description_3: [0x15],
		);

		// Name
		bytes.name.write_string(&self.name);

		// Effects
		bytes.effect_description_0.write_string(&self.effect_description[0]);
		bytes.effect_description_1.write_string(&self.effect_description[1]);
		bytes.effect_description_2.write_string(&self.effect_description[2]);
		bytes.effect_description_3.write_string(&self.effect_description[3]);

		// Unknown
		self.effect.serialize_bytes(bytes.effect).into_ok();

		// Return Ok
		Ok(())
	}
}
