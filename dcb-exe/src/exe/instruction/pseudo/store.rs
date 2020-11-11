//! Store instructions

// Imports
use crate::exe::instruction::{
	basic::{store::StoreKind, InstIter},
	BasicInst, Register,
};
use int_conv::{Join, SignExtended, Signed};

/// Store pseudo instructions
///
/// Alias for
/// ```mips
/// lui $at, {hi}
/// s* $dst, {lo}($at)
/// ```
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[derive(derive_more::Display)]
#[display(fmt = "{} {dst}, {target:#x}", "kind.mnemonic()")]
pub struct StorePseudoInst {
	/// Destination register
	pub dst: Register,

	/// Target
	pub target: u32,

	/// Kind
	pub kind: StoreKind,
}

impl StorePseudoInst {
	/// Decodes this pseudo instruction
	#[must_use]
	pub fn decode(iter: InstIter<'_, impl Iterator<Item = u32> + Clone>) -> Option<Self> {
		let peeker = iter.peeker();
		let inst = match peeker.next()?? {
			BasicInst::Lui(lui) => match peeker.next()?? {
				BasicInst::Store(store) => Self {
					dst:    store.dst,
					target: (u32::join(0, lui.value).as_signed() + store.offset.sign_extended::<i32>()).as_unsigned(),
					kind:   store.kind,
				},
				_ => return None,
			},
			_ => return None,
		};

		peeker.apply();
		Some(inst)
	}
}
