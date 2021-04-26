//! Alu immediate instructions

// Imports
use crate::inst::{
	basic::{Decodable, Encodable, ModifiesReg, Parsable, ParseError},
	parse, InstFmt, ParseCtx, Register,
};
use dcb_util::SignedHex;
use int_conv::{Signed, Truncated, ZeroExtended};
use std::{convert::TryInto, fmt};

/// Instruction kind
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Kind {
	/// Add signed with overflow trap
	Add(i16),

	/// Add signed without overflow trap
	AddUnsigned(i16),

	/// Set on less than signed
	SetLessThan(i16),

	/// Set on less than unsigned
	SetLessThanUnsigned(u16),

	/// Bit and
	And(u16),

	/// Bit or
	Or(u16),

	/// Bit xor
	Xor(u16),
}

impl Kind {
	/// Returns this kind's mnemonic
	#[must_use]
	pub const fn mnemonic(self) -> &'static str {
		match self {
			Self::Add(_) => "addi",
			Self::AddUnsigned(_) => "addiu",
			Self::SetLessThan(_) => "slti",
			Self::SetLessThanUnsigned(_) => "sltiu",
			Self::And(_) => "andi",
			Self::Or(_) => "ori",
			Self::Xor(_) => "xori",
		}
	}

	/// Returns a displayable with the value of this kind
	#[must_use]
	pub fn value_fmt(self) -> impl fmt::Display {
		dcb_util::DisplayWrapper::new(move |f| match self {
			// Signed
			Self::Add(rhs) | Self::AddUnsigned(rhs) | Self::SetLessThan(rhs) => write!(f, "{:#}", SignedHex(rhs)),
			// Unsigned
			Self::SetLessThanUnsigned(rhs) | Self::And(rhs) | Self::Or(rhs) | Self::Xor(rhs) => write!(f, "{rhs:#x}"),
		})
	}
}

/// Alu immediate instructions
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Inst {
	/// Destination register
	pub dst: Register,

	/// Lhs argument
	pub lhs: Register,

	/// Kind
	pub kind: Kind,
}

impl Decodable for Inst {
	type Raw = u32;

	#[bitmatch::bitmatch]
	fn decode(raw: Self::Raw) -> Option<Self> {
		let [p, s, t, i] = #[bitmatch]
		match raw {
			"001ppp_sssss_ttttt_iiiii_iiiii_iiiiii" => [p, s, t, i],
			_ => return None,
		};

		Some(Self {
			dst:  Register::new(t)?,
			lhs:  Register::new(s)?,
			kind: match p {
				0x0 => Kind::Add(i.truncated::<u16>().as_signed()),
				0x1 => Kind::AddUnsigned(i.truncated::<u16>().as_signed()),
				0x2 => Kind::SetLessThan(i.truncated::<u16>().as_signed()),
				0x3 => Kind::SetLessThanUnsigned(i.truncated::<u16>()),
				0x4 => Kind::And(i.truncated::<u16>()),
				0x5 => Kind::Or(i.truncated::<u16>()),
				0x6 => Kind::Xor(i.truncated::<u16>()),
				_ => return None,
			},
		})
	}
}

impl Encodable for Inst {
	#[bitmatch::bitmatch]
	fn encode(&self) -> Self::Raw {
		#[rustfmt::skip]
		let (p, i): (u32, u32) = match self.kind {
			Kind::Add                (rhs) => (0x0, rhs.as_unsigned().zero_extended::<u32>()),
			Kind::AddUnsigned        (rhs) => (0x1, rhs.as_unsigned().zero_extended::<u32>()),
			Kind::SetLessThan        (rhs) => (0x2, rhs.as_unsigned().zero_extended::<u32>()),
			Kind::SetLessThanUnsigned(rhs) => (0x3, rhs              .zero_extended::<u32>()),
			Kind::And                (rhs) => (0x4, rhs              .zero_extended::<u32>()),
			Kind::Or                 (rhs) => (0x5, rhs              .zero_extended::<u32>()),
			Kind::Xor                (rhs) => (0x6, rhs              .zero_extended::<u32>()),
		};
		let t = self.dst.idx();
		let s = self.lhs.idx();

		bitpack!("001ppp_sssss_ttttt_iiiii_iiiii_iiiiii")
	}
}

impl Parsable for Inst {
	fn parse<Ctx: ?Sized + ParseCtx>(mnemonic: &str, args: &[parse::Arg], _ctx: &Ctx) -> Result<Self, ParseError> {
		#[rustfmt::skip]
		let to_kind = match mnemonic {
			"addi"  => |value: i64| value.try_into().map(Kind::Add                ),
			"addiu" => |value: i64| value.try_into().map(Kind::AddUnsigned        ),
			"slti"  => |value: i64| value.try_into().map(Kind::SetLessThan        ),
			"sltiu" => |value: i64| value.try_into().map(Kind::SetLessThanUnsigned),
			"andi"  => |value: i64| value.try_into().map(Kind::And                ),
			"ori"   => |value: i64| value.try_into().map(Kind::Or                 ),
			"xori"  => |value: i64| value.try_into().map(Kind::Xor                ),
			_ => return Err(ParseError::UnknownMnemonic),
		};

		match *args {
			// Disallow `slti` and `sltiu` in short form
			[parse::Arg::Register(_), parse::Arg::Literal(_)] if ["slti", "sltiu"].contains(&mnemonic) => Err(ParseError::InvalidArguments),

			// Else parse both `$dst, $lhs, value` and `$dst, value`.
			[parse::Arg::Register(lhs @ dst), parse::Arg::Literal(value)] |
			[parse::Arg::Register(dst), parse::Arg::Register(lhs), parse::Arg::Literal(value)] => Ok(Self {
				dst,
				lhs,
				kind: to_kind(value).map_err(|_| ParseError::LiteralOutOfRange)?,
			}),
			_ => Err(ParseError::InvalidArguments),
		}
	}
}

impl InstFmt for Inst {
	fn fmt(&self, _pos: crate::Pos, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let Self { dst, lhs, kind } = self;
		let mnemonic = kind.mnemonic();
		let value = kind.value_fmt();

		// If we're not `slti[u]` and if `$dst` and `$lhs` are the same,
		// only print one of them
		match !matches!(kind, Kind::SetLessThan(_) | Kind::SetLessThanUnsigned(_)) && dst == lhs {
			true => write!(f, "{mnemonic} {dst}, {value}"),
			false => write!(f, "{mnemonic} {dst}, {lhs}, {value}"),
		}
	}
}

impl ModifiesReg for Inst {
	fn modifies_reg(&self, reg: Register) -> bool {
		self.dst == reg
	}
}
