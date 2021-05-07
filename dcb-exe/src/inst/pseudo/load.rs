//! Load instructions

// Imports
use super::{Decodable, Encodable};
use crate::{
	inst::{
		basic::{self, load::Kind},
		parse::LineArg,
		DisplayCtx, InstDisplay, InstFmtArg, InstSize, Parsable, ParseCtx, ParseError, Register,
	},
	Pos,
};
use int_conv::{Join, SignExtended, Signed, Split};
use std::array;

/// Load pseudo instructions
///
/// Alias for
/// ```mips
/// lui $dst, {hi}
/// l* $dst, {lo}($dst)
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
		#[allow(clippy::suspicious_operation_groupings)] // We're checking for `lui $dst, {} / l* $dst, {}($dst)`.
		let inst = match insts.next()? {
			basic::Inst::Lui(lui) => match insts.next()? {
				basic::Inst::Load(load) if load.value == lui.dst && load.addr == load.value => Self {
					value:  load.value,
					target: Pos(
						(u32::join(0, lui.value).as_signed() + load.offset.sign_extended::<i32>()).as_unsigned(),
					),
					kind:   load.kind,
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
				dst:   self.value,
				value: hi,
			}),
			basic::Inst::Load(basic::load::Inst {
				value:  self.value,
				addr:   self.value,
				offset: lo.as_signed(),
				kind:   self.kind,
			}),
		])
	}
}

impl<'a> Parsable<'a> for Inst {
	fn parse<Ctx: ?Sized + ParseCtx<'a>>(
		mnemonic: &'a str, args: &'a [LineArg], ctx: &Ctx,
	) -> Result<Self, ParseError> {
		let kind = match mnemonic {
			"lbi" => Kind::Byte,
			"lhi" => Kind::HalfWord,
			"lwli" => Kind::WordLeft,
			"lwi" => Kind::Word,
			"lbui" => Kind::ByteUnsigned,
			"lhui" => Kind::HalfWordUnsigned,
			"lwri" => Kind::WordRight,
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
			Kind::Byte => "lbi",
			Kind::HalfWord => "lhi",
			Kind::WordLeft => "lwli",
			Kind::Word => "lwi",
			Kind::ByteUnsigned => "lbui",
			Kind::HalfWordUnsigned => "lhui",
			Kind::WordRight => "lwri",
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
