//! Support effects

// Lints
#![allow(
	// We have a lot of `a, b, c, x, y` from the formulas,
	// but we can't change those names since they're the actual
	// names of the variables in the formulas
	clippy::many_single_char_names
)] 

// byteorder
use byteorder::{ByteOrder, LittleEndian};

// Crate
use crate::{
	game::{
		Bytes,
		card::property::{DigimonProperty, SupportEffectOperation, AttackType, PlayerType, Slot},
	},
};

// Types
//--------------------------------------------------------------------------------------------------
	/// A digimon's support effects
	#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
	#[derive(serde::Serialize, serde::Deserialize)]
	#[serde(tag = "type")]
	// TODO: Move this `allow` to the variant once clippy allows
	#[allow(clippy::pub_enum_variant_names)] // `Effect` on `VoidOpponentSupportEffect` isn't refering to the enum
	pub enum SupportEffect
	{
		/// Changes a property of either digimon
		/// 
		/// # Valid properties
		/// Only the following properties are valid for this effect:
		/// - `OwnSpeciality`     / `OpnSpeciality`    ,
		/// - `OwnHP`             / `OpnHP`            ,
		/// - `OwnCircleAttack`   / `OpnCircleAttack`  ,
		/// - `OwnTriangleAttack` / `OpnTriangleAttack`,
		/// - `OwnCrossAttack`    / `OpnCrossAttack`   ,
		/// - `OwnAttack`         / `OpnAttack`        ,
		/// - `OwnLevel`          / `OpnLevel`         ,
		/// 
		/// # Equation
		/// This variant uses the following equation
		/// to calculate the property:
		/// 
		/// `<property> = ( <A> + <Y> ) + ( <C> <op> ( <B> + <X> ) )`
		#[serde(rename = "Change property")]
		ChangeProperty {
			property: DigimonProperty,
			
			a: Option<DigimonProperty>,
			b: Option<DigimonProperty>,
			c: Option<DigimonProperty>,
			
			x: u16,
			y: u16,
			
			op: SupportEffectOperation,
		},
		
		/// A player uses an attack type
		#[serde(rename = "Use attack")]
		UseAttack {
			player: PlayerType,
			attack: AttackType,
		},
		
		/// Set the temp slot
		/// 
		/// # Equation
		/// This variant uses the following equation
		/// to calculate the property:
		/// 
		/// `<temp slot> = <A> + (<B> <op> <C>)`
		#[serde(rename = "Set temp slot")]
		SetTempSlot {
			a: Option<DigimonProperty>,
			b: Option<DigimonProperty>,
			c: Option<DigimonProperty>,
			
			op: SupportEffectOperation,
		},
		
		/// Moves cards from a slot to another
		/// 
		/// # Valid moves
		/// Only the following moves are valid for this effect, for both the player and opponent:
		/// - `Hand`    -> `Offline`
		/// - `Hand`    -> `Online`
		/// - `Online`  -> `Offline`
		/// - `Offline` -> `Online`
		/// - `Dp`      -> `Offline`
		#[serde(rename = "Move cards")]
		MoveCards {
			player     : PlayerType,
			source     : Slot,
			destination: Slot,
			
			count: u16,
		},
		
		/// Shuffles a player's online deck
		#[serde(rename = "Shuffle online deck")]
		ShuffleOnlineDeck {
			player: PlayerType,
		},
		
		/// Voids the opponent's support effect
		#[serde(rename = "Void opponent support effect")]
		VoidOpponentSupportEffect,
		
		/// Voids the opponent's support option effect
		#[serde(rename = "Void opponent support option effect")]
		VoidOpponentSupportOptionEffect,
		
		/// Picks the partner from the online deck and puts it onto the hand
		#[serde(rename = "Pick partner card")]
		PickPartnerCard,
		
		/// Cycles the opponent's attack types
		/// 
		/// # Order
		/// The order is the following:
		/// - `Circle` -> `Triangle`
		/// - `Triangle` -> `Cross`
		/// - `Cross` -> `Circle`
		#[serde(rename = "Cycle opponent attack type")]
		CycleOpponentAttackType,
		
		/// If the digimon is Ko'd it revives with health
		#[serde(rename = "Ko'd digimon revives")]
		KoDigimonRevives {
			health: u16,
		},
		
		/// A player draws cards
		#[serde(rename = "Draw cards")]
		DrawCards {
			player: PlayerType,
			
			count: u16,
		},
		
		/// Own attack becomes Eat Up HP
		#[serde(rename = "Own attack becomes Eat Up HP")]
		OwnAttackBecomesEatUpHP,
		
		/// A player attacks first
		#[serde(rename = "Attack first")]
		AttackFirst {
			player: PlayerType
		},
	}
	
	
	/// The error type thrown by `FromBytes`
	#[derive(Debug, derive_more::Display)]
	pub enum FromBytesError
	{
		/// An unknown effect type was found
		#[display(fmt = "Unknown byte for an effect type: {}", byte)]
		UnknownEffectType { byte: u8 },
		
		/// An unknown property argument was found
		#[display(fmt = "An unknown property was found for the {} property", rank)]
		PropertyArgument {
			rank: &'static str,
			
			
			err: crate::game::card::property::digimon_property::FromBytesError,
		},
		
		/// An unknown operation was found
		#[display(fmt = "An unknown operation was found")]
		Operation( crate::game::card::property::support_effect_operation::FromBytesError ),
		
		/// An unknown attack type was found
		#[display(fmt = "An unknown attack type was found")]
		AttackType( crate::game::card::property::attack_type::FromBytesError ),
	}
	
	impl std::error::Error for FromBytesError {
		fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
			match self {
				Self::UnknownEffectType{ .. } => None,
				Self::PropertyArgument{ err, ..} => Some(err),
				Self::Operation(err) => Some(err),
				Self::AttackType(err) => Some(err),
			}
		}
	}
//--------------------------------------------------------------------------------------------------

// Impl
//--------------------------------------------------------------------------------------------------
	impl Bytes for SupportEffect
	{
		const BUF_BYTE_SIZE : usize = 0x10;
		
		type FromError = FromBytesError;
		
		/// `bytes` should include the `exists` byte
		fn from_bytes(bytes: &[u8]) -> Result<Self, Self::FromError>
		{
			// Assert that we do exist
			assert_ne!(bytes[0x0], 0);
			
			// The effect type byte
			let effect_type_byte = bytes[0x1];
			
			// The properties
			let a = if bytes[0x2] != 0 {
				Some( DigimonProperty::from_bytes( &bytes[0x2..0x3] )  .map_err(|err| FromBytesError::PropertyArgument{ rank: "1st", err })? )
			} else { None };
			
			let b = if bytes[0x4] != 0 {
				Some( DigimonProperty::from_bytes( &bytes[0x4..0x5] )  .map_err(|err| FromBytesError::PropertyArgument{ rank: "2nd", err })? )
			} else { None };
			
			let c = if bytes[0x6] != 0 {
				Some( DigimonProperty::from_bytes( &bytes[0x6..0x7] )  .map_err(|err| FromBytesError::PropertyArgument{ rank: "3rd", err })? )
			} else { None };
			
			// The numbers
			let x = LittleEndian::read_u16( &bytes[0xa..0xc] );
			let y = LittleEndian::read_u16( &bytes[0xc..0xe] );
			
			// The operation
			let op = SupportEffectOperation::from_bytes( &bytes[0xf..0x10] )  .map_err(FromBytesError::Operation)?;
			
			// Check what the effect type is
			match effect_type_byte
			{
				0..=13 => {
					Ok( Self::ChangeProperty {
						// Note: unwrapping is fine here because we know that `effect_type_byte+1` is between 1 and 14 inclusive
						property: DigimonProperty::from_bytes( &[ effect_type_byte+1 ] )
							.expect("Unable to get digimon property from bytes"),
						a, b, c, x, y, op,
					})
				},
				
				// Take lower byte from `x` for these
				16 => { Ok( Self::UseAttack{ player: PlayerType::Player  , attack: AttackType::from_bytes( &[x.to_le_bytes()[0]] )  .map_err(FromBytesError::AttackType)? } ) },
				17 => { Ok( Self::UseAttack{ player: PlayerType::Opponent, attack: AttackType::from_bytes( &[x.to_le_bytes()[0]] )  .map_err(FromBytesError::AttackType)? } ) },
				
				
				25 => { Ok( Self::SetTempSlot{ a, b, c, op } ) },
				
				26 => { Ok( Self::MoveCards{ player: PlayerType::Player  , source: Slot::Hand   , destination: Slot::Offline, count: y } ) },
				27 => { Ok( Self::MoveCards{ player: PlayerType::Opponent, source: Slot::Hand   , destination: Slot::Offline, count: y } ) },
				
				30 => { Ok( Self::MoveCards{ player: PlayerType::Player  , source: Slot::Hand   , destination: Slot::Online , count: y } ) },
				31 => { Ok( Self::MoveCards{ player: PlayerType::Opponent, source: Slot::Hand   , destination: Slot::Online , count: y } ) },
				
				32 => { Ok( Self::MoveCards{ player: PlayerType::Player  , source: Slot::Online , destination: Slot::Offline, count: y } ) },
				33 => { Ok( Self::MoveCards{ player: PlayerType::Opponent, source: Slot::Online , destination: Slot::Offline, count: y } ) },
				
				34 => { Ok( Self::MoveCards{ player: PlayerType::Player  , source: Slot::Offline, destination: Slot::Online , count: y } ) },
				35 => { Ok( Self::MoveCards{ player: PlayerType::Opponent, source: Slot::Offline, destination: Slot::Online , count: y } ) },
				
				36 => { Ok( Self::MoveCards{ player: PlayerType::Player  , source: Slot::Dp     , destination: Slot::Offline, count: y } ) },
				37 => { Ok( Self::MoveCards{ player: PlayerType::Opponent, source: Slot::Dp     , destination: Slot::Offline, count: y } ) },
				
				
				42 => { Ok( Self::ShuffleOnlineDeck{ player: PlayerType::Player   } ) },
				43 => { Ok( Self::ShuffleOnlineDeck{ player: PlayerType::Opponent } ) },
				
				44 => { Ok( Self::VoidOpponentSupportEffect       ) },
				45 => { Ok( Self::VoidOpponentSupportOptionEffect ) },
				
				46 => { Ok( Self::PickPartnerCard ) },
				
				47 => { Ok( Self::CycleOpponentAttackType ) },
				
				48 => { Ok( Self::KoDigimonRevives{ health: y } ) },
				
				49 => { Ok( Self::DrawCards{ player: PlayerType::Player  , count: y } ) },
				50 => { Ok( Self::DrawCards{ player: PlayerType::Opponent, count: y } ) },
				
				51 => { Ok( Self::OwnAttackBecomesEatUpHP ) },
				
				52 => { Ok( Self::AttackFirst{ player: PlayerType::Player   } ) },
				53 => { Ok( Self::AttackFirst{ player: PlayerType::Opponent } ) },
				
				_ => Err( FromBytesError::UnknownEffectType{ byte: effect_type_byte } ),
			}
		}
		
		type ToError = !;
		fn to_bytes(&self, _bytes: &mut [u8]) -> Result<(), Self::ToError>
		{
			// Match which effect we are
			todo!()
		}
	}
//--------------------------------------------------------------------------------------------------
