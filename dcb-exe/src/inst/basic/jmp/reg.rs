//! Jump register instructions

// Imports
use crate::{
	inst::{
		basic::{Decode, Encode, ModifiesReg},
		exec::{ExecCtx, ExecError, Executable},
		parse::LineArg,
		DisplayCtx, InstDisplay, InstFmtArg, Parsable, ParseCtx, ParseError, Register,
	},
	Pos,
};

/// Jmp register instruction kind
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Kind {
	/// Jump
	Jump,

	/// Jump and link
	JumpLink(Register),
}

impl Kind {
	/// Returns this kind's mnemonic
	#[must_use]
	pub const fn mnemonic(self) -> &'static str {
		match self {
			Self::Jump => "jr",
			Self::JumpLink(_) => "jalr",
		}
	}
}

/// Jmp register instructions
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Inst {
	/// Target
	pub target: Register,

	/// Kind
	pub kind: Kind,
}

impl Decode for Inst {
	#[bitmatch::bitmatch]
	fn decode(raw: u32) -> Option<Self> {
		let [s, d, f] = #[bitmatch]
		match raw {
			"000000_sssss_?????_ddddd_?????_00100f" => [s, d, f],
			_ => return None,
		};

		let kind = match f {
			0 => Kind::Jump,
			1 => Kind::JumpLink(Register::new(d)?),
			_ => return None,
		};
		let target = Register::new(s)?;

		Some(Self { target, kind })
	}
}

impl Encode for Inst {
	#[bitmatch::bitmatch]
	fn encode(&self) -> u32 {
		let (f, d): (u32, u32) = match self.kind {
			Kind::Jump => (0, 0),
			Kind::JumpLink(reg) => (1, reg.idx()),
		};
		let s = self.target.idx();

		bitpack!("000000_sssss_?????_ddddd_?????_00100f")
	}
}


impl<'a> Parsable<'a> for Inst {
	fn parse<Ctx: ?Sized + ParseCtx<'a>>(
		mnemonic: &'a str, args: &'a [LineArg], _ctx: &Ctx,
	) -> Result<Self, ParseError> {
		let (target, kind) = match mnemonic {
			"jr" => match *args {
				[LineArg::Register(target)] => (target, Kind::Jump),
				_ => return Err(ParseError::InvalidArguments),
			},

			"jalr" => match *args {
				[LineArg::Register(target)] => (target, Kind::JumpLink(Register::Ra)),
				[LineArg::Register(target), LineArg::Register(reg)] => (target, Kind::JumpLink(reg)),
				_ => return Err(ParseError::InvalidArguments),
			},

			_ => return Err(ParseError::UnknownMnemonic),
		};

		Ok(Self { target, kind })
	}
}

impl<'a> InstDisplay<'a> for Inst {
	type Mnemonic = &'static str;

	type Args = impl IntoIterator<Item = InstFmtArg<'a>>;

	fn mnemonic<Ctx: DisplayCtx>(&'a self, _ctx: &Ctx) -> Self::Mnemonic {
		self.kind.mnemonic()
	}

	#[auto_enums::auto_enum(Iterator)]
	fn args<Ctx: DisplayCtx>(&'a self, _ctx: &Ctx) -> Self::Args {
		let &Self { target, kind } = self;

		match kind {
			// If linking with `$ra`, don't output it
			Kind::Jump | Kind::JumpLink(Register::Ra) => [InstFmtArg::Register(target)].into_iter(),
			Kind::JumpLink(reg) => [InstFmtArg::Register(target), InstFmtArg::Register(reg)].into_iter(),
		}
	}
}

impl ModifiesReg for Inst {
	fn modifies_reg(&self, _reg: Register) -> bool {
		false
	}
}

impl Executable for Inst {
	fn exec<Ctx: ExecCtx>(&self, state: &mut Ctx) -> Result<(), ExecError> {
		// If we should link, set `$ra`
		if let Kind::JumpLink(link) = self.kind {
			state.store_reg(link, (state.pc() + 8u32).0);
		}

		// Then set the jump
		state.queue_jump(Pos(state.load_reg(self.target)))
	}
}
