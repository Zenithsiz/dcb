//! A digimon's support effect
//! 
//! This module contains the [`Effect`] struct, which describes a support effect.
//! 
//! # Layout
//! Each support effect has a size of `0x10` bytes, and it's general layout is the following:
//! 
//! | Offset | Size | Type                 | Name                      | Location               | Details                                                |
//! |--------|------|----------------------|---------------------------|------------------------|--------------------------------------------------------|
//! | 0x0    | 0x1  | `bool`               | Exists                    | N/A                    | If `0`, the effect does not exist                      |
//! | 0x1    | 0x1  | N/A                  | Effect Type               | N/A                    | Determines which [`Effect`] variant is used.           |
//! | 0x2    | 0xe  | N/A                  | Arguments                 | N/A                    | The arguments used for the current [`Effect`] variant. |

// byteorder
use byteorder::{ByteOrder, LittleEndian};

// Crate
use crate::game::{
	Bytes,
	util,
	card::property::{
		self, DigimonProperty, EffectOperation, AttackType, PlayerType, Slot
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
pub enum Effect
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
		
		op: EffectOperation,
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
		
		op: EffectOperation,
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
	Operation( #[error(source)] property::effect_operation::FromBytesError ),
	
	/// Unknown attack type for [`Effect::UseAttack`]
	#[display(fmt = "Unknown attack type")]
	UseAttackAttackType( #[error(source)] property::attack_type::FromBytesError ),
	
	/// Unknown effect type
	#[display(fmt = "Unknown byte for an effect type: {}", "byte")]
	EffectType { byte: u8 },
}

/// Error type for [`Bytes::from_bytes`]
#[derive(Debug)]
#[derive(derive_more::Display, err_impl::Error)]
pub enum ToBytesError
{
	/// Invalid move [`Effect::MoveCards`] effect
	#[display(fmt = "Invalid move cards effect ({} => {})", source, destination)]
	InvalidMoveCards {
		source     : Slot,
		destination: Slot,
	}
}

#[allow(clippy::use_self)] // False positive
impl Bytes for Option<Effect>
{
	type ByteArray = [u8; 0x10];
	
	type FromError = FromBytesError;
	
	/// `bytes` should include the `exists` byte
	fn from_bytes(bytes: &Self::ByteArray) -> Result<Self, Self::FromError>
	{
		// Utility uses
		use PlayerType::{Player, Opponent};
		use Slot::{Hand, Online as OnlineDeck, Offline as OfflineDeck, Dp as DpSlot};
		
		// Get all byte arrays we need
		let bytes = util::array_split!(bytes,
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
		
		// If the exists byte is 0, return None
		if *bytes.exists == 0 {
			return Ok(None);
		}
		
		// Else create getters for all arguments
		let get_a = || (*bytes.a != 0)
			.then(|| DigimonProperty::from_bytes(bytes.a))
			.transpose()
			.map_err(FromBytesError::FirstProperty);
		let get_b = || (*bytes.b != 0)
			.then(|| DigimonProperty::from_bytes(bytes.b))
			.transpose()
			.map_err(FromBytesError::SecondProperty);
		let get_c = || (*bytes.c != 0)
			.then(|| DigimonProperty::from_bytes(bytes.c))
			.transpose()
			.map_err(FromBytesError::ThirdProperty);
		
		// The number arguments
		let x = LittleEndian::read_u16( bytes.x );
		let y = LittleEndian::read_u16( bytes.y );
		
		// Attack type
		// Lower byte of `x`
		let get_attack_type = || AttackType::from_bytes( &x.to_le_bytes()[0] )
			.map_err(FromBytesError::UseAttackAttackType);
		
		// The operation argument
		let get_op = || EffectOperation::from_bytes( bytes.op )
			.map_err(FromBytesError::Operation);
		
		// And check what the effect type is
		let effect = match bytes.effect_type
		{
			0..=13 => Effect::ChangeProperty {
				// Note: unwrapping is fine here because we know that `effect_type_byte+1` is between 1 and 14 inclusive
				property: DigimonProperty::from_bytes( &(bytes.effect_type+1) )
					.expect("Unable to get digimon property from bytes"),
				a: get_a()?, b: get_b()?, c: get_c()?, x, y, op: get_op()?,
			},
			
			
			16 => Effect::UseAttack{ player: Player  , attack: get_attack_type()? },
			17 => Effect::UseAttack{ player: Opponent, attack: get_attack_type()? },
			
			
			25 => Effect::SetTempSlot{ a: get_a()?, b: get_b()?, c: get_c()?, op: get_op()? },
			
			26 => Effect::MoveCards{ player: Player  , source: Hand, destination: OfflineDeck, count: y },
			27 => Effect::MoveCards{ player: Opponent, source: Hand, destination: OfflineDeck, count: y },
			
			30 => Effect::MoveCards{ player: Player  , source: Hand, destination: OnlineDeck, count: y },
			31 => Effect::MoveCards{ player: Opponent, source: Hand, destination: OnlineDeck, count: y },
			
			32 => Effect::MoveCards{ player: Player  , source: OnlineDeck, destination: OfflineDeck, count: y },
			33 => Effect::MoveCards{ player: Opponent, source: OnlineDeck, destination: OfflineDeck, count: y },
			
			34 => Effect::MoveCards{ player: Player  , source: OfflineDeck, destination: OnlineDeck, count: y },
			35 => Effect::MoveCards{ player: Opponent, source: OfflineDeck, destination: OnlineDeck, count: y },
			
			36 => Effect::MoveCards{ player: Player  , source: DpSlot, destination: OfflineDeck, count: y },
			37 => Effect::MoveCards{ player: Opponent, source: DpSlot, destination: OfflineDeck, count: y },
			
			
			42 => Effect::ShuffleOnlineDeck{ player: Player   },
			43 => Effect::ShuffleOnlineDeck{ player: Opponent },
			
			44 => Effect::VoidOpponentSupportEffect,
			45 => Effect::VoidOpponentSupportOptionEffect,
			
			46 => Effect::PickPartnerCard,
			
			47 => Effect::CycleOpponentAttackType,
			
			48 => Effect::KoDigimonRevives{ health: y },
			
			49 => Effect::DrawCards{ player: Player  , count: y },
			50 => Effect::DrawCards{ player: Opponent, count: y },
			
			51 => Effect::OwnAttackBecomesEatUpHP,
			
			52 => Effect::AttackFirst{ player: Player   },
			53 => Effect::AttackFirst{ player: Opponent },
			
			&byte => return Err( FromBytesError::EffectType { byte } ),
		};
		
		// And return the effect
		Ok( Some(effect) )
	}
	
	type ToError = ToBytesError;
	#[allow(clippy::too_many_lines)] // It's a single match, we can't really split it
	fn to_bytes(&self, bytes: &mut Self::ByteArray) -> Result<(), Self::ToError>
	{
		// Utility uses
		use PlayerType::{Player, Opponent};
		use Slot::{Hand, Online as OnlineDeck, Offline as OfflineDeck, Dp as DpSlot};
		
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
		
		// Try to get the effect, if it doesn't exist, zero the exists byte and return
		let effect = match self {
			Some(effect) => effect,
			None => {
				*bytes.exists = 0;
				return Ok(());
			}
		};
		
		// Else set that the effect exists
		*bytes.exists = 1;
		
		// Setters
		let bytes_a = bytes.a;
		let bytes_b = bytes.b;
		let bytes_c = bytes.c;
		let mut set_a = |a: &Option<DigimonProperty>| if let Some(a) = a {
			a.to_bytes(bytes_a).into_ok();
		} else {
			*bytes_a = 0;
		};
		let mut set_b = |b: &Option<DigimonProperty>| if let Some(b) = b {
			b.to_bytes(bytes_b).into_ok();
		} else {
			*bytes_b = 0;
		};
		let mut set_c = |c: &Option<DigimonProperty>| if let Some(c) = c {
			c.to_bytes(bytes_c).into_ok();
		} else {
			*bytes_c = 0;
		};
		let bytes_attack_type = &mut bytes.x[0];
		let mut set_attack_type = |attack: &AttackType| attack.to_bytes( bytes_attack_type ).into_ok();
		
		// Check our variant and fill `bytes` with info
		#[allow(clippy::unneeded_field_pattern)] // Placeholder
		match effect {
			Effect::ChangeProperty { property, a, b, c, x, y, op } => {
				// Write the property minus one
				property.to_bytes(bytes.effect_type).into_ok();
				*bytes.effect_type -= 1;
				
				// Write all arguments
				set_a(a);
				set_b(b);
				set_c(c);
				LittleEndian::write_u16(bytes.x, *x);
				LittleEndian::write_u16(bytes.y, *y);
				op.to_bytes(bytes.op).into_ok();
			},
			
			Effect::UseAttack { player, attack } => {
				*bytes.effect_type = match player {
					Player   => 16,
					Opponent => 17,
				};
				set_attack_type(attack);
			},
			
			Effect::SetTempSlot { a, b, c, op } => {
				*bytes.effect_type = 25;
				set_a(a);
				set_b(b);
				set_c(c);
				op.to_bytes(bytes.op).into_ok();
			}
			
			Effect::MoveCards { player, source, destination, count } => {
				*bytes.effect_type = match (player, source, destination) {
					(Player  , Hand, OfflineDeck) => 26,
					(Opponent, Hand, OfflineDeck) => 27,
					
					(Player  , Hand, OnlineDeck) => 30,
					(Opponent, Hand, OnlineDeck) => 31,
					
					(Player  , OnlineDeck, OfflineDeck) => 32,
					(Opponent, OnlineDeck, OfflineDeck) => 33,
					
					(Player  , OfflineDeck, OnlineDeck) => 34,
					(Opponent, OfflineDeck, OnlineDeck) => 35,
					
					(Player  , DpSlot, OfflineDeck) => 36,
					(Opponent, DpSlot, OfflineDeck) => 37,
					
					(_, &source, &destination) => return Err(ToBytesError::InvalidMoveCards { source, destination }),
				};
				LittleEndian::write_u16(bytes.y, *count);
			}
			
			Effect::ShuffleOnlineDeck { player } => *bytes.effect_type = match player {
				Player   => 42,
				Opponent => 43,
			},
			
			Effect::VoidOpponentSupportEffect       => *bytes.effect_type = 42,
			Effect::VoidOpponentSupportOptionEffect => *bytes.effect_type = 43,
			
			Effect::PickPartnerCard => *bytes.effect_type = 46,
			
			Effect::CycleOpponentAttackType => *bytes.effect_type = 47,
			
			Effect::KoDigimonRevives { health } => {
				LittleEndian::write_u16(bytes.y, *health);
			},
			
			Effect::DrawCards { player, count } => {
				*bytes.effect_type = match player {
					Player   => 49,
					Opponent => 50,
				};
				LittleEndian::write_u16(bytes.y, *count);
			}
			
			Effect::OwnAttackBecomesEatUpHP => *bytes.effect_type = 51,
			
			Effect::AttackFirst { player } => *bytes.effect_type = match player {
				Player   => 52,
				Opponent => 53,
			},
		}
		
		// And return Ok
		Ok(())
	}
}
