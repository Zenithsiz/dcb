#![doc(include = "digivolve_effect.md")]

// Imports
use dcb_bytes::Bytes;

/// A digivolve's effect
#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(strum::IntoStaticStr, strum::Display, strum::EnumIter)]
pub enum DigivolveEffect {
	/// Can digivolve regardless of own speciality,
	/// level of digivolve points
	#[strum(serialize = "Disregard speciality & level & dp")]
	DisregardSpecialityLevelDP,

	/// Can digivolve armor to champion or ultimate
	#[strum(serialize = "Armor to champion or ultimate")]
	ArmorToChampionUltimate,

	/// Can digivolve regardless of speciality by adding 20 dp
	#[strum(serialize = "Disregard speciality for 20 dp")]
	DisregardSpecialityFor20DP,

	/// Can digivolve at same level with dp, ignoring speciality
	#[strum(serialize = "Same level without dp, ignoring speciality")]
	SameLevelWithDPIgnoringSpeciality,

	/// Can digivolve from rookie to ultimate
	#[strum(serialize = "Rookie to ultimate")]
	RookieToUltimate,

	/// Downgrade armor to rookie
	#[strum(serialize = "Downgrade armor to rookie")]
	DowngradeArmorToRookie,

	/// Can disregard dp.
	/// Not possible in abnormal states
	#[strum(serialize = "Disregard dp in non-abnormal states")]
	DisregardDPInNonAbnormalStates,

	/// Downgrade by a level.
	/// Hp doubled when successful
	#[strum(serialize = "Downgrade level with hp boost on success")]
	DowngradeLevelWithHpBoostOnSuccess,
}

impl DigivolveEffect {
	/// Return a script describing this effect
	#[must_use]
	pub fn as_str(self) -> &'static str {
		self.into()
	}
}

/// Error type for [`Bytes::deserialize_bytes`](dcb_bytes::Bytes::deserialize_bytes)
#[derive(PartialEq, Eq, Clone, Copy, Debug, thiserror::Error)]
pub enum DeserializeBytesError {
	/// Unknown effect type
	#[error("Unknown bytes for an effect type: {:?}", bytes)]
	UnknownEffect {
		/// Unknown bytes
		bytes: [u8; 3],
	},
}

impl Bytes for DigivolveEffect {
	type ByteArray = [u8; 0x3];
	type DeserializeError = DeserializeBytesError;
	type SerializeError = !;

	fn deserialize_bytes(bytes: &Self::ByteArray) -> Result<Self, Self::DeserializeError> {
		let effect = match *bytes {
			[0, 0, 5] => Self::DisregardSpecialityLevelDP,
			[14, 21, 6] => Self::ArmorToChampionUltimate,
			[12, 16, 0] => Self::DisregardSpecialityFor20DP,
			[10, 16, 3] => Self::SameLevelWithDPIgnoringSpeciality,
			[8, 14, 1] => Self::RookieToUltimate,
			[4, 14, 7] => Self::DowngradeArmorToRookie,
			[6, 14, 2] => Self::DisregardDPInNonAbnormalStates,
			[2, 5, 4] => Self::DowngradeLevelWithHpBoostOnSuccess,
			bytes => return Err(DeserializeBytesError::UnknownEffect { bytes }),
		};

		Ok(effect)
	}

	fn serialize_bytes(&self, bytes: &mut Self::ByteArray) -> Result<(), Self::SerializeError> {
		*bytes = match *self {
			Self::DisregardSpecialityLevelDP => [0, 0, 5],
			Self::ArmorToChampionUltimate => [14, 21, 6],
			Self::DisregardSpecialityFor20DP => [12, 16, 0],
			Self::SameLevelWithDPIgnoringSpeciality => [10, 16, 3],
			Self::RookieToUltimate => [8, 14, 1],
			Self::DowngradeArmorToRookie => [4, 14, 7],
			Self::DisregardDPInNonAbnormalStates => [6, 14, 2],
			Self::DowngradeLevelWithHpBoostOnSuccess => [2, 5, 4],
		};

		Ok(())
	}
}
