//! System calls

// Imports
use int_conv::{Truncated, ZeroExtended};
use std::{convert::TryFrom, fmt};

/// Sys instruction func
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[derive(num_enum::IntoPrimitive, num_enum::TryFromPrimitive)]
#[repr(u8)]
pub enum SysFunc {
	/// Sys
	Sys   = 0xc,

	/// Break
	Break = 0xd,
}

/// Raw representation
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct SysRaw {
	/// Rs
	pub s: u32,

	/// Rt
	pub t: u32,

	/// Rd
	pub d: u32,

	/// Immediate
	pub i: u32,

	/// Func
	pub f: u32,
}

/// Syscall instructions
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct SysInst {
	/// Comment
	pub comment: u32,

	/// Function
	pub func: SysFunc,
}

impl SysInst {
	/// Decodes this instruction
	#[must_use]
	pub fn decode(SysRaw { t, d, s, i, f }: SysRaw) -> Option<Self> {
		let s = s.truncated::<u8>();
		let t = t.truncated::<u8>();
		let d = d.truncated::<u8>();
		let i = i.truncated::<u8>();
		let comment = u32::from_be_bytes([s, t, d, i]);

		let func = SysFunc::try_from(f.truncated::<u8>()).ok()?;

		Some(Self { comment, func })
	}

	/// Encodes this instruction
	#[must_use]
	#[allow(clippy::many_single_char_names)] // `Raw` has single character names
	pub fn encode(self) -> SysRaw {
		let [s, t, d, i] = self.comment.to_be_bytes();
		let s = s.zero_extended::<u32>();
		let t = t.zero_extended::<u32>();
		let d = d.zero_extended::<u32>();
		let i = i.zero_extended::<u32>();
		let f = u8::from(self.func).zero_extended::<u32>();

		SysRaw { s, t, d, i, f }
	}
}

impl fmt::Display for SysInst {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let Self { func, comment } = self;

		let mnemonic = match func {
			SysFunc::Sys => "sys",
			SysFunc::Break => "break",
		};

		write!(f, "{mnemonic} {comment}")
	}
}
