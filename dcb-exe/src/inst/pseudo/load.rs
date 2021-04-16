//! Load instructions

// Imports
use super::Decodable;
use crate::{
	inst::{basic, InstSize, InstTarget, InstTargetFmt, Register},
	Pos,
};
use int_conv::{Join, SignExtended, Signed};

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

impl InstTarget for Inst {
	fn target(&self, _pos: Pos) -> Pos {
		self.target
	}
}

impl InstTargetFmt for Inst {
	fn fmt(&self, _pos: crate::Pos, target: impl std::fmt::Display, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let Self { dst, kind, .. } = self;
		let mnemonic = kind.mnemonic();

		write!(f, "{mnemonic} {dst}, {target}")
	}
}
