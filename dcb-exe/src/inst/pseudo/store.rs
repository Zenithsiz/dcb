//! Store instructions

// Imports
use super::{Decodable, Encodable};
use crate::{
	inst::{
		basic::{self, store::Kind},
		parse::LineArg,
		DisplayCtx, InstDisplay, InstFmtArg, InstSize, Parsable, ParseCtx, ParseError, Register,
	},
	Pos,
};
use int_conv::{Join, SignExtended, Signed, Split};
use std::array;

/// Store pseudo instructions
///
/// Alias for
/// ```mips
/// lui $at, {hi}
/// s* $dst, {lo}($at)
/// ```
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Inst {
	/// Value register
	pub value: Register,

	/// Target
	pub target: Pos,

	/// Kind
	pub kind: Kind,
}

impl Decodable for Inst {
	fn decode(mut insts: impl Iterator<Item = basic::Inst> + Clone) -> Option<Self> {
		let inst = match insts.next()? {
			basic::Inst::Lui(lui) if lui.dst == Register::At => match insts.next()? {
				basic::Inst::Store(store) if store.addr == Register::At => Self {
					value:  store.value,
					target: Pos(
						(u32::join(0, lui.value).as_signed() + store.offset.sign_extended::<i32>()).as_unsigned(),
					),
					kind:   store.kind,
				},
				_ => return None,
			},
			_ => return None,
		};

		Some(inst)
	}
}

impl Encodable for Inst {
	type Iterator = impl Iterator<Item = basic::Inst>;

	fn encode(&self) -> Self::Iterator {
		let addr = self.target.0;
		let (lo, hi) = match addr.lo().as_signed() < 0 {
			true => (addr.lo(), addr.hi().wrapping_add(1)),
			false => addr.lo_hi(),
		};

		std::array::IntoIter::new([
			basic::Inst::Lui(basic::lui::Inst {
				dst:   Register::At,
				value: hi,
			}),
			basic::Inst::Store(basic::store::Inst {
				value:  self.value,
				addr:   Register::At,
				offset: lo.as_signed(),
				kind:   self.kind,
			}),
		])
	}
}

impl<'a> Parsable<'a> for Inst {
	fn parse<Ctx: ?Sized + ParseCtx>(mnemonic: &'a str, args: &'a [LineArg], ctx: &'a Ctx) -> Result<Self, ParseError> {
		let kind = match mnemonic {
			"sbi" => Kind::Byte,
			"shi" => Kind::HalfWord,
			"swli" => Kind::WordLeft,
			"swi" => Kind::Word,
			"swri" => Kind::WordRight,
			_ => return Err(ParseError::UnknownMnemonic),
		};

		let (value, target) = match *args {
			[LineArg::Register(value), ref arg] => (value, ctx.arg_pos(arg)?),
			_ => return Err(ParseError::InvalidArguments),
		};

		Ok(Self { value, target, kind })
	}
}

impl<'a> InstDisplay<'a> for Inst {
	type Args = array::IntoIter<InstFmtArg<'a>, 2>;
	type Mnemonic = &'static str;

	fn mnemonic<Ctx: DisplayCtx>(&'a self, _ctx: &Ctx) -> Self::Mnemonic {
		match self.kind {
			Kind::Byte => "sbi",
			Kind::HalfWord => "shi",
			Kind::WordLeft => "swli",
			Kind::Word => "swi",
			Kind::WordRight => "swri",
		}
	}

	fn args<Ctx: DisplayCtx>(&'a self, _ctx: &Ctx) -> Self::Args {
		let &Self { value, target, .. } = self;

		array::IntoIter::new([InstFmtArg::Register(value), InstFmtArg::Target(target)])
	}
}

impl InstSize for Inst {
	fn size(&self) -> usize {
		8
	}
}
