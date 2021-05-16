//! Cross move effect

// Imports
use super::{AttackType, Speciality};

/// A digimon's cross move effect
#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub enum CrossMoveEffect {
	/// Attack first
	AttackFirst,

	/// Attack to 0
	AttackToZero(AttackType),

	/// counter
	Counter(AttackType),

	/// Crash
	Crash,

	/// Eat up HP
	EatUpHP,

	/// Jamming
	Jamming,

	/// 3x against speciality
	TripleAgainst(Speciality),
}

impl CrossMoveEffect {
	/// Returns `true` if the effect is [`AttackToZero`](Self::AttackToZero).
	#[must_use]
	pub const fn is_attack_to_zero(self) -> bool {
		matches!(self, Self::AttackToZero(..))
	}

	/// Returns `true` if the effect is [`Counter`](Self::Counter).
	#[must_use]
	pub const fn is_counter(self) -> bool {
		matches!(self, Self::Counter(..))
	}

	/// Returns `true` if the effect is [`TripleAgainst`](Self::TripleAgainst).
	#[must_use]
	pub const fn is_triple_against(self) -> bool {
		matches!(self, Self::TripleAgainst(..))
	}

	/// Returns a string representing this effect
	#[must_use]
	pub const fn as_str(self) -> &'static str {
		match self {
			Self::AttackFirst => "Attack first",
			Self::AttackToZero(_) => "Attack to zero",
			Self::Counter(_) => "Counter",
			Self::Crash => "Crash",
			Self::EatUpHP => "Eat up HP",
			Self::Jamming => "Jamming",
			Self::TripleAgainst(_) => "Triple against",
		}
	}
}

/// Error type for [`::dcb_bytes::Bytes::from_bytes`]
#[derive(PartialEq, Eq, Clone, Copy, Debug, thiserror::Error)]
pub enum FromBytesError {
	/// Unknown value
	#[error("Unknown byte {:#x} for a cross move effect", byte)]
	UnknownValue {
		/// The byte found
		byte: u8,
	},
}
impl ::dcb_bytes::Bytes for CrossMoveEffect {
	type ByteArray = u8;
	type FromError = FromBytesError;
	type ToError = !;

	fn from_bytes(byte: &Self::ByteArray) -> Result<Self, Self::FromError> {
		match byte {
			1 => Ok(Self::AttackFirst),
			2 => Ok(Self::AttackToZero(AttackType::Circle)),
			3 => Ok(Self::AttackToZero(AttackType::Triangle)),
			4 => Ok(Self::AttackToZero(AttackType::Cross)),
			5 => Ok(Self::Counter(AttackType::Circle)),
			6 => Ok(Self::Counter(AttackType::Triangle)),
			7 => Ok(Self::Counter(AttackType::Cross)),
			8 => Ok(Self::Crash),
			9 => Ok(Self::EatUpHP),
			10 => Ok(Self::Jamming),
			11 => Ok(Self::TripleAgainst(Speciality::Fire)),
			12 => Ok(Self::TripleAgainst(Speciality::Ice)),
			13 => Ok(Self::TripleAgainst(Speciality::Nature)),
			14 => Ok(Self::TripleAgainst(Speciality::Darkness)),
			15 => Ok(Self::TripleAgainst(Speciality::Rare)),
			&byte => Err(Self::FromError::UnknownValue { byte }),
		}
	}

	#[allow(unreachable_code, unused_variables)]
	fn to_bytes(&self, byte: &mut Self::ByteArray) -> Result<(), Self::ToError> {
		*byte = match self {
			Self::AttackFirst => 1,
			Self::AttackToZero(AttackType::Circle) => 2,
			Self::AttackToZero(AttackType::Triangle) => 3,
			Self::AttackToZero(AttackType::Cross) => 4,
			Self::Counter(AttackType::Circle) => 5,
			Self::Counter(AttackType::Triangle) => 6,
			Self::Counter(AttackType::Cross) => 7,
			Self::Crash => 8,
			Self::EatUpHP => 9,
			Self::Jamming => 10,
			Self::TripleAgainst(Speciality::Fire) => 11,
			Self::TripleAgainst(Speciality::Ice) => 12,
			Self::TripleAgainst(Speciality::Nature) => 13,
			Self::TripleAgainst(Speciality::Darkness) => 14,
			Self::TripleAgainst(Speciality::Rare) => 15,
		};
		Ok(())
	}
}
