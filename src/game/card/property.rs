//! Card properties

// Macros
//--------------------------------------------------------------------------------------------------
	/// Defines a module and an enum inside of it representing a simple property
	/// 
	/// # Details
	/// Both the enum and the error inherit the module's visibility
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
							
							// Name
							// Note: All variants must be simple enums associated with a value
							// Note: This value will not be used in the enum definition, instead
							//       it will be used for associating each variant with a value.
							$enum_variant_name:ident
							
							$( ($enum_variant_rename:literal) )?
							
							=>
							
							$enum_variant_value:expr,
						)*
						
						// Error
						_ => $error_name:ident($error_display:literal)
						
						$(,)?
					}
					
					// Any further definitions inside the module
					$( $extra_defs:tt )*
				}
			)+
		) =>
		{
			// Modules
			$(
				// The module
				$mod_vis mod $mod_name
				{
					// Types
					//--------------------------------------------------------------------------------------------------
						$( #[$enum_attr] )*
						#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
						#[derive(::serde::Serialize, ::serde::Deserialize)]
						$mod_vis enum $enum_name
						{
							$(
								$( #[$enum_variant_attr] )*
								
								$( #[serde(rename = $enum_variant_rename)] )?
								
								$enum_variant_name,
							)*
						}
						
						/// The error type thrown by `FromBytes`
						#[derive(Debug, ::derive_more::Display)]
						#[display(fmt = $error_display, byte)]
						$mod_vis struct $error_name {
							byte: u8,
						}
					//--------------------------------------------------------------------------------------------------
					
					// Impl
					//--------------------------------------------------------------------------------------------------
						generate_from_to_bytes!($enum_name, 1, $error_name, [
							$(
								$enum_variant_name => $enum_variant_value,
							)*
						]);
						
						impl ::std::error::Error for $error_name {
							// No source
						}
						
						// Extra definitions
						$( $extra_defs )*
					//--------------------------------------------------------------------------------------------------
				}
			)*
		}
	}
//--------------------------------------------------------------------------------------------------

// Modules
//--------------------------------------------------------------------------------------------------
	generate_enum_property_mod!(
		pub mod slot {
			/// A player's card slots
			enum Slot
			{
				Hand   => 0, Dp      => 1,
				Online => 2, Offline => 3,
				
				_ => UnknownSlot("Unknown byte 0x{:x} for a slot")
			}
		}
		
		pub mod arrow_color {
			/// A digimon effect's arrow color 
			enum ArrowColor
			{
				Red   => 1, Green => 2,
				Blue  => 3,
				
				_ => UnknownArrowColor("Unknown byte 0x{:x} for an arrow color")
			}
		}
		
		pub mod attack_type {
			/// A digimon's attack type
			enum AttackType
			{
				Circle => 0, Triangle => 1,
				Cross  => 2,
				
				_ => UnknownAttackType("Unknown byte 0x{:x} for an attack type")
			}
		}
		
		pub mod card_type {
			/// A card type
			enum CardType
			{
				Digimon   => 0,
				Item      => 1,
				Digivolve => 2,
				
				_ => UnknownCardType("Unknown byte 0x{:x} for a card type")
			}
			
			impl CardType
			{
				/// Returns the byte size of the corresponding card
				pub fn card_byte_size(self) -> usize
				{
					use crate::game::Bytes;
					
					match self
					{
						CardType::Digimon   => <crate::game::card::Digimon   as Bytes>::BUF_BYTE_SIZE,
						CardType::Item      => <crate::game::card::Item      as Bytes>::BUF_BYTE_SIZE,
						CardType::Digivolve => <crate::game::card::Digivolve as Bytes>::BUF_BYTE_SIZE,
					}
				}
			}
		}
		
		pub mod player_type {
			/// A player type
			enum PlayerType
			{
				Opponent => 0,
				Player   => 1,
				
				_ => UnknownPlayerType("Unknown byte 0x{:x} for a player type")
			}
		}
		
		pub mod level {
			/// A digimon's level
			enum Level
			{
				Rookie   => 0, Armor    => 1,
				Champion => 2, Ultimate => 3,
				
				_ => UnknownLevel("Unknown byte 0x{:x} for a level")
			}
		}
		
		pub mod speciality {
			/// A digimon's speciality
			enum Speciality
			{
				Fire      => 0, Ice       => 1,
				Nature    => 2, Darkness  => 3,
				Rare      => 4,
				
				_ => UnknownSpeciality("Unknown byte 0x{:x} for a speciality")
			}
		}
		
		pub mod support_effect_operation {
			/// A digimon's support effect operation
			enum SupportEffectOperation
			{
				Addition       => 0,
				Subtraction    => 1,
				Multiplication => 2,
				Division       => 3,
				
				_ => UnknownSupportEffectOperation("Unknown byte 0x{:x} for a support effect operation")
			}
		}
			
		pub mod support_condition_operation {
			/// A digimon's support condition operation
			/// 
			/// # Todo
			/// These don't seem to be 100% right, the less than property, sometimes does less than number, might be a range check
			enum SupportConditionOperation
			{
				LessThanProperty   ("Less than property"   ) => 0,
				LessThanNumber     ("Less than number"     ) => 1,
				MoreThanProperty   ("More than property"   ) => 2,
				MoreThanNumber     ("More than number"     ) => 3,
				DifferentFromNumber("Different from number") => 4,
				EqualToNumber      ("Equal to number"      ) => 5,
				
				_ => UnknownSupportConditionOperation("Unknown byte 0x{:x} for a support condition operation")
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
				
				Crash                => 8,
				EatUpHP("Eat Up HP") => 9,
				Jamming              => 10,
				
					FireFoe3x("Fire Foe x3"    ) => 11,
					 IceFoe3x("Ice Foe x3"     ) => 12,
				  NatureFoe3x("Nature Foe x3"  ) => 13,
				DarknessFoe3x("Darkness Foe x3") => 14,
					RareFoe3x("Rare Foe x3"    ) => 15,
				
				_ => UnknownCrossMoveEffect("Unknown byte 0x{:x} for a cross move effect")
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
				
				_ => UnknownDigimonProperty("Unknown byte 0x{:x} for a digimon property")
			}
		}
	);
	
	
	// Complex
	pub mod moves;
	pub mod support_effect;
	pub mod support_condition;
//--------------------------------------------------------------------------------------------------

// Exports
pub use level::Level;
pub use speciality::Speciality;
pub use moves::Move;
pub use cross_move_effect::CrossMoveEffect;

pub use digimon_property::DigimonProperty;

pub use support_effect::SupportEffect;
pub use support_effect_operation::SupportEffectOperation;

pub use support_condition::SupportCondition;
pub use support_condition_operation::SupportConditionOperation;

pub use card_type::CardType;
pub use arrow_color::ArrowColor;

pub use attack_type::AttackType;
pub use player_type::PlayerType;

pub use slot::Slot;
