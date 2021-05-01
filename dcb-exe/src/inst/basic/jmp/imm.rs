//! Jump immediate instructions

// Imports
use crate::{
	inst::{
		basic::{Decode, Encode, ModifiesReg},
		exec::{ExecError, ExecState, Executable},
		parse::LineArg,
		DisplayCtx, InstDisplay, InstFmtArg, Parsable, ParseCtx, ParseError, Register,
	},
	Pos,
};
use std::array;

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
	/// Returns this instruction's target
	#[must_use]
	pub fn target(self, pos: Pos) -> Pos {
		Self::target_of(self.imm, pos)
	}

	/// Returns the target using an immediate
	#[must_use]
	pub fn target_of(imm: u32, pos: Pos) -> Pos {
		(pos & 0xf0000000) + imm * 4
	}
}

impl Decode for Inst {
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

impl Encode for Inst {
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

impl<'a> Parsable<'a> for Inst {
	fn parse<Ctx: ?Sized + ParseCtx>(mnemonic: &'a str, args: &'a [LineArg], ctx: &'a Ctx) -> Result<Self, ParseError> {
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

impl<'a> InstDisplay<'a> for Inst {
	type Args = array::IntoIter<InstFmtArg<'a>, 1>;
	type Mnemonic = &'static str;

	fn mnemonic<Ctx: DisplayCtx>(&'a self, _ctx: &Ctx) -> Self::Mnemonic {
		self.kind.mnemonic()
	}

	fn args<Ctx: DisplayCtx>(&'a self, ctx: &Ctx) -> Self::Args {
		array::IntoIter::new([InstFmtArg::Target(Self::target_of(self.imm, ctx.cur_pos()))])
	}
}

impl ModifiesReg for Inst {
	fn modifies_reg(&self, _reg: Register) -> bool {
		false
	}
}

impl Executable for Inst {
	fn exec(&self, state: &mut ExecState) -> Result<(), ExecError> {
		// If we should link, set `$ra`
		if matches!(self.kind, Kind::JumpLink) {
			state[Register::Ra] = (state.pc() + 8u32).0;
		}

		// Then set the jump
		state.set_jump(self.target(state.pc()))
	}
}
