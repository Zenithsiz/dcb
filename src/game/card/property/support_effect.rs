//! A digimon's support effect
//! 
//! This module contains the [`SupportEffect`] struct, which describes a support effect.
//! 
//! # Layout
//! Each support effect has a size of `0x10` bytes, and it's general layout is the following:
//! 
//! | Offset | Size | Type                 | Name                      | Location               | Details                                                       |
//! |--------|------|----------------------|---------------------------|------------------------|---------------------------------------------------------------|
//! | 0x0    | 0x1  | `bool`               | Exists                    | N/A                    | If `0`, the effect does not exist                             |
//! | 0x1    | 0x1  | N/A                  | Effect Type               | N/A                    | Determines which [`SupportEffect`] variant is used.           |
//! | 0x2    | 0xe  | N/A                  | Arguments                 | N/A                    | The arguments used for the current [`SupportEffect`] variant. |

// byteorder
use byteorder::{ByteOrder, LittleEndian};

// Crate
use crate::game::{
	Bytes,
	util,
	card::property::{
		self, DigimonProperty, SupportEffectOperation, AttackType, PlayerType, Slot
	},
};

/// A digimon's support effects
/// 
/// As this type is wildly volatile in which arguments it uses and from where,
/// it is an `enum` with struct variants instead of a struct. This simplifices argument
/// verification and, from a language perspective, makes more sense as an implementation.
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

/// Error type for [`Bytes::from_bytes`]
#[derive(Debug)]
#[derive(derive_more::Display, err_impl::Error)]
pub enum FromBytesError
{
	/// Unknown property for first property argument
	#[display(fmt = "Unknown property for first property argument")]
	FirstProperty( #[error(source)] property::digimon_property::FromBytesError ),
	
	/// Unknown property for second property argument
	#[display(fmt = "Unknown property for second property argument")]
	SecondProperty( #[error(source)] property::digimon_property::FromBytesError ),
	
	/// Unknown property for third property argument
	#[display(fmt = "Unknown property for third property argument")]
	ThirdProperty( #[error(source)] property::digimon_property::FromBytesError ),
	
	/// Unknown operation argument
	#[display(fmt = "Unknown operation argument")]
	Operation( #[error(source)] property::support_effect_operation::FromBytesError ),
	
	/// Unknown attack type for [`SupportEffect::UseAttack`]
	#[display(fmt = "Unknown attack type")]
	UseAttackAttackType( #[error(source)] property::attack_type::FromBytesError ),
	
	/// Unknown effect type
	#[display(fmt = "Unknown byte for an effect type: {}", "byte")]
	EffectType { byte: u8 },
}

impl Bytes for SupportEffect
{
	type ByteArray = [u8; 0x10];
	
	type FromError = FromBytesError;
	
	/// `bytes` should include the `exists` byte
	fn from_bytes(bytes: &Self::ByteArray) -> Result<Self, Self::FromError>
	{
		// Utility uses
		use PlayerType::{Player, Opponent};
		use Slot::{Hand, Online as OnlineDeck, Offline as OfflineDeck, Dp as DpSlot};
		
		// The effect type byte
		let effect_type_byte = bytes[0x1];
		
		// The property argument getters
		let get_a = || (bytes[0x2] != 0)
			.then(|| DigimonProperty::from_bytes( &bytes[0x2] ))
			.transpose()
			.map_err(FromBytesError::FirstProperty);
		let get_b = || (bytes[0x4] != 0)
			.then(|| DigimonProperty::from_bytes( &bytes[0x4] ))
			.transpose()
			.map_err(FromBytesError::SecondProperty);
		let get_c = || (bytes[0x6] != 0)
			.then(|| DigimonProperty::from_bytes( &bytes[0x6] ))
			.transpose()
			.map_err(FromBytesError::ThirdProperty);
		let get_attack_type = || AttackType::from_bytes( &bytes[0xa] ) // Lower byte of `x`
			.map_err(FromBytesError::UseAttackAttackType);
		
		// The number arguments
		let x = LittleEndian::read_u16( &bytes[0xa..0xc] );
		let y = LittleEndian::read_u16( &bytes[0xc..0xe] );
		
		// The operation argument
		let op = SupportEffectOperation::from_bytes( &bytes[0xf] )
			.map_err(FromBytesError::Operation)?;
		
		// Check what the effect type is
		match effect_type_byte
		{
			0..=13 => Ok( Self::ChangeProperty {
				// Note: unwrapping is fine here because we know that `effect_type_byte+1` is between 1 and 14 inclusive
				property: DigimonProperty::from_bytes( &(effect_type_byte+1) )
					.expect("Unable to get digimon property from bytes"),
				a: get_a()?, b: get_b()?, c: get_c()?, x, y, op,
			}),
			
			
			// Lower byte of `x` contains the attack type
			16 => Ok( Self::UseAttack{ player: Player  , attack: get_attack_type()? }),
			17 => Ok( Self::UseAttack{ player: Opponent, attack: get_attack_type()? }),
			
			
			25 => Ok( Self::SetTempSlot{ a: get_a()?, b: get_b()?, c: get_c()?, op } ),
			
			26 => Ok( Self::MoveCards{ player: Player  , source: Hand, destination: OfflineDeck, count: y } ),
			27 => Ok( Self::MoveCards{ player: Opponent, source: Hand, destination: OfflineDeck, count: y } ),
			
			30 => Ok( Self::MoveCards{ player: Player  , source: Hand, destination: OnlineDeck, count: y } ),
			31 => Ok( Self::MoveCards{ player: Opponent, source: Hand, destination: OnlineDeck, count: y } ),
			
			32 => Ok( Self::MoveCards{ player: Player  , source: OnlineDeck, destination: OfflineDeck, count: y } ),
			33 => Ok( Self::MoveCards{ player: Opponent, source: OnlineDeck, destination: OfflineDeck, count: y } ),
			
			34 => Ok( Self::MoveCards{ player: Player  , source: OfflineDeck, destination: OnlineDeck, count: y } ),
			35 => Ok( Self::MoveCards{ player: Opponent, source: OfflineDeck, destination: OnlineDeck, count: y } ),
			
			36 => Ok( Self::MoveCards{ player: Player  , source: DpSlot, destination: OfflineDeck, count: y } ),
			37 => Ok( Self::MoveCards{ player: Opponent, source: DpSlot, destination: OfflineDeck, count: y } ),
			
			
			42 => Ok( Self::ShuffleOnlineDeck{ player: Player   } ),
			43 => Ok( Self::ShuffleOnlineDeck{ player: Opponent } ),
			
			44 => Ok( Self::VoidOpponentSupportEffect       ),
			45 => Ok( Self::VoidOpponentSupportOptionEffect ),
			
			46 => Ok( Self::PickPartnerCard ),
			
			47 => Ok( Self::CycleOpponentAttackType ),
			
			48 => Ok( Self::KoDigimonRevives{ health: y } ),
			
			49 => Ok( Self::DrawCards{ player: Player  , count: y } ),
			50 => Ok( Self::DrawCards{ player: Opponent, count: y } ),
			
			51 => Ok( Self::OwnAttackBecomesEatUpHP ),
			
			52 => Ok( Self::AttackFirst{ player: Player   } ),
			53 => Ok( Self::AttackFirst{ player: Opponent } ),
			
			_ => Err( FromBytesError::EffectType{ byte: effect_type_byte } ),
		}
	}
	
	type ToError = !;
	fn to_bytes(&self, bytes: &mut Self::ByteArray) -> Result<(), Self::ToError>
	{
		// Get all byte arrays we need
		let bytes = util::array_split_mut!(bytes,
			exists     : 0x1,
			effect_type: 0x1,
			a          : 0x1,
			_unknown_3 : 0x1,
			b          : 0x1,
			_unknown_5 : 0x1,
			c          : 0x1,
			_unknown_7 : [0x3],
			x          : [0x2],
			y          : [0x2],
			_unknown_e : 0x1,
			op         : 0x1,
		);
		
		// Set that the effect exists
		*bytes.exists = 1;
		
		// Check our variant and fill `bytes` with info
		#[allow(clippy::unneeded_field_pattern)] // Placeholder
		match self {
			Self::ChangeProperty { property, a, b, c, x, y, op } => {
				property.to_bytes(bytes.effect_type)?;
				*bytes.effect_type -= 1;
				if let Some(a) = a { a.to_bytes(bytes.a)?; }
				if let Some(b) = b { b.to_bytes(bytes.b)?; }
				if let Some(c) = c { c.to_bytes(bytes.c)?; }
				LittleEndian::write_u16(bytes.x, *x);
				LittleEndian::write_u16(bytes.y, *y);
				op.to_bytes(bytes.op)?;
			},
			
			Self::UseAttack { player: _, attack: _ } => todo!(),
			
			Self::SetTempSlot { a: _, b: _, c: _, op: _ } => todo!(),
			
			Self::MoveCards { player: _, source: _, destination: _, count: _ } => todo!(),
			
			Self::ShuffleOnlineDeck { player: _ } => todo!(),
			
			Self::VoidOpponentSupportEffect => todo!(),
			
			Self::VoidOpponentSupportOptionEffect => todo!(),
			
			Self::PickPartnerCard => todo!(),
			
			Self::CycleOpponentAttackType => todo!(),
			
			Self::KoDigimonRevives { health: _ } => todo!(),
			
			Self::DrawCards { player: _, count: _ } => todo!(),
			
			Self::OwnAttackBecomesEatUpHP => todo!(),
			
			Self::AttackFirst { player: _ } => todo!(),
		}
		
		// And return Ok
		Ok(())
	}
}
