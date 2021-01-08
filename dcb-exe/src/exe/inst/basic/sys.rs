//! System calls

/// Sys instruction func
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Kind {
	/// Syscall
	Sys,

	/// Break
	Break,
}

impl Kind {
	/// Returns the mnemonic associated with this syscall kind
	#[must_use]
	pub const fn mnemonic(self) -> &'static str {
		match self {
			Self::Sys => "sys",
			Self::Break => "break",
		}
	}
}

/// Raw representation
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Raw {
	/// Comment
	pub c: u32,

	/// Func (bottom bit)
	pub f: u32,
}

/// Syscall instructions
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[derive(derive_more::Display)]
#[display(fmt = "{}, {comment:#x}", "kind.mnemonic()")]
pub struct Inst {
	/// Comment
	pub comment: u32,

	/// Kind
	pub kind: Kind,
}

impl Inst {
	/// Decodes this instruction
	#[must_use]
	pub const fn decode(Raw { c, f }: Raw) -> Option<Self> {
		let kind = match f {
			0 => Kind::Sys,
			1 => Kind::Break,
			_ => return None,
		};

		Some(Self { comment: c, kind })
	}

	/// Encodes this instruction
	#[must_use]
	#[allow(clippy::many_single_char_names)] // `Raw` has single character names
	pub const fn encode(self) -> Raw {
		let c = self.comment;
		let f = match self.kind {
			Kind::Sys => 0,
			Kind::Break => 1,
		};

		Raw { c, f }
	}
}
