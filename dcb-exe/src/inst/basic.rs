//! Basic instructions
//!
//! All instructions in this module are a single word long, and
//! may be decoded from a `u32` via the [`Inst::decode`] method,
//! using the [`Decode`] trait.

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
use super::{
	exec::{ExecCtx, ExecError, Executable},
	parse::{LineArg, Parsable},
	DisplayCtx, InstDisplay, InstFmtArg, InstSize, ParseCtx, ParseError, Register,
};
use std::fmt;

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

impl Decode for Inst {
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

/// Encode error
#[derive(PartialEq, Clone, Debug, thiserror::Error)]
pub enum EncodeError {
	/// Sys
	#[error("Unable to encode `Shift` instruction")]
	Shift(shift::EncodeError),

	/// Sys
	#[error("Unable to encode `Sys` instruction")]
	Sys(sys::EncodeError),
}

impl TryEncode for Inst {
	type Error = EncodeError;

	#[rustfmt::skip]
	fn try_encode(&self) -> Result<u32, Self::Error> {
		match self {
			Self::Alu  (inst) => Ok(inst.encode()),
			Self::Cond (inst) => Ok(inst.encode()),
			Self::Jmp  (inst) => Ok(inst.encode()),
			Self::Load (inst) => Ok(inst.encode()),
			Self::Lui  (inst) => Ok(inst.encode()),
			Self::Mult (inst) => Ok(inst.encode()),
			Self::Shift(inst) => inst.try_encode().map_err(EncodeError::Shift),
			Self::Store(inst) => Ok(inst.encode()),
			Self::Sys  (inst) => inst.try_encode().map_err(EncodeError::Sys),
			Self::Co   (inst) => Ok(inst.encode()),
		}
	}
}

impl<'a> Parsable<'a> for Inst {
	fn parse<Ctx: ?Sized + ParseCtx<'a>>(
		mnemonic: &'a str, args: &'a [LineArg], ctx: &Ctx,
	) -> Result<Self, ParseError> {
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

impl<'a> InstDisplay<'a> for Inst {
	type Args = impl Iterator<Item = InstFmtArg<'a>>;
	type Mnemonic = impl fmt::Display;

	#[auto_enums::auto_enum(Display)]
	#[rustfmt::skip]
	fn mnemonic<Ctx: DisplayCtx>(&'a self, ctx: &Ctx) -> Self::Mnemonic {
		match self {
			Inst::Alu  (inst) => inst.mnemonic(ctx),
			Inst::Cond (inst) => inst.mnemonic(ctx),
			Inst::Jmp  (inst) => inst.mnemonic(ctx),
			Inst::Load (inst) => inst.mnemonic(ctx),
			Inst::Lui  (inst) => inst.mnemonic(ctx),
			Inst::Mult (inst) => inst.mnemonic(ctx),
			Inst::Shift(inst) => inst.mnemonic(ctx),
			Inst::Store(inst) => inst.mnemonic(ctx),
			Inst::Sys  (inst) => inst.mnemonic(ctx),
			Inst::Co   (inst) => inst.mnemonic(ctx),
		}
	}

	#[auto_enums::auto_enum(Iterator)]
	#[rustfmt::skip]
	fn args<Ctx: DisplayCtx>(&'a self, ctx: &Ctx) -> Self::Args {
		match self {
			Inst::Alu  (inst) => inst.args(ctx),
			Inst::Cond (inst) => inst.args(ctx),
			Inst::Jmp  (inst) => inst.args(ctx),
			Inst::Load (inst) => inst.args(ctx),
			Inst::Lui  (inst) => inst.args(ctx),
			Inst::Mult (inst) => inst.args(ctx),
			Inst::Shift(inst) => inst.args(ctx),
			Inst::Store(inst) => inst.args(ctx),
			Inst::Sys  (inst) => inst.args(ctx),
			Inst::Co   (inst) => inst.args(ctx),
		}
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

impl Executable for Inst {
	#[rustfmt::skip]
	fn exec<Ctx: ExecCtx>(&self, state: &mut Ctx) -> Result<(), ExecError> {
		match self {
			Inst::Alu  (inst) => inst.exec(state),
			Inst::Cond (inst) => inst.exec(state),
			Inst::Jmp  (inst) => inst.exec(state),
			Inst::Load (inst) => inst.exec(state),
			Inst::Lui  (inst) => inst.exec(state),
			Inst::Mult (inst) => inst.exec(state),
			Inst::Shift(inst) => inst.exec(state),
			Inst::Store(inst) => inst.exec(state),
			Inst::Sys  (inst) => inst.exec(state),
			Inst::Co   (_) => todo!(),
		}
	}
}

// Any basic decodable instruction is 4 bytes
impl<T: Decode> InstSize for T {
	fn size(&self) -> usize {
		4
	}
}

/// A decodable basic instruction
pub trait Decode: Sized {
	/// Decodes this instruction
	#[must_use]
	fn decode(raw: u32) -> Option<Self>;
}

/// An encodable basic instruction
pub trait Encode {
	/// Encodes this instruction
	#[must_use]
	fn encode(&self) -> u32;
}

/// An encodable basic instruction with possible failure
pub trait TryEncode {
	/// Error type
	type Error;

	/// Attempts to encode the instructions
	fn try_encode(&self) -> Result<u32, Self::Error>;
}

/// Register modifying instructions
pub trait ModifiesReg {
	/// Returns if this instruction modifies `reg`.
	fn modifies_reg(&self, reg: Register) -> bool;
}
