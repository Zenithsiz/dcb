//! Basic instructions
//!
//! All instructions in this module are a single word long, and
//! may be decoded from a `u32` via the [`Inst::decode`](<Inst as Decodable>::decode) method,
//! using the [`Decodable`] trait.

// Modules
pub mod alu;
pub mod co;
pub mod cond;
pub mod jmp;
pub mod load;
pub mod lui;
pub mod mult;
pub mod shift;
pub mod store;
pub mod sys;

// Imports
use super::{parse::LineArg, InstSize, ParseCtx, ParseError, Register};
use crate::inst::InstFmt;

/// All basic instructions
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[derive(derive_more::TryInto)]
pub enum Inst {
	/// Alu
	Alu(alu::Inst),

	/// Condition
	Cond(cond::Inst),

	/// Jump
	Jmp(jmp::Inst),

	/// Load
	Load(load::Inst),

	/// Load upper immediate
	Lui(lui::Inst),

	/// Multiplication
	Mult(mult::Inst),

	/// Shift
	Shift(shift::Inst),

	/// Store
	Store(store::Inst),

	/// Syscall
	Sys(sys::Inst),

	/// Co-processor
	Co(co::Inst),
}


impl Decodable for Inst {
	#[rustfmt::skip]
	fn decode(raw: u32) -> Option<Self> {
		None
			.or_else(|| alu  ::Inst::decode(raw).map(Self::Alu  ))
			.or_else(|| cond ::Inst::decode(raw).map(Self::Cond ))
			.or_else(|| jmp  ::Inst::decode(raw).map(Self::Jmp  ))
			.or_else(|| load ::Inst::decode(raw).map(Self::Load ))
			.or_else(|| lui  ::Inst::decode(raw).map(Self::Lui  ))
			.or_else(|| mult ::Inst::decode(raw).map(Self::Mult ))
			.or_else(|| shift::Inst::decode(raw).map(Self::Shift))
			.or_else(|| store::Inst::decode(raw).map(Self::Store))
			.or_else(|| sys  ::Inst::decode(raw).map(Self::Sys  ))
			.or_else(|| co   ::Inst::decode(raw).map(Self::Co   ))
	}
}

impl Encodable for Inst {
	#[rustfmt::skip]
	fn encode(&self) -> u32 {
		match self {
			Self::Alu  (inst) => inst.encode(),
			Self::Cond (inst) => inst.encode(),
			Self::Jmp  (inst) => inst.encode(),
			Self::Load (inst) => inst.encode(),
			Self::Lui  (inst) => inst.encode(),
			Self::Mult (inst) => inst.encode(),
			Self::Shift(inst) => inst.encode(),
			Self::Store(inst) => inst.encode(),
			Self::Sys  (inst) => inst.encode(),
			Self::Co   (inst) => inst.encode(),
		}
	}
}

impl super::parse::Parsable for Inst {
	fn parse<Ctx: ?Sized + ParseCtx>(mnemonic: &str, args: &[LineArg], ctx: &Ctx) -> Result<Self, ParseError> {
		#[rustfmt::skip]
		let parsers: &[&dyn Fn() -> Result<Self, ParseError>] = &[
			&|| alu  ::Inst::parse(mnemonic, args, ctx).map(Self::Alu  ),
			&|| cond ::Inst::parse(mnemonic, args, ctx).map(Self::Cond ),
			&|| jmp  ::Inst::parse(mnemonic, args, ctx).map(Self::Jmp  ),
			&|| load ::Inst::parse(mnemonic, args, ctx).map(Self::Load ),
			&|| lui  ::Inst::parse(mnemonic, args, ctx).map(Self::Lui  ),
			&|| mult ::Inst::parse(mnemonic, args, ctx).map(Self::Mult ),
			&|| shift::Inst::parse(mnemonic, args, ctx).map(Self::Shift),
			&|| store::Inst::parse(mnemonic, args, ctx).map(Self::Store),
			&|| sys  ::Inst::parse(mnemonic, args, ctx).map(Self::Sys  ),
			&|| co   ::Inst::parse(mnemonic, args, ctx).map(Self::Co   ),
		];

		// Try to parse each one one by one.
		// If we get an unknown mnemonic, try the next, else return the error.
		for parser in parsers {
			match parser() {
				Ok(inst) => return Ok(inst),
				Err(ParseError::UnknownMnemonic) => continue,
				Err(err) => return Err(err),
			}
		}

		Err(ParseError::UnknownMnemonic)
	}
}

impl ModifiesReg for Inst {
	#[rustfmt::skip]
	fn modifies_reg(&self, reg: Register) -> bool {
		match self {
			Inst::Alu  (inst) => inst.modifies_reg(reg),
			Inst::Cond (inst) => inst.modifies_reg(reg),
			Inst::Jmp  (inst) => inst.modifies_reg(reg),
			Inst::Load (inst) => inst.modifies_reg(reg),
			Inst::Lui  (inst) => inst.modifies_reg(reg),
			Inst::Mult (inst) => inst.modifies_reg(reg),
			Inst::Shift(inst) => inst.modifies_reg(reg),
			Inst::Store(inst) => inst.modifies_reg(reg),
			Inst::Sys  (inst) => inst.modifies_reg(reg),
			Inst::Co   (inst) => inst.modifies_reg(reg),
		}
	}
}

// Any basic decodable instruction is 4 bytes
impl<T: Decodable> InstSize for T {
	fn size(&self) -> usize {
		4
	}
}

impl InstFmt for Inst {
	#[rustfmt::skip]
	fn fmt(&self, pos: crate::Pos, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self {
			Self::Alu  (inst) => inst.fmt(pos, f),
			Self::Cond (inst) => inst.fmt(pos, f),
			Self::Jmp  (inst) => inst.fmt(pos, f),
			Self::Load (inst) => inst.fmt(pos, f),
			Self::Lui  (inst) => inst.fmt(pos, f),
			Self::Mult (inst) => inst.fmt(pos, f),
			Self::Shift(inst) => inst.fmt(pos, f),
			Self::Store(inst) => inst.fmt(pos, f),
			Self::Sys  (inst) => inst.fmt(pos, f),
			Self::Co   (inst) => inst.fmt(pos, f),
		}
	}
}

/// A decodable basic instruction
pub trait Decodable: Sized {
	/// Decodes this instruction
	#[must_use]
	fn decode(raw: u32) -> Option<Self>;
}

/// An encodable basic instruction
pub trait Encodable {
	/// Encodes this instruction
	#[must_use]
	fn encode(&self) -> u32;
}

// TODO: TryEncode?

/// Register modifying instructions
pub trait ModifiesReg: Decodable {
	/// Returns if this instruction modifies `reg`.
	fn modifies_reg(&self, reg: Register) -> bool;
}
