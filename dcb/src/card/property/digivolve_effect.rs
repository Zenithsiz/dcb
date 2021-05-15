#![doc(include = "digivolve_effect.md")]

// Imports
use dcb_bytes::Bytes;

/// A digivolve's effect
#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub enum DigivolveEffect {
	/// Can digivolve regardless of own speciality,
	/// level of digivolve points
	DisregardSpecialityLevelDP,

	/// Can digivolve armor to champion or ultimate
	ArmorToChampionUltimate,

	/// Can digivolve regardless of speciality by adding 20 dp
	DisregardSpecialityFor20DP,

	/// Can digivolve at same level with dp, ignoring speciality
	SameLevelWithDPIgnoringSpeciality,

	/// Can digivolve from rookie to ultimate
	RookieToUltimate,

	/// Downgrade armor to rookie
	DowngradeArmorToRookie,

	/// Can disregard dp.
	/// Not possible in abnormal states
	DisregardDPInNonAbnormalStates,

	/// Downgrade by a level.
	/// Hp doubled when successful
	DowngradeLevelWithHpBoostOnSuccess,
}

impl DigivolveEffect {
	/// All of the effects
	pub const ALL: &'static [Self] = &[
		Self::DisregardSpecialityLevelDP,
		Self::ArmorToChampionUltimate,
		Self::DisregardSpecialityFor20DP,
		Self::SameLevelWithDPIgnoringSpeciality,
		Self::RookieToUltimate,
		Self::DowngradeArmorToRookie,
		Self::DisregardDPInNonAbnormalStates,
		Self::DowngradeLevelWithHpBoostOnSuccess,
	];

	/// Return a script describing this effect
	#[must_use]
	pub const fn as_str(self) -> &'static str {
		match self {
			Self::DisregardSpecialityLevelDP => "Disregard speciality & level & dp",
			Self::ArmorToChampionUltimate => "Armor to champion or ultimate",
			Self::DisregardSpecialityFor20DP => "Disregard speciality for 20 dp",
			Self::SameLevelWithDPIgnoringSpeciality => "Same level without dp, ignoring speciality",
			Self::RookieToUltimate => "Rookie to ultimate",
			Self::DowngradeArmorToRookie => "Downgrade armor to rookie",
			Self::DisregardDPInNonAbnormalStates => "Disregard dp in non-abnormal states",
			Self::DowngradeLevelWithHpBoostOnSuccess => "Downgrade level with hp boost on success",
		}
	}
}

/// Error type for [`Bytes::from_bytes`](dcb_bytes::Bytes::from_bytes)
#[derive(PartialEq, Eq, Clone, Copy, Debug, thiserror::Error)]
pub enum FromBytesError {
	/// Unknown effect type
	#[error("Unknown bytes for an effect type: {:?}", bytes)]
	UnknownEffect {
		/// Unknown bytes
		bytes: [u8; 3],
	},
}

impl Bytes for DigivolveEffect {
	type ByteArray = [u8; 0x3];
	type FromError = FromBytesError;
	type ToError = !;

	fn from_bytes(bytes: &Self::ByteArray) -> Result<Self, Self::FromError> {
		let effect = match *bytes {
			[0, 0, 5] => Self::DisregardSpecialityLevelDP,
			[14, 21, 6] => Self::ArmorToChampionUltimate,
			[12, 16, 0] => Self::DisregardSpecialityFor20DP,
			[10, 16, 3] => Self::SameLevelWithDPIgnoringSpeciality,
			[8, 14, 1] => Self::RookieToUltimate,
			[4, 14, 7] => Self::DowngradeArmorToRookie,
			[6, 14, 2] => Self::DisregardDPInNonAbnormalStates,
			[2, 5, 4] => Self::DowngradeLevelWithHpBoostOnSuccess,
			bytes => return Err(FromBytesError::UnknownEffect { bytes }),
		};

		Ok(effect)
	}

	fn to_bytes(&self, bytes: &mut Self::ByteArray) -> Result<(), Self::ToError> {
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
