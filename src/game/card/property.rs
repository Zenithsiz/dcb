//! Card properties

/// Defines and implements a property enum
// TODO: Make better documentation
// TODO: Turn into a `macro` once they work
macro_rules! generate_enum_property_mod
{
	// Entry point
	(
		// The modules
		$(
			// Module
			$mod_vis:vis mod $mod_name:ident
			{
				// Enum attributes
				$( #[$enum_attr:meta] )*

				// Enum
				enum $enum_name:ident
				{
					// Enum variants
					$(
						// Attributes
						$( #[$enum_variant_attr:meta] )*

						// Variant
						// Note: Must have no data
						$enum_variant_name:ident

						// `Display` conversion name
						($enum_variant_rename:literal)

						=>

						// Variant value
						$enum_variant_value:literal,
					)+

					// Extra fields for `Bytes::from_bytes`.
					$(
						$from_bytes_value:literal => $from_bytes_body:tt,
					)*

					// Error
					_ => $error_unknown_value_display:literal

					$(,)?
				}

				// Any further definitions inside the module
				$( $extra_defs:tt )*
			}
		)*
	) =>
	{
		// Modules
		$(
			// The module
			$mod_vis mod $mod_name
			{
				// The property enum
				$( #[$enum_attr] )*
				#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
				#[derive(::serde::Serialize, ::serde::Deserialize)]
				#[derive(::derive_more::Display)]
				pub enum $enum_name
				{
					$(
						$( #[$enum_variant_attr] )*
						#[serde(rename = $enum_variant_rename)]
						#[display(fmt = $enum_variant_rename)]
						$enum_variant_name = $enum_variant_value,
					)+
				}

				/// Error type for [`$crate::game::Bytes::from_bytes`]
				#[derive(PartialEq, Eq, Clone, Copy, ::std::fmt::Debug, ::thiserror::Error)]
				pub enum FromBytesError {

					/// Unknown value
					#[error($error_unknown_value_display, byte)]
					UnknownValue {
						byte: u8,
					}
				}

				impl $crate::game::Bytes for $enum_name
				{
					type ByteArray = u8;

					type FromError = FromBytesError;
					fn from_bytes(byte: &Self::ByteArray) -> Result<Self, Self::FromError>
					{
						match byte {
							$(
								$enum_variant_value =>
								Ok( <$enum_name>::$enum_variant_name ),
							)+

							$(
								$from_bytes_value => {
									Ok( { $from_bytes_body } )
								}
							)*

							&byte => Err( Self::FromError::UnknownValue{ byte } ),
						}
					}

					type ToError = !;
					#[allow(unreachable_code, unused_variables)] // For when there are multiple values
					fn to_bytes(&self, byte: &mut Self::ByteArray) -> Result<(), Self::ToError>
					{
						*byte = match self {
							$(
								<$enum_name>::$enum_variant_name => $enum_variant_value,
							)+
						};

						Ok(())
					}
				}

				// Extra definitions
				$( $extra_defs )*
			}
		)*
	}
}

/// Implements [`Bytes`](crate::game::Bytes) for `Option<E>` where `E`
/// is the first argument of this macro and an enum.
///
/// This is done by supplying a sentinel value which is read/written as `None`.
macro generate_enum_property_option {
	(
		$( $enum_name:ty => $sentinel_value:literal ),* $(,)?
	) => {
		$(
			#[allow(clippy::diverging_sub_expression)] // Errors might be `!`
			impl $crate::game::Bytes for Option<$enum_name> {
				type ByteArray = <$enum_name as $crate::game::Bytes>::ByteArray;

				type FromError = <$enum_name as $crate::game::Bytes>::FromError;
				fn from_bytes(bytes: &Self::ByteArray) -> Result<Self, Self::FromError>
				{
					match bytes {
						$sentinel_value => Ok( None ),
						_               => Ok( Some( $crate::game::Bytes::from_bytes(bytes)? ) ),
					}
				}

				type ToError = <$enum_name as $crate::game::Bytes>::ToError;
				fn to_bytes(&self, bytes: &mut Self::ByteArray) -> Result<(), Self::ToError>
				{
					match self {
						Some(value) => $crate::game::Bytes::to_bytes(value, bytes)?,
						None        => *bytes = $sentinel_value,
					}

					Ok(())
				}
			}
		)*
	}
}

generate_enum_property_mod!(
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
				match self
				{
					Self::Digimon   => std::mem::size_of::< <crate::game::card::Digimon   as crate::game::Bytes>::ByteArray >(),
					Self::Item      => std::mem::size_of::< <crate::game::card::Item      as crate::game::Bytes>::ByteArray >(),
					Self::Digivolve => std::mem::size_of::< <crate::game::card::Digivolve as crate::game::Bytes>::ByteArray >(),
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
				log::info!("Once the file is patched for the first time, this warning should not appear again.");
				Self::EqualToNumber
			},

			_ => "Unknown byte {:#x} for a effect condition operation",
		}
	}

	pub mod cross_move_effect {
		/// A digimon's cross move effect
		enum CrossMoveEffect
		{
			FirstAttack("Attack first") => 1,

			  CircleTo0("Circle to 0"  ) => 2,
			TriangleTo0("Triangle to 0") => 3,
			   CrossTo0("Cross to 0"   ) => 4,

			  CircleCounter("Circle counter"  ) => 5,
			TriangleCounter("Triangle counter") => 6,
			   CrossCounter("Cross counter"   ) => 7,

			Crash  ("Crash"    ) => 8,
			EatUpHP("Eat Up HP") => 9,
			Jamming("Jamming"  ) => 10,

				FireFoe3x("Fire Foe x3"    ) => 11,
				 IceFoe3x("Ice Foe x3"     ) => 12,
			  NatureFoe3x("Nature Foe x3"  ) => 13,
			DarknessFoe3x("Darkness Foe x3") => 14,
				RareFoe3x("Rare Foe x3"    ) => 15,

			_ => "Unknown byte {:#x} for a cross move effect",
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

generate_enum_property_option!(
	ArrowColor      => 0,
	CrossMoveEffect => 0,
	DigimonProperty => 0,
);

// Complex
pub mod effect;
pub mod effect_condition;
pub mod moves; // Note: Can't be `move`, as it's a keyword

// Exports
pub use arrow_color::ArrowColor;
pub use attack_type::AttackType;
pub use card_type::CardType;
pub use cross_move_effect::CrossMoveEffect;
pub use digimon_property::DigimonProperty;
pub use effect::Effect;
pub use effect_condition::EffectCondition;
pub use effect_condition_operation::EffectConditionOperation;
pub use effect_operation::EffectOperation;
pub use level::Level;
pub use moves::Move;
pub use player_type::PlayerType;
pub use slot::Slot;
pub use speciality::Speciality;
