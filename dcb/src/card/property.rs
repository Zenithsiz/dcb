//! Card properties

dcb_util::generate_enum_property_mod!(
	pub mod slot {
		/// A player's card slots
		enum Slot
		{
			Hand   ("Hand"   ) => 0,
			Dp     ("Dp"     ) => 1,
			Online ("Online" ) => 2,
			Offline("Offline") => 3,

			_ => "Unknown byte {:#x} for a slot"
		}
	}

	pub mod arrow_color {
		/// A digimon effect's arrow color
		enum ArrowColor
		{
			Red  ("Red"  ) => 1,
			Green("Green") => 2,
			Blue ("Blue" ) => 3,

			_ => "Unknown byte {:#x} for an arrow color"
		}
	}

	pub mod attack_type {
		/// A digimon's attack type
		enum AttackType
		{
			Circle  ("Circle"  ) => 0,
			Triangle("Triangle") => 1,
			Cross   ("Cross"   ) => 2,

			_ => "Unknown byte {:#x} for an attack type"
		}
	}

	pub mod card_type {
		/// A card type
		enum CardType
		{
			Digimon  ("Digimon"  ) => 0,
			Item     ("Item"     ) => 1,
			Digivolve("Digivolve") => 2,

			_ => "Unknown byte {:#x} for a card type"
		}

		impl CardType
		{
			/// Returns the byte size of the corresponding card
			#[must_use]
			pub const fn byte_size(self) -> usize
			{
				use crate::card::{Digimon, Item, Digivolve};
				use dcb_bytes::Bytes;

				match self
				{
					Self::Digimon   => std::mem::size_of::< <Digimon   as Bytes>::ByteArray >(),
					Self::Item      => std::mem::size_of::< <Item      as Bytes>::ByteArray >(),
					Self::Digivolve => std::mem::size_of::< <Digivolve as Bytes>::ByteArray >(),
				}
			}
		}
	}

	pub mod player_type {
		/// A player type
		enum PlayerType
		{
			Opponent("Opponent") => 0,
			Player  ("Player"  ) => 1,

			_ => "Unknown byte {:#x} for a player type",
		}
	}

	pub mod level {
		/// A digimon's level
		enum Level
		{
			Rookie  ("Rookie"  ) => 0,
			Armor   ("Armor"   ) => 1,
			Champion("Champion") => 2,
			Ultimate("Ultimate") => 3,

			_ => "Unknown byte {:#x} for a level",
		}
	}

	pub mod speciality {
		/// A digimon's speciality
		enum Speciality
		{
			Fire    ("Fire"    ) => 0,
			Ice     ("Ice"     ) => 1,
			Nature  ("Nature"  ) => 2,
			Darkness("Darkness") => 3,
			Rare    ("Rare"    ) => 4,

			_ => "Unknown byte {:#x} for a speciality",
		}
	}

	pub mod effect_operation {
		/// A digimon's effect operation
		enum EffectOperation
		{
			Addition      ("Addition"      ) => 0,
			Subtraction   ("Subtraction"   ) => 1,
			Multiplication("Multiplication") => 2,
			Division      ("Division"      ) => 3,

			_ => "Unknown byte {:#x} for a effect operation",
		}
	}

	pub mod effect_condition_operation {
		/// A digimon's effect condition operation
		///
		/// # Todo
		/// These don't seem to be 100% right, the less than property, sometimes does less than number, might be a range check
		enum EffectConditionOperation
		{
			LessThanProperty   ("Less than property"   ) => 0,
			LessThanNumber     ("Less than number"     ) => 1,
			MoreThanProperty   ("More than property"   ) => 2,
			MoreThanNumber     ("More than number"     ) => 3,
			DifferentFromNumber("Different from number") => 4,
			EqualToNumber      ("Equal to number"      ) => 5,

			// Aquilamon bug in the original game file
			0xFF => {
				log::warn!("Found byte 0xFF for effect condition operation. Interpreting as `EqualToNumber`");
				log::info!("The previous warning should only appear for \"Aquilamon\" in the original game file.");
				Self::EqualToNumber
			},

			_ => "Unknown byte {:#x} for a effect condition operation",
		}
	}

	pub mod digimon_property {
		/// A digimon's property
		enum DigimonProperty
		{
			OwnSpeciality    ("Own speciality"          ) => 1,
			OpnSpeciality    ("Opponent speciality"     ) => 2,
			OwnHP            ("Own HP"                  ) => 3,
			OpnHP            ("Opponent HP"             ) => 4,
			OwnCircleAttack  ("Own circle attack"       ) => 5,
			OpnCircleAttack  ("Opponent circle attack"  ) => 6,
			OwnTriangleAttack("Own triangle attack"     ) => 7,
			OpnTriangleAttack("Opponent triangle attack") => 8,
			OwnCrossAttack   ("Own cross attack"        ) => 9,
			OpnCrossAttack   ("Opponent cross attack"   ) => 10,
			OwnAttack        ("Own attack"              ) => 11,
			OpnAttack        ("Opponent attack"         ) => 12,
			OwnLevel         ("Own level"               ) => 13,
			OpnLevel         ("Opponent level"          ) => 14,

			OwnAttackType("Own attack type"     )     => 17,
			OpnAttackType("Opponent attack type")     => 18,

			AttackOrder      ("Attack order"                 ) => 20,
			CardsInOwnHand   ("Cards in own hand"            ) => 21,
			CardsInOpnHand   ("Cards in opponent hand"       ) => 22,
			CardsInOwnDpSlot ("Cards in own dp slot"         ) => 23,
			CardsInOpnDpSlot ("Cards in opponent dp slot"    ) => 24,
			CardsInOwnOffDeck("Cards in own offline deck"    ) => 25,
			TempSlot         ("Temp slot"                    ) => 26,
			CardsInOwnOnDeck ("Cards in own online deck"     ) => 27,
			CardsInOpnOnDeck ("Cards in opponent online deck") => 28,

			_ => "Unknown byte {:#x} for a digimon property",
		}
	}
);

/// A possible [`ArrowColor`]
#[repr(transparent)]
#[derive(ref_cast::RefCast)]
#[derive(derive_more::From, derive_more::Into)]
#[derive(dcb_bytes_derive::ProxySentinel)]
#[proxy_sentinel(value = 0, wrapper_type = "ArrowColor")]
pub struct MaybeArrowColor(Option<ArrowColor>);

/// A possible [`CrossMoveEffect`]
#[repr(transparent)]
#[derive(ref_cast::RefCast)]
#[derive(derive_more::From, derive_more::Into)]
#[derive(dcb_bytes_derive::ProxySentinel)]
#[proxy_sentinel(value = 0, wrapper_type = "CrossMoveEffect")]
pub struct MaybeCrossMoveEffect(Option<CrossMoveEffect>);

/// A possible [`DigimonProperty`]
#[repr(transparent)]
#[derive(ref_cast::RefCast)]
#[derive(derive_more::From, derive_more::Into)]
#[derive(dcb_bytes_derive::ProxySentinel)]
#[proxy_sentinel(value = 0, wrapper_type = "DigimonProperty")]
pub struct MaybeDigimonProperty(Option<DigimonProperty>);

// Complex
pub mod cross_move_effect;
pub mod digivolve_effect;
pub mod effect;
pub mod effect_condition;
pub mod moves; // Note: Can't be `move`, as it's a keyword

// Exports
pub use arrow_color::ArrowColor;
pub use attack_type::AttackType;
pub use card_type::CardType;
pub use cross_move_effect::CrossMoveEffect;
pub use digimon_property::DigimonProperty;
pub use digivolve_effect::DigivolveEffect;
pub use effect::{Effect, MaybeEffect};
pub use effect_condition::{EffectCondition, MaybeEffectCondition};
pub use effect_condition_operation::EffectConditionOperation;
pub use effect_operation::EffectOperation;
pub use level::Level;
pub use moves::Move;
pub use player_type::PlayerType;
pub use slot::Slot;
pub use speciality::Speciality;
