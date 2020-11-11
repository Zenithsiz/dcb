//! Alu register instructions

// Imports
use crate::exe::instruction::Register;

/// Raw representation
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct AluRegInstRaw {
	/// Rs
	s: u32,

	/// Rt
	t: u32,

	/// Rd
	d: u32,

	/// Func
	f: u32,
}

/// Alu register instruction kind
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[derive(derive_more::Display)]
pub enum AluRegInstKind {
	/// Add signed with overflow trap
	#[display(fmt = "add")]
	Add,

	/// Add signed without overflow trap
	#[display(fmt = "addu")]
	AddUnsigned,

	/// Sub signed with overflow trap
	#[display(fmt = "sub")]
	Sub,

	/// Sub signed without overflow trap
	#[display(fmt = "subu")]
	SubUnsigned,

	/// Bit and
	#[display(fmt = "and")]
	And,

	/// Bit or
	#[display(fmt = "or")]
	Or,

	/// Bit xor
	#[display(fmt = "xor")]
	Xor,

	/// Bit nor
	#[display(fmt = "nor")]
	Nor,

	/// Set on less than signed
	#[display(fmt = "slt")]
	SetLessThan,

	/// Set on less than unsigned
	#[display(fmt = "sltu")]
	SetLessThanUnsigned,
}

/// Alu register instructions
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[derive(derive_more::Display)]
#[display(fmt = "{kind} {dst}, {lhs}, {rhs}")]
pub struct AluRegInst {
	/// Destination register
	pub dst: Register,

	/// Lhs argument
	pub lhs: Register,

	/// Rhs argument
	pub rhs: Register,

	/// Kind
	pub kind: AluRegInstKind,
}

impl AluRegInst {
	/// Decodes this instruction
	#[must_use]
	pub fn decode(raw: AluRegInstRaw) -> Option<Self> {
		let kind = match raw.f {
			0x20 => AluRegInstKind::Add,
			0x21 => AluRegInstKind::AddUnsigned,
			0x22 => AluRegInstKind::Sub,
			0x23 => AluRegInstKind::SubUnsigned,
			0x24 => AluRegInstKind::And,
			0x25 => AluRegInstKind::Or,
			0x26 => AluRegInstKind::Xor,
			0x27 => AluRegInstKind::Nor,
			0x2a => AluRegInstKind::SetLessThan,
			0x2b => AluRegInstKind::SetLessThanUnsigned,
			_ => return None,
		};

		Some(Self {
			dst: Register::new(raw.d)?,
			lhs: Register::new(raw.s)?,
			rhs: Register::new(raw.t)?,
			kind,
		})
	}

	/// Encodes this instruction
	#[must_use]
	pub fn encode(self) -> AluRegInstRaw {
		let f = match self.kind {
			AluRegInstKind::Add => 0x20,
			AluRegInstKind::AddUnsigned => 0x21,
			AluRegInstKind::Sub => 0x22,
			AluRegInstKind::SubUnsigned => 0x23,
			AluRegInstKind::And => 0x24,
			AluRegInstKind::Or => 0x25,
			AluRegInstKind::Xor => 0x26,
			AluRegInstKind::Nor => 0x27,
			AluRegInstKind::SetLessThan => 0x2a,
			AluRegInstKind::SetLessThanUnsigned => 0x2b,
		};

		let d = self.dst.idx();
		let s = self.lhs.idx();
		let t = self.rhs.idx();

		AluRegInstRaw { f, t, d, s }
	}
}
