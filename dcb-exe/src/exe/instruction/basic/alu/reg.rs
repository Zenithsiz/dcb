//! Alu register instructions

// Imports
use crate::exe::instruction::Register;

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

/// Raw representation
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct AluRegInstRaw {
	/// Rs
	s: u32,

	/// Rt
	t: u32,

	/// Rd
	d: u32,

	/// Func (lower 4 bits)
	f: u32,
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
			0x0 => AluRegInstKind::Add,
			0x1 => AluRegInstKind::AddUnsigned,
			0x2 => AluRegInstKind::Sub,
			0x3 => AluRegInstKind::SubUnsigned,
			0x4 => AluRegInstKind::And,
			0x5 => AluRegInstKind::Or,
			0x6 => AluRegInstKind::Xor,
			0x7 => AluRegInstKind::Nor,
			0xa => AluRegInstKind::SetLessThan,
			0xb => AluRegInstKind::SetLessThanUnsigned,
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
			AluRegInstKind::Add => 0x0,
			AluRegInstKind::AddUnsigned => 0x1,
			AluRegInstKind::Sub => 0x2,
			AluRegInstKind::SubUnsigned => 0x3,
			AluRegInstKind::And => 0x4,
			AluRegInstKind::Or => 0x5,
			AluRegInstKind::Xor => 0x6,
			AluRegInstKind::Nor => 0x7,
			AluRegInstKind::SetLessThan => 0xa,
			AluRegInstKind::SetLessThanUnsigned => 0xb,
		};

		let d = self.dst.idx();
		let s = self.lhs.idx();
		let t = self.rhs.idx();

		AluRegInstRaw { f, t, d, s }
	}
}
