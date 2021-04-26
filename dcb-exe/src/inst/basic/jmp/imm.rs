//! Jump immediate instructions

// Imports
use crate::{
	inst::{
		basic::{Decodable, Encodable, ModifiesReg},
		parse::LineArg,
		InstTarget, InstTargetFmt, Parsable, ParseCtx, ParseError, Register,
	},
	Pos,
};

/// Jmp immediate instruction kind
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Kind {
	/// Jump
	Jump,

	/// Jump and link
	JumpLink,
}

impl Kind {
	/// Returns this kind's mnemonic
	#[must_use]
	pub const fn mnemonic(self) -> &'static str {
		match self {
			Self::Jump => "j",
			Self::JumpLink => "jal",
		}
	}
}

/// Jmp register instructions
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Inst {
	/// Immediate
	pub imm: u32,

	/// Kind
	pub kind: Kind,
}

impl Inst {
	/// Returns the target using an immediate
	#[must_use]
	pub fn target_of(imm: u32, pos: Pos) -> Pos {
		(pos & 0xf0000000) + imm * 4
	}
}

impl Decodable for Inst {
	#[bitmatch::bitmatch]
	fn decode(raw: u32) -> Option<Self> {
		let [p, i] = #[bitmatch]
		match raw {
			"00001p_iiiii_iiiii_iiiii_iiiii_iiiiii" => [p, i],
			_ => return None,
		};

		let kind = match p {
			0 => Kind::Jump,
			1 => Kind::JumpLink,
			_ => unreachable!(),
		};

		Some(Self { imm: i, kind })
	}
}

impl Encodable for Inst {
	#[bitmatch::bitmatch]
	fn encode(&self) -> u32 {
		let p: u32 = match self.kind {
			Kind::Jump => 0,
			Kind::JumpLink => 1,
		};
		let i = self.imm;

		bitpack!("00001p_iiiii_iiiii_iiiii_iiiii_iiiiii")
	}
}

impl Parsable for Inst {
	fn parse<Ctx: ?Sized + ParseCtx>(mnemonic: &str, args: &[LineArg], ctx: &Ctx) -> Result<Self, ParseError> {
		let (pos, kind) = match mnemonic {
			"j" => match args {
				[arg] => (ctx.arg_pos(arg)?, Kind::Jump),
				_ => return Err(ParseError::InvalidArguments),
			},

			"jal" => match args {
				[arg] => (ctx.arg_pos(arg)?, Kind::JumpLink),
				_ => return Err(ParseError::InvalidArguments),
			},

			_ => return Err(ParseError::UnknownMnemonic),
		};

		// If the position isn't word aligned, return Err
		if !pos.is_word_aligned() {
			return Err(ParseError::TargetAlign);
		}

		// Else get our imm from it
		let imm = (pos.0 & 0x0fff_ffff) / 4;

		Ok(Self { imm, kind })
	}
}

impl InstTarget for Inst {
	fn target(&self, pos: Pos) -> Pos {
		Self::target_of(self.imm, pos)
	}
}

impl InstTargetFmt for Inst {
	fn fmt(&self, _pos: Pos, target: impl std::fmt::Display, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let mnemonic = self.kind.mnemonic();

		write!(f, "{mnemonic} {target}")
	}
}

impl ModifiesReg for Inst {
	fn modifies_reg(&self, _reg: Register) -> bool {
		false
	}
}
