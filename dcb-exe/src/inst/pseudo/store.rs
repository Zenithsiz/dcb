//! Store instructions

// Imports
use super::{Decodable, Encodable};
use crate::{
	inst::{basic, InstSize, InstTarget, InstTargetFmt, Register},
	Pos,
};
use int_conv::{Join, SignExtended, Signed, Split};

/// Store pseudo instructions
///
/// Alias for
/// ```mips
/// lui $at, {hi}
/// s* $dst, {lo}($at)
/// ```
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Inst {
	/// Value register
	pub value: Register,

	/// Target
	pub target: Pos,

	/// Kind
	pub kind: basic::store::Kind,
}

impl Decodable for Inst {
	fn decode(mut insts: impl Iterator<Item = basic::Inst> + Clone) -> Option<Self> {
		let inst = match insts.next()? {
			basic::Inst::Lui(lui) if lui.dst == Register::At => match insts.next()? {
				basic::Inst::Store(store) if store.addr == Register::At => Self {
					value:  store.value,
					target: Pos((u32::join(0, lui.value).as_signed() + store.offset.sign_extended::<i32>()).as_unsigned()),
					kind:   store.kind,
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
				dst:   Register::At,
				value: hi,
			}),
			basic::Inst::Store(basic::store::Inst {
				value:  self.value,
				addr:   Register::At,
				offset: lo.as_signed(),
				kind:   self.kind,
			}),
		])
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
		let Self { value, kind, .. } = self;
		let mnemonic = kind.mnemonic();

		write!(f, "{mnemonic} {value}, {target}")
	}
}
