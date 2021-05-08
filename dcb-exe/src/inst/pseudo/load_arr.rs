//! Load array

// Imports
use super::{Decodable, Encodable};
use crate::inst::{
	basic, parse::LineArg, DisplayCtx, InstDisplay, InstFmtArg, InstSize, Parsable, ParseCtx, ParseError, Register,
};
use std::array;

/// Load instruction kind
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Kind {
	/// Byte, `i8`
	Byte,

	/// Half-word, `i16`
	HalfWord,

	/// Word, `u32`
	Word,

	/// Byte unsigned, `u8`
	ByteUnsigned,

	/// Half-word unsigned, `u16`
	HalfWordUnsigned,
}

impl Kind {
	/// Returns this kind's size
	#[must_use]
	pub const fn size(self) -> u8 {
		match self {
			Kind::Byte | Kind::ByteUnsigned => 1,
			Kind::HalfWord | Kind::HalfWordUnsigned => 2,
			Kind::Word => 4,
		}
	}
}

/// Load array
///
/// Alias for
/// ```mips
/// l{b, bu, h, hu, w} $.., start($addr)
/// l{b, bu, h, hu, w} $.., start+4($addr)
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
			basic::Inst::Load(basic::load::Inst {
				value,
				addr,
				offset,
				kind,
			}) => (addr, value, offset, match kind {
				basic::load::Kind::Byte => Kind::Byte,
				basic::load::Kind::HalfWord => Kind::HalfWord,
				basic::load::Kind::Word => Kind::Word,
				basic::load::Kind::ByteUnsigned => Kind::ByteUnsigned,
				basic::load::Kind::HalfWordUnsigned => Kind::HalfWordUnsigned,
				_ => return None,
			}),
			_ => return None,
		};

		// Then keep reading while they're in order.
		let mut registers = vec![reg];
		let mut cur_offset = offset;
		while let Some(basic::Inst::Load(basic::load::Inst {
			value,
			addr: Register::Sp,
			offset,
			kind: next_kind,
		})) = insts.next()
		{
			match (kind, next_kind) {
				(Kind::Byte, basic::load::Kind::Byte) |
				(Kind::HalfWord, basic::load::Kind::HalfWord) |
				(Kind::Word, basic::load::Kind::Word) |
				(Kind::ByteUnsigned, basic::load::Kind::ByteUnsigned) |
				(Kind::HalfWordUnsigned, basic::load::Kind::HalfWordUnsigned) => (),
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
			basic::Inst::Load(basic::load::Inst {
				value:  reg,
				addr:   Register::Sp,
				offset: start_offset + i16::from(kind.size()) * idx,
				kind:   match kind {
					Kind::Byte => basic::load::Kind::Byte,
					Kind::HalfWord => basic::load::Kind::HalfWord,
					Kind::Word => basic::load::Kind::Word,
					Kind::ByteUnsigned => basic::load::Kind::ByteUnsigned,
					Kind::HalfWordUnsigned => basic::load::Kind::HalfWordUnsigned,
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
			"lbarr" => Kind::Byte,
			"lharr" => Kind::HalfWord,
			"lwarr" => Kind::Word,
			"lbuarr" => Kind::ByteUnsigned,
			"lhuarr" => Kind::HalfWordUnsigned,
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
	type Args = array::IntoIter<InstFmtArg<'a>, 2>;
	type Mnemonic = &'static str;

	fn mnemonic<Ctx: DisplayCtx>(&'a self, _ctx: &Ctx) -> Self::Mnemonic {
		match self.kind {
			Kind::Byte => "lbarr",
			Kind::HalfWord => "lharr",
			Kind::Word => "lwarr",
			Kind::ByteUnsigned => "lbuarr",
			Kind::HalfWordUnsigned => "lhuarr",
		}
	}

	fn args<Ctx: DisplayCtx>(&'a self, _ctx: &Ctx) -> Self::Args {
		array::IntoIter::new([
			InstFmtArg::RegArray(&self.registers),
			InstFmtArg::register_offset(self.addr, self.offset),
		])
	}
}

impl InstSize for Inst {
	fn size(&self) -> usize {
		self.registers.len() * 4
	}
}
