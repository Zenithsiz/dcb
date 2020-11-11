//! System calls

/// Sys instruction func
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum SysInstKind {
	/// Syscall
	Sys,

	/// Break
	Break,
}

impl SysInstKind {
	/// Returns the mnemonic associated with this syscall kind
	pub fn mnemonic(self) -> &'static str {
		match self {
			SysInstKind::Sys => "sys",
			SysInstKind::Break => "break",
		}
	}
}

/// Raw representation
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct SysInstRaw {
	/// Comment
	pub c: u32,

	/// Func (bottom bit)
	pub f: u32,
}

/// Syscall instructions
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[derive(derive_more::Display)]
#[display(fmt = "{}, {comment:#x}", "kind.mnemonic()")]
pub struct SysInst {
	/// Comment
	pub comment: u32,

	/// Kind
	pub kind: SysInstKind,
}

impl SysInst {
	/// Decodes this instruction
	#[must_use]
	pub fn decode(SysInstRaw { c, f }: SysInstRaw) -> Option<Self> {
		let kind = match f {
			0 => SysInstKind::Sys,
			1 => SysInstKind::Break,
			_ => return None,
		};

		Some(Self { comment: c, kind })
	}

	/// Encodes this instruction
	#[must_use]
	#[allow(clippy::many_single_char_names)] // `Raw` has single character names
	pub fn encode(self) -> SysInstRaw {
		let c = self.comment.to_be_bytes();
		let f = match self.kind {
			SysInstKind::Sys => 0,
			SysInstKind::Break => 1,
		};

		SysInstRaw { c, f }
	}
}
