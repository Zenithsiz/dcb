//! Pseudo instructions
//!
//! All instructions in this module are variable length, and are decoded
//! from a starting basic instruction and remaining instruction bytes,
//! via the [`Decodable`] trait.

// Modules
pub mod bios;
pub mod load;
pub mod load_arr;
pub mod load_imm;
pub mod move_reg;
pub mod nop;
pub mod store;
pub mod store_arr;

// Imports
use super::{basic, parse::LineArg, DisplayCtx, InstDisplay, InstFmtArg, InstSize, Parsable, ParseCtx, ParseError};
use core::fmt;

/// A pseudo instruction
#[derive(PartialEq, Eq, Clone, Debug)]
#[derive(derive_more::TryInto)]
pub enum Inst {
	/// Load immediate
	LoadImm(load_imm::Inst),

	/// No-op
	Nop(nop::Inst),

	/// Move register
	MoveReg(move_reg::Inst),

	/// Load
	Load(load::Inst),

	/// Store
	Store(store::Inst),

	/// Load array
	LoadArr(load_arr::Inst),

	/// Store array
	StoreArr(store_arr::Inst),

	/// Bios
	Bios(bios::Inst),
}

impl Decodable for Inst {
	#[rustfmt::skip]
	fn decode(insts: impl Iterator<Item = basic::Inst> + Clone) -> Option<Self> {
		// Note: Order is important
		None.or_else(|| bios        ::Inst::decode(insts.clone()).map(Self::Bios    ))
			.or_else(|| load_imm    ::Inst::decode(insts.clone()).map(Self::LoadImm ))
			.or_else(|| nop         ::Inst::decode(insts.clone()).map(Self::Nop     ))
			.or_else(|| load_arr    ::Inst::decode(insts.clone()).map(Self::LoadArr ))
			.or_else(|| store_arr   ::Inst::decode(insts.clone()).map(Self::StoreArr))
			.or_else(|| load        ::Inst::decode(insts.clone()).map(Self::Load    ))
			.or_else(|| store       ::Inst::decode(insts.clone()).map(Self::Store   ))
			.or_else(|| move_reg    ::Inst::decode(insts.clone()).map(Self::MoveReg ))
	}
}

impl<'a> Encodable<'a> for Inst {
	type Iterator = impl IntoIterator<Item = basic::Inst> + 'a;

	#[auto_enums::auto_enum(Iterator)]
	fn encode(&'a self) -> Self::Iterator {
		match self {
			Inst::LoadImm(inst) => inst.encode(),
			Inst::Nop(inst) => inst.encode(),
			Inst::MoveReg(inst) => inst.encode(),
			Inst::LoadArr(inst) => inst.encode(),
			Inst::StoreArr(inst) => inst.encode(),
			Inst::Load(inst) => inst.encode(),
			Inst::Store(inst) => inst.encode(),
			Inst::Bios(inst) => inst.encode(),
		}
	}
}

impl<'a> Parsable<'a> for Inst {
	fn parse<Ctx: ?Sized + ParseCtx<'a>>(
		mnemonic: &'a str, args: &'a [LineArg], ctx: &Ctx,
	) -> Result<Self, ParseError> {
		#[rustfmt::skip]
		let parsers: &[&dyn Fn() -> Result<Self, ParseError>] = &[
			&|| load_imm ::Inst::parse(mnemonic, args, ctx).map(Self::LoadImm),
			&|| nop      ::Inst::parse(mnemonic, args, ctx).map(Self::Nop),
			&|| move_reg ::Inst::parse(mnemonic, args, ctx).map(Self::MoveReg),
			&|| load_arr ::Inst::parse(mnemonic, args, ctx).map(Self::LoadArr),
			&|| store_arr::Inst::parse(mnemonic, args, ctx).map(Self::StoreArr),
			&|| load     ::Inst::parse(mnemonic, args, ctx).map(Self::Load),
			&|| store    ::Inst::parse(mnemonic, args, ctx).map(Self::Store),
			&|| bios     ::Inst::parse(mnemonic, args, ctx).map(Self::Bios),
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
			Inst::LoadImm (inst) => inst.mnemonic(ctx),
			Inst::Nop     (inst) => inst.mnemonic(ctx),
			Inst::MoveReg (inst) => inst.mnemonic(ctx),
			Inst::LoadArr (inst) => inst.mnemonic(ctx),
			Inst::StoreArr(inst) => inst.mnemonic(ctx),
			Inst::Load    (inst) => inst.mnemonic(ctx),
			Inst::Store   (inst) => inst.mnemonic(ctx),
			Inst::Bios    (inst) => inst.mnemonic(ctx),
		}
	}

	#[auto_enums::auto_enum(Iterator)]
	#[rustfmt::skip]
	fn args<Ctx: DisplayCtx>(&'a self, ctx: &Ctx) -> Self::Args {
		match self {
			Inst::LoadImm (inst) => inst.args(ctx),
			Inst::Nop     (inst) => inst.args(ctx),
			Inst::MoveReg (inst) => inst.args(ctx),
			Inst::LoadArr (inst) => inst.args(ctx),
			Inst::StoreArr(inst) => inst.args(ctx),
			Inst::Load    (inst) => inst.args(ctx),
			Inst::Store   (inst) => inst.args(ctx),
			Inst::Bios    (inst) => inst.args(ctx),
		}
	}
}

impl InstSize for Inst {
	fn size(&self) -> usize {
		match self {
			Self::LoadImm(inst) => inst.size(),
			Self::Nop(inst) => inst.size(),
			Self::MoveReg(inst) => inst.size(),
			Self::LoadArr(inst) => inst.size(),
			Self::StoreArr(inst) => inst.size(),
			Self::Load(inst) => inst.size(),
			Self::Store(inst) => inst.size(),
			Self::Bios(inst) => inst.size(),
		}
	}
}
/// A decodable pseudo instruction
pub trait Decodable: InstSize + Sized {
	/// Decodes this instruction
	#[must_use]
	fn decode(insts: impl Iterator<Item = basic::Inst> + Clone) -> Option<Self>;
}

/// An encodable pseudo instruction
pub trait Encodable<'a> {
	/// Iterator type
	type Iterator: IntoIterator<Item = basic::Inst> + 'a;

	/// Encodes this instruction as basic instructions
	#[must_use]
	fn encode(&'a self) -> Self::Iterator;
}
