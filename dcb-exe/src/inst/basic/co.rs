//! Co-processor instructions

// Imports
use super::ModifiesReg;
use crate::inst::{
	basic::{Decode, Encode},
	parse::LineArg,
	DisplayCtx, InstDisplay, InstFmtArg, Parsable, ParseCtx, ParseError, Register,
};
use int_conv::{Signed, Truncated, ZeroExtended};
use std::{array, convert::TryInto, fmt};

/// Co-processor register kind
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum RegisterKind {
	/// Data
	Data,

	/// Control
	Control,
}

/// Co-processor instruction kind
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Kind {
	/// Execute
	CopN {
		/// Command to execute
		imm: u32,
	},

	/// Move register from co-processor
	MoveFrom {
		/// Destination
		dst: Register,

		/// Source
		src: u8,

		/// Register kind
		kind: RegisterKind,
	},

	/// Move register to co-processor
	MoveTo {
		/// Destination
		dst: u8,

		/// Source
		src: Register,

		/// Register kind
		kind: RegisterKind,
	},

	/// Branch if
	Branch {
		/// Offset
		offset: i16,

		/// Value to branch on
		on: bool,
	},

	/// Load co-processor
	Load {
		/// Destination
		dst: u8,

		/// Source
		src: Register,

		/// offset
		offset: i16,
	},

	/// Store co-processor
	Store {
		/// Destination
		dst: u8,

		/// Source
		src: Register,

		/// offset
		offset: i16,
	},
}

/// Store instructions
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Inst {
	/// Co-processor number
	pub n: u32,

	/// Kind
	pub kind: Kind,
}

impl Decode for Inst {
	#[bitmatch::bitmatch]
	fn decode(raw: u32) -> Option<Self> {
		// Get `n`
		#[bitmatch]
		let "????nn_?????_?????_?????_?????_??????" = raw;

		#[rustfmt::skip]
		let kind = #[bitmatch] match raw {
			"0100??_1iiii_iiiii_iiiii_iiiii_iiiiii" => Kind::CopN     { imm: i },
			"0100??_00000_ttttt_ddddd_?????_000000" => Kind::MoveFrom { dst: Register::new(t)?, src: d.truncated(), kind: RegisterKind::Data    },
			"0100??_00010_ttttt_ddddd_?????_000000" => Kind::MoveFrom { dst: Register::new(t)?, src: d.truncated(), kind: RegisterKind::Control },
			"0100??_00100_ttttt_ddddd_?????_000000" => Kind::MoveTo   { dst: d.truncated(), src: Register::new(t)?, kind: RegisterKind::Data    },
			"0100??_00110_ttttt_ddddd_?????_000000" => Kind::MoveTo   { dst: d.truncated(), src: Register::new(t)?, kind: RegisterKind::Control },
			"0100??_01000_00000_iiiii_iiiii_iiiiii" => Kind::Branch   { offset: i.truncated::<u16>().as_signed(), on: false },
			"0100??_01000_00001_iiiii_iiiii_iiiiii" => Kind::Branch   { offset: i.truncated::<u16>().as_signed(), on: true  },
			"1100??_sssss_ttttt_iiiii_iiiii_iiiiii" => Kind::Load     { dst: t.truncated(), src: Register::new(s)?, offset: i.truncated::<u16>().as_signed() },
			"1110??_sssss_ttttt_iiiii_iiiii_iiiiii" => Kind::Store    { dst: t.truncated(), src: Register::new(s)?, offset: i.truncated::<u16>().as_signed() },
			_ => return None,
		};

		Some(Self { n, kind })
	}
}

impl Encode for Inst {
	#[bitmatch::bitmatch]
	fn encode(&self) -> u32 {
		let n = self.n;

		match self.kind {
			Kind::CopN { imm: i } => bitpack!("0100nn_1iiii_iiiii_iiiii_iiiii_iiiiii"),
			Kind::MoveFrom { dst, src, kind } => {
				let t = dst.idx();
				let d = src.zero_extended::<u32>();
				match kind {
					RegisterKind::Data => bitpack!("0100nn_00000_ttttt_ddddd_?????_000000"),
					RegisterKind::Control => bitpack!("0100nn_00010_ttttt_ddddd_?????_000000"),
				}
			},
			Kind::MoveTo { dst, src, kind } => {
				let d = dst.zero_extended::<u32>();
				let t = src.idx();
				match kind {
					RegisterKind::Data => bitpack!("0100nn_00100_ttttt_ddddd_?????_000000"),
					RegisterKind::Control => bitpack!("0100nn_00110_ttttt_ddddd_?????_000000"),
				}
			},
			Kind::Branch { offset, on } => {
				let i = offset.as_unsigned().zero_extended::<u32>();
				match on {
					true => bitpack!("0100nn_01000_00001_iiiii_iiiii_iiiiii"),
					false => bitpack!("0100nn_01000_00000_iiiii_iiiii_iiiiii"),
				}
			},
			Kind::Load { dst, src, offset } => {
				let t = dst.zero_extended::<u32>();
				let s = src.idx();
				let i = offset.as_unsigned().zero_extended::<u32>();
				bitpack!("1100nn_sssss_ttttt_iiiii_iiiii_iiiiii")
			},
			Kind::Store { dst, src, offset } => {
				let t = dst.zero_extended::<u32>();
				let s = src.idx();
				let i = offset.as_unsigned().zero_extended::<u32>();
				bitpack!("1110nn_sssss_ttttt_iiiii_iiiii_iiiiii")
			},
		}
	}
}

impl<'a> Parsable<'a> for Inst {
	fn parse<Ctx: ?Sized + ParseCtx>(mnemonic: &'a str, args: &'a [LineArg], _ctx: &'a Ctx) -> Result<Self, ParseError> {
		let inst = match mnemonic {
			"cop0" | "cop1" | "cop2" | "cop3" => {
				let n = mnemonic[3..].parse().expect("Unable to parse 0..=3");
				let imm = match *args {
					[LineArg::Literal(imm)] => imm.try_into().map_err(|_| ParseError::LiteralOutOfRange)?,
					_ => return Err(ParseError::InvalidArguments),
				};

				Inst { n, kind: Kind::CopN { imm } }
			},
			"mfc0" | "mfc1" | "mfc2" | "mfc3" | "cfc0" | "cfc1" | "cfc2" | "cfc3" | "mtc0" | "mtc1" | "mtc2" | "mtc3" | "ctc0" | "ctc1" |
			"ctc2" | "ctc3" => {
				let n = mnemonic[3..].parse().expect("Unable to parse 0..=3");
				let (reg, imm) = match *args {
					[LineArg::Register(dst), LineArg::Literal(src)] => (dst, src.try_into().map_err(|_| ParseError::LiteralOutOfRange)?),
					_ => return Err(ParseError::InvalidArguments),
				};

				let kind = match &mnemonic[0..=0] {
					"m" => RegisterKind::Data,
					"c" => RegisterKind::Control,
					_ => unreachable!(),
				};

				match &mnemonic[1..=1] {
					"f" => Inst {
						n,
						kind: Kind::MoveFrom { dst: reg, src: imm, kind },
					},
					"t" => Inst {
						n,
						kind: Kind::MoveTo { dst: imm, src: reg, kind },
					},
					_ => unreachable!(),
				}
			},
			"lwc0" | "lwc1" | "lwc2" | "lwc3" | "swc0" | "swc1" | "swc2" | "swc3" => {
				let n = mnemonic[3..].parse().expect("Unable to parse 0..=3");
				let (dst, src, offset) = match *args {
					[LineArg::Literal(dst), LineArg::RegisterOffset { register: src, offset }] => (
						dst.try_into().map_err(|_| ParseError::LiteralOutOfRange)?,
						src,
						offset.try_into().map_err(|_| ParseError::LiteralOutOfRange)?,
					),
					_ => return Err(ParseError::InvalidArguments),
				};

				match &mnemonic[0..=0] {
					"l" => Inst {
						n,
						kind: Kind::Load { dst, src, offset },
					},
					"s" => Inst {
						n,
						kind: Kind::Store { dst, src, offset },
					},
					_ => unreachable!(),
				}
			},

			// TODO: bc{n}[f, t]
			_ => return Err(ParseError::UnknownMnemonic),
		};

		Ok(inst)
	}
}

impl<'a> InstDisplay<'a> for Inst {
	type Args = impl Iterator<Item = InstFmtArg<'a>>;
	type Mnemonic = impl fmt::Display;

	fn mnemonic<Ctx: DisplayCtx>(&'a self, _ctx: &Ctx) -> Self::Mnemonic {
		/// Wrapper necessary for `impl Trait` to work without using `Ctx`.
		fn wrapper(n: u32, kind: Kind) -> impl fmt::Display {
			dcb_util::DisplayWrapper::new(move |f| match kind {
				Kind::CopN { .. } => write!(f, "cop{n}"),
				Kind::MoveFrom { kind, .. } => match kind {
					RegisterKind::Control => write!(f, "cfc{n}"),
					RegisterKind::Data => write!(f, "mfc{n}"),
				},
				Kind::MoveTo { kind, .. } => match kind {
					RegisterKind::Data => write!(f, "mtc{n}"),
					RegisterKind::Control => write!(f, "ctc{n}"),
				},
				Kind::Branch { on, .. } => match on {
					true => write!(f, "bc{n}f"),
					false => write!(f, "bc{n}t"),
				},
				Kind::Load { .. } => write!(f, "lwc{n}"),
				Kind::Store { .. } => write!(f, "swc{n}"),
			})
		}

		wrapper(self.n, self.kind)
	}

	#[auto_enums::auto_enum(Iterator)]
	fn args<Ctx: DisplayCtx>(&'a self, _ctx: &Ctx) -> Self::Args {
		let &Self { kind, .. } = self;
		match kind {
			Kind::CopN { imm } => array::IntoIter::new([InstFmtArg::literal(imm)]),
			Kind::MoveFrom { dst, src, .. } => array::IntoIter::new([InstFmtArg::Register(dst), InstFmtArg::literal(src)]),
			Kind::MoveTo { dst, src, .. } => array::IntoIter::new([InstFmtArg::Register(src), InstFmtArg::literal(dst)]),
			Kind::Branch { offset, .. } => array::IntoIter::new([InstFmtArg::literal(offset)]),
			Kind::Load { dst, src, offset } => array::IntoIter::new([InstFmtArg::literal(dst), InstFmtArg::register_offset(src, offset)]),
			Kind::Store { dst, src, offset } => array::IntoIter::new([InstFmtArg::literal(dst), InstFmtArg::register_offset(src, offset)]),
		}
	}
}

impl ModifiesReg for Inst {
	fn modifies_reg(&self, reg: Register) -> bool {
		match self.kind {
			Kind::MoveFrom { dst, .. } => dst == reg,
			Kind::CopN { .. } | Kind::MoveTo { .. } | Kind::Branch { .. } | Kind::Load { .. } | Kind::Store { .. } => false,
		}
	}
}
