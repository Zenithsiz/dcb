//! Function kind

/// Function kind
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub enum FuncKind {
	/// Known
	Known,

	/// Heuristics
	Heuristics,
}

impl FuncKind {
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

	/// Returns `true` if the function kind is [`Known`](Self::Known).
	#[must_use]
	pub const fn is_known(self) -> bool {
		matches!(self, Self::Known)
	}

	/// Returns `true` if the function kind is [`Heuristics`](Self::Heuristics).
	#[must_use]
	pub const fn is_heuristics(self) -> bool {
		matches!(self, Self::Heuristics)
	}
}
