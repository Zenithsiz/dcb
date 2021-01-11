//! Load instructions

// Imports
use crate::{
	exe::inst::{basic, InstFmt, InstSize, Register},
	Pos,
};
use int_conv::{Join, SignExtended, Signed};

use super::Decodable;

/// Load pseudo instructions
///
/// Alias for
/// ```mips
/// lui $dst, {hi}
/// l* $dst, {lo}($dst)
/// ```
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Inst {
	/// Destination register
	pub dst: Register,

	/// Target
	pub target: Pos,

	/// Kind
	pub kind: basic::load::Kind,
}

impl Decodable for Inst {
	fn decode(mut insts: impl Iterator<Item = basic::Inst> + Clone) -> Option<Self> {
		let inst = match insts.next()? {
			basic::Inst::Lui(lui) => match insts.next()? {
				basic::Inst::Load(load) if load.dst == lui.dst && load.dst == load.src => Self {
					dst:    lui.dst,
					target: Pos((u32::join(0, lui.value).as_signed() + load.offset.sign_extended::<i32>()).as_unsigned()),
					kind:   load.kind,
				},
				_ => return None,
			},
			_ => return None,
		};

		Some(inst)
	}
}

impl InstSize for Inst {
	fn size(&self) -> usize {
		8
	}
}

impl InstFmt for Inst {
	fn mnemonic(&self) -> &'static str {
		self.kind.mnemonic()
	}

	fn fmt(&self, _pos: crate::Pos, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let Self { dst, kind, target } = self;
		let mnemonic = kind.mnemonic();

		write!(f, "{mnemonic} {dst}, {target}")
	}
}
