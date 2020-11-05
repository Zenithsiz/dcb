//! Load instructions

// Imports
use crate::game::exe::instruction::{
	basic::{load::LoadKind, LoadInst, LuiInst},
	Register,
};
use int_conv::{Join, SignExtended, Signed};

/// Load pseudo instructions
///
/// Alias for
/// ```mips
/// lui $rx, {hi}
/// l* $rx, {lo}($rx)
/// ```
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[derive(derive_more::Display)]
#[display(fmt = "{} {dst}, {target:#x}", "kind.mnemonic()")]
pub struct LoadPseudoInst {
	/// Destination register
	pub dst: Register,

	/// Target
	pub target: u32,

	/// Kind
	pub kind: LoadKind,
}

impl LoadPseudoInst {
	/// Decodes this pseudo instruction
	#[must_use]
	pub fn decode(lui: LuiInst, load: LoadInst) -> Self {
		Self {
			dst:    load.dst,
			target: (u32::join(0, lui.value).as_signed() + load.offset.sign_extended::<i32>()).as_unsigned(),
			kind:   load.kind,
		}
	}
}
