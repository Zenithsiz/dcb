//! Effect condition operation

// Imports
use dcb_bytes::Bytes;

/// A card's effect condition operation
///
/// # Todo
/// These don't seem to be 100% right, the less than property, sometimes does less than number, might be a range check
#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(strum::IntoStaticStr, strum::Display, strum::EnumIter)]
pub enum EffectConditionOperation {
	/// Less than property
	#[strum(serialize = "Less than property")]
	LessThanProperty    = 0,

	/// Less than number
	#[strum(serialize = "Less than number")]
	LessThanNumber      = 1,

	/// More than property
	#[strum(serialize = "More than property")]
	MoreThanProperty    = 2,

	/// More than number
	#[strum(serialize = "More than number")]
	MoreThanNumber      = 3,

	/// Different from number
	#[strum(serialize = "Different from number")]
	DifferentFromNumber = 4,

	/// Equal to number
	#[strum(serialize = "Equal to number")]
	EqualToNumber       = 5,
}

/// Error type for [`Bytes::from_bytes`]
#[derive(PartialEq, Eq, Clone, Copy, Debug, thiserror::Error)]
pub enum FromBytesError {
	/// Unknown value
	#[error("Unknown byte {:#x} for a effect condition operation", byte)]
	UnknownValue {
		/// Unknown byte
		byte: u8,
	},
}

impl Bytes for EffectConditionOperation {
	type ByteArray = u8;
	type FromError = FromBytesError;
	type ToError = !;

	fn from_bytes(byte: &Self::ByteArray) -> Result<Self, Self::FromError> {
		match byte {
			0 => Ok(Self::LessThanProperty),
			1 => Ok(Self::LessThanNumber),
			2 => Ok(Self::MoreThanProperty),
			3 => Ok(Self::MoreThanNumber),
			4 => Ok(Self::DifferentFromNumber),
			5 => Ok(Self::EqualToNumber),
			// TODO: Not do this here and just have someone above check?
			0xFF => {
				log::warn!("Found byte 0xFF for effect condition operation. Interpreting as `EqualToNumber`");
				log::info!("The previous warning should only appear for \"Aquilamon\" in the original game file.");
				Ok(Self::EqualToNumber)
			},
			&byte => Err(FromBytesError::UnknownValue { byte }),
		}
	}

	#[allow(unreachable_code, unused_variables)]
	fn to_bytes(&self, byte: &mut Self::ByteArray) -> Result<(), Self::ToError> {
		*byte = match self {
			Self::LessThanProperty => 0,
			Self::LessThanNumber => 1,
			Self::MoreThanProperty => 2,
			Self::MoreThanNumber => 3,
			Self::DifferentFromNumber => 4,
			Self::EqualToNumber => 5,
		};
		Ok(())
	}
}
impl EffectConditionOperation {
	/// Returns a string representing this
	#[must_use]
	pub fn as_str(self) -> &'static str {
		self.into()
	}
}
impl EffectConditionOperation {
	/// Returns the operator string of this operation
	#[must_use]
	pub const fn operator_str(self) -> &'static str {
		match self {
			Self::LessThanProperty | Self::LessThanNumber => "<",
			Self::MoreThanProperty | Self::MoreThanNumber => ">",
			Self::DifferentFromNumber => "!=",
			Self::EqualToNumber => "==",
		}
	}

	/// Returns if this operator targets a property
	#[must_use]
	pub const fn targets_property(self) -> bool {
		matches!(self, Self::LessThanProperty | Self::MoreThanProperty)
	}
}
