//! Data kind

/// Data kind
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub enum DataKind {
	/// Known
	Known,

	/// Foreign
	Foreign,

	/// Heuristics
	Heuristics,
}

impl DataKind {
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

	/// Returns `true` if the data kind is [`Known`](Self::Known).
	#[must_use]
	pub const fn is_known(self) -> bool {
		matches!(self, Self::Known)
	}

	/// Returns `true` if the data kind is [`Foreign`](Self::Foreign).
	#[must_use]
	pub const fn is_foreign(self) -> bool {
		matches!(self, Self::Foreign)
	}

	/// Returns `true` if the data kind is [`Heuristics`](Self::Heuristics).
	#[must_use]
	pub const fn is_heuristics(self) -> bool {
		matches!(self, Self::Heuristics)
	}
}
