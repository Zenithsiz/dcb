//! Store array

// Imports
use super::{Decodable, Encodable};
use crate::inst::{
	basic, parse::LineArg, DisplayCtx, InstDisplay, InstFmtArg, InstSize, Parsable, ParseCtx, ParseError, Register,
};

/// Store instruction kind
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Kind {
	/// Byte, `i8`
	Byte,

	/// Half-word, `i16`
	HalfWord,

	/// Word, `u32`
	Word,
}

impl Kind {
	/// Returns this kind's size
	#[must_use]
	pub const fn size(self) -> u8 {
		match self {
			Kind::Byte => 1,
			Kind::HalfWord => 2,
			Kind::Word => 4,
		}
	}
}

/// Store array
///
/// Alias for
/// ```mips
/// s{b, h, w} $.., start($addr)
/// s{b, h, w} $.., start+4($addr)
/// ...
/// ```
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Inst {
	/// Start offset
	offset: i16,

	/// Register offset
	addr: Register,

	/// All registers
	registers: Vec<Register>,

	/// Kind
	kind: Kind,
}


impl Decodable for Inst {
	fn decode(mut insts: impl Iterator<Item = basic::Inst> + Clone) -> Option<Self> {
		// If it's a `sw $.., offset($sp)`, get the initial offset and keep reading
		let (addr, reg, offset, kind) = match insts.next()? {
			basic::Inst::Store(basic::store::Inst {
				value,
				addr,
				offset,
				kind,
			}) => (addr, value, offset, match kind {
				basic::store::Kind::Byte => Kind::Byte,
				basic::store::Kind::HalfWord => Kind::HalfWord,
				basic::store::Kind::Word => Kind::Word,
				_ => return None,
			}),
			_ => return None,
		};

		// Then keep reading while they're in order.
		let mut registers = vec![reg];
		let mut cur_offset = offset;
		while let Some(basic::Inst::Store(basic::store::Inst {
			value,
			addr: Register::Sp,
			offset,
			kind: next_kind,
		})) = insts.next()
		{
			match (kind, next_kind) {
				(Kind::Byte, basic::store::Kind::Byte) |
				(Kind::HalfWord, basic::store::Kind::HalfWord) |
				(Kind::Word, basic::store::Kind::Word) => (),
				_ => break,
			};

			if offset != cur_offset + i16::from(kind.size()) {
				break;
			}

			cur_offset += i16::from(kind.size());
			registers.push(value);
		}

		// If we got at least 2 saves, return us
		match registers.len() {
			2..=usize::MAX => Some(Self {
				offset,
				addr,
				registers,
				kind,
			}),
			_ => None,
		}
	}
}

impl<'a> Encodable<'a> for Inst {
	type Iterator = impl Iterator<Item = basic::Inst> + 'a;

	fn encode(&'a self) -> Self::Iterator {
		let kind = self.kind;
		let start_offset = self.offset;
		self.registers.iter().copied().zip(0..).map(move |(reg, idx)| {
			basic::Inst::Store(basic::store::Inst {
				value:  reg,
				addr:   Register::Sp,
				offset: start_offset + i16::from(kind.size()) * idx,
				kind:   match kind {
					Kind::Byte => basic::store::Kind::Byte,
					Kind::HalfWord => basic::store::Kind::HalfWord,
					Kind::Word => basic::store::Kind::Word,
				},
			})
		})
	}
}

impl<'a> Parsable<'a> for Inst {
	fn parse<Ctx: ?Sized + ParseCtx<'a>>(
		mnemonic: &'a str, args: &'a [LineArg], ctx: &Ctx,
	) -> Result<Self, ParseError> {
		let kind = match mnemonic {
			"sbarr" => Kind::Byte,
			"sharr" => Kind::HalfWord,
			"swarr" => Kind::Word,
			_ => return Err(ParseError::UnknownMnemonic),
		};

		let (offset, addr, registers) = match *args {
			[LineArg::RegisterArr(ref registers), LineArg::RegisterOffset { register, ref offset }] => {
				(ctx.eval_expr_as(offset)?, register, registers.clone())
			},
			[LineArg::RegisterArr(ref registers), LineArg::Register(addr)] => (0, addr, registers.clone()),
			_ => return Err(ParseError::InvalidArguments),
		};

		Ok(Self {
			offset,
			addr,
			registers,
			kind,
		})
	}
}

impl<'a> InstDisplay<'a> for Inst {
	type Args = [InstFmtArg<'a>; 2];
	type Mnemonic = &'static str;

	fn mnemonic<Ctx: DisplayCtx>(&'a self, _ctx: &Ctx) -> Self::Mnemonic {
		match self.kind {
			Kind::Byte => "sbarr",
			Kind::HalfWord => "sharr",
			Kind::Word => "swarr",
		}
	}

	fn args<Ctx: DisplayCtx>(&'a self, _ctx: &Ctx) -> Self::Args {
		[
			InstFmtArg::RegArray(&self.registers),
			InstFmtArg::register_offset(self.addr, self.offset),
		]
	}
}

impl InstSize for Inst {
	fn size(&self) -> usize {
		self.registers.len() * 4
	}
}
