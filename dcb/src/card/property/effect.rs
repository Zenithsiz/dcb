#![doc(include = "effect.md")]

// Imports
use crate::card::property::{self, AttackType, DigimonProperty, EffectOperation, PlayerType, Slot};
use byteorder::{ByteOrder, LittleEndian};
use dcb_bytes::Bytes;
use dcb_util::{array_split, array_split_mut};

/// A digimon's support effects
///
/// As this type is wildly volatile in which arguments it uses and from where,
/// it is an `enum` with struct variants instead of a struct. This simplifies argument
/// verification and, from a language perspective, makes more sense as an implementation.
#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
// TODO: Move this `allow` to the variant once clippy allows
#[allow(clippy::pub_enum_variant_names)] // `Effect` on `VoidOpponentSupportEffect` isn't referring to the enum
pub enum Effect {
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
	#[allow(clippy::missing_docs_in_private_items)] // Explained the formula
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
		/// Player being forced to use `attack`
		player: PlayerType,

		/// Attack being forced to be used
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
	#[allow(clippy::missing_docs_in_private_items)] // Explained the formula
	SetTempSlot {
		a:  Option<DigimonProperty>,
		b:  Option<DigimonProperty>,
		c:  Option<DigimonProperty>,
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
		/// Which player has their cards moved
		player: PlayerType,

		/// Source slot
		source: Slot,

		/// Destination slot
		destination: Slot,

		/// Number of cards
		count: u16,
	},

	/// Shuffles a player's online deck
	#[serde(rename = "Shuffle online deck")]
	ShuffleOnlineDeck {
		/// Player to shuffle the deck of
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
		/// Health to revive with
		health: u16,
	},

	/// A player draws cards
	#[serde(rename = "Draw cards")]
	DrawCards {
		/// Player drawing the cards
		player: PlayerType,

		/// How many cards to draw
		count: u16,
	},

	/// Own attack becomes Eat Up HP
	#[serde(rename = "Own attack becomes Eat Up HP")]
	OwnAttackBecomesEatUpHP,

	/// A player attacks first
	#[serde(rename = "Attack first")]
	AttackFirst {
		/// Player attacking first
		player: PlayerType,
	},
}

/// Error type for [`Bytes::from_bytes`](dcb_bytes::Bytes::from_bytes)
#[derive(PartialEq, Eq, Clone, Copy, Debug, thiserror::Error)]
pub enum FromBytesError {
	/// Unknown property for first property argument
	#[error("Unknown property for first property argument")]
	FirstProperty(#[source] property::digimon_property::FromBytesError),

	/// Unknown property for second property argument
	#[error("Unknown property for second property argument")]
	SecondProperty(#[source] property::digimon_property::FromBytesError),

	/// Unknown property for third property argument
	#[error("Unknown property for third property argument")]
	ThirdProperty(#[source] property::digimon_property::FromBytesError),

	/// Unknown operation argument
	#[error("Unknown operation argument")]
	Operation(#[source] property::effect_operation::FromBytesError),

	/// Unknown attack type for [`Effect::UseAttack`]
	#[error("Unknown attack type")]
	UseAttackAttackType(#[source] property::attack_type::FromBytesError),

	/// Unknown effect type
	#[error("Unknown byte for an effect type: {}", byte)]
	EffectType {
		/// Unknown byte
		byte: u8,
	},
}

/// Error type for [`Bytes::from_bytes`](dcb_bytes::Bytes::from_bytes)
#[derive(PartialEq, Eq, Clone, Copy, Debug, thiserror::Error)]
pub enum ToBytesError {
	/// Invalid move [`Effect::MoveCards`] effect
	#[error("Invalid move cards effect ({} => {})", from, to)]
	InvalidMoveCards {
		/// Slot we tried to move from
		from: Slot,

		/// Slot we tried to move to
		to: Slot,
	},
}

impl Bytes for Effect {
	type ByteArray = [u8; 0xf];
	type FromError = FromBytesError;
	type ToError = ToBytesError;

	fn from_bytes(bytes: &Self::ByteArray) -> Result<Self, Self::FromError> {
		// Utility uses
		use PlayerType::{Opponent, Player};
		use Slot::{Dp as DpSlot, Hand, Offline as OfflineDeck, Online as OnlineDeck};

		// Get all byte arrays we need
		let bytes = array_split!(bytes,
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

		// Else create getters for all arguments
		let get_a = || {
			(*bytes.a != 0)
				.then(|| DigimonProperty::from_bytes(bytes.a))
				.transpose()
				.map_err(FromBytesError::FirstProperty)
		};
		let get_b = || {
			(*bytes.b != 0)
				.then(|| DigimonProperty::from_bytes(bytes.b))
				.transpose()
				.map_err(FromBytesError::SecondProperty)
		};
		let get_c = || {
			(*bytes.c != 0)
				.then(|| DigimonProperty::from_bytes(bytes.c))
				.transpose()
				.map_err(FromBytesError::ThirdProperty)
		};

		// The number arguments
		let x = LittleEndian::read_u16(bytes.x);
		let y = LittleEndian::read_u16(bytes.y);

		// Attack type
		// Lower byte of `x`
		let get_attack_type = || AttackType::from_bytes(&x.to_le_bytes()[0]).map_err(FromBytesError::UseAttackAttackType);

		// The operation argument
		let get_op = || EffectOperation::from_bytes(bytes.op).map_err(FromBytesError::Operation);

		// And check what the effect type is
		#[rustfmt::skip]
		let effect = match bytes.effect_type {
			0..=13 => Self::ChangeProperty {
				// Note: unwrapping is fine here because we know that `effect_type_byte+1` is between 1 and 14 inclusive
				property: DigimonProperty::from_bytes( &(bytes.effect_type+1) )
					.expect("Unable to get digimon property from bytes"),
				a: get_a()?, b: get_b()?, c: get_c()?, x, y, op: get_op()?,
			},

			16 => Self::UseAttack{ player: Player  , attack: get_attack_type()? },
			17 => Self::UseAttack{ player: Opponent, attack: get_attack_type()? },

			25 => Self::SetTempSlot{ a: get_a()?, b: get_b()?, c: get_c()?, op: get_op()? },

			26 => Self::MoveCards{ player: Player  , source: Hand, destination: OfflineDeck, count: y },
			27 => Self::MoveCards{ player: Opponent, source: Hand, destination: OfflineDeck, count: y },

			30 => Self::MoveCards{ player: Player  , source: Hand, destination: OnlineDeck, count: y },
			31 => Self::MoveCards{ player: Opponent, source: Hand, destination: OnlineDeck, count: y },

			32 => Self::MoveCards{ player: Player  , source: OnlineDeck, destination: OfflineDeck, count: y },
			33 => Self::MoveCards{ player: Opponent, source: OnlineDeck, destination: OfflineDeck, count: y },

			34 => Self::MoveCards{ player: Player  , source: OfflineDeck, destination: OnlineDeck, count: y },
			35 => Self::MoveCards{ player: Opponent, source: OfflineDeck, destination: OnlineDeck, count: y },

			36 => Self::MoveCards{ player: Player  , source: DpSlot, destination: OfflineDeck, count: y },
			37 => Self::MoveCards{ player: Opponent, source: DpSlot, destination: OfflineDeck, count: y },

			42 => Self::ShuffleOnlineDeck{ player: Player   },
			43 => Self::ShuffleOnlineDeck{ player: Opponent },

			44 => Self::VoidOpponentSupportEffect,
			45 => Self::VoidOpponentSupportOptionEffect,

			46 => Self::PickPartnerCard,

			47 => Self::CycleOpponentAttackType,

			48 => Self::KoDigimonRevives{ health: y },

			49 => Self::DrawCards{ player: Player  , count: y },
			50 => Self::DrawCards{ player: Opponent, count: y },

			51 => Self::OwnAttackBecomesEatUpHP,

			52 => Self::AttackFirst{ player: Player   },
			53 => Self::AttackFirst{ player: Opponent },

			&byte => return Err( FromBytesError::EffectType { byte } ),
		};

		// And return the effect
		Ok(effect)
	}

	#[allow(clippy::too_many_lines)] // It's a single match, we can't really split it
	fn to_bytes(&self, bytes: &mut Self::ByteArray) -> Result<(), Self::ToError> {
		// Utility uses
		use PlayerType::{Opponent, Player};
		use Slot::{Dp as DpSlot, Hand, Offline as OfflineDeck, Online as OnlineDeck};

		// Get all byte arrays we need
		let bytes = array_split_mut!(bytes,
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

		// Setters
		let bytes_a = bytes.a;
		let bytes_b = bytes.b;
		let bytes_c = bytes.c;
		let mut set_a = |a: &Option<DigimonProperty>| {
			if let Some(a) = a {
				a.to_bytes(bytes_a).into_ok();
			} else {
				*bytes_a = 0;
			}
		};
		let mut set_b = |b: &Option<DigimonProperty>| {
			if let Some(b) = b {
				b.to_bytes(bytes_b).into_ok();
			} else {
				*bytes_b = 0;
			}
		};
		let mut set_c = |c: &Option<DigimonProperty>| {
			if let Some(c) = c {
				c.to_bytes(bytes_c).into_ok();
			} else {
				*bytes_c = 0;
			}
		};
		let bytes_attack_type = &mut bytes.x[0];
		let mut set_attack_type = |attack: &AttackType| attack.to_bytes(bytes_attack_type).into_ok();

		// Check our variant and fill `bytes` with info
		#[rustfmt::skip]
		match self {
			Self::ChangeProperty { property, a, b, c, x, y, op } => {
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

			Self::UseAttack { player, attack } => {
				*bytes.effect_type = match player {
					Player   => 16,
					Opponent => 17,
				};
				set_attack_type(attack);
			},

			Self::SetTempSlot { a, b, c, op } => {
				*bytes.effect_type = 25;
				set_a(a);
				set_b(b);
				set_c(c);
				op.to_bytes(bytes.op).into_ok();
			}

			Self::MoveCards { player, source, destination, count } => {
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

					(_, &from, &to) => return Err( ToBytesError::InvalidMoveCards { from, to } ),
				};
				LittleEndian::write_u16(bytes.y, *count);
			}

			Self::ShuffleOnlineDeck { player } => *bytes.effect_type = match player {
				Player   => 42,
				Opponent => 43,
			},

			Self::VoidOpponentSupportEffect       => *bytes.effect_type = 42,
			Self::VoidOpponentSupportOptionEffect => *bytes.effect_type = 43,

			Self::PickPartnerCard => *bytes.effect_type = 46,

			Self::CycleOpponentAttackType => *bytes.effect_type = 47,

			Self::KoDigimonRevives { health } => {
				LittleEndian::write_u16(bytes.y, *health);
			},

			Self::DrawCards { player, count } => {
				*bytes.effect_type = match player {
					Player   => 49,
					Opponent => 50,
				};
				LittleEndian::write_u16(bytes.y, *count);
			}

			Self::OwnAttackBecomesEatUpHP => *bytes.effect_type = 51,

			Self::AttackFirst { player } => *bytes.effect_type = match player {
				Player   => 52,
				Opponent => 53,
			},
		}

		// And return Ok
		Ok(())
	}
}

/// A possible effect
#[repr(transparent)]
#[derive(ref_cast::RefCast)]
#[derive(derive_more::From, derive_more::Into)]
pub struct MaybeEffect(Option<Effect>);

impl Bytes for MaybeEffect {
	type ByteArray = [u8; 0x10];
	type FromError = FromBytesError;
	type ToError = ToBytesError;

	// `bytes` should include the `exists` byte
	fn from_bytes(bytes: &Self::ByteArray) -> Result<Self, Self::FromError> {
		let bytes = array_split!(bytes,
			exists : 0x1,
			effect : [0xf],
		);

		// If the exists byte is 0, return None
		if *bytes.exists == 0 {
			return Ok(Self(None));
		}

		// Else get the effect
		Ok(Self(Some(Effect::from_bytes(bytes.effect)?)))
	}

	fn to_bytes(&self, bytes: &mut Self::ByteArray) -> Result<(), Self::ToError> {
		let bytes = array_split_mut!(bytes,
			exists: 0x1,
			effect: [0xf],
		);

		// Check if we exist
		match &self.0 {
			Some(effect) => {
				*bytes.exists = 1;
				effect.to_bytes(bytes.effect)?;
			},
			None => {
				*bytes.exists = 0;
			},
		};

		// An return Ok
		Ok(())
	}
}
