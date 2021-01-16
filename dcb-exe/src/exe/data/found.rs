//! Data found

/// How data was found
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub enum Found {
	/// Known
	Known,

	/// Heuristics
	Heuristics,
}

impl Found {
	/// Returns `Self::Known`
	#[must_use]
	pub const fn known() -> Self {
		Self::Known
	}

	/// Returns `Self::Heuristics`
	#[must_use]
	pub const fn heuristics() -> Self {
		Self::Heuristics
	}

	/// Returns if known
	#[must_use]
	pub fn is_known(self) -> bool {
		self == Self::Known
	}

	/// Returns if found heuristically
	#[must_use]
	pub fn is_heuristics(self) -> bool {
		self == Self::Heuristics
	}
}
