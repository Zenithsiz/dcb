//! Multiplications

// Imports
use crate::exe::inst::{
	basic::{Decodable, Encodable},
	InstFmt, Register,
};

/// Operation kind
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum MultKind {
	/// Multiplication
	Mult,

	/// Division
	Div,
}

/// Operation mode
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum MultMode {
	/// Signed
	Signed,

	/// Unsigned
	Unsigned,
}

/// Multiplication register
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum MultReg {
	/// Lo
	Lo,

	/// Hi
	Hi,
}

/// Raw representation
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Raw {
	/// Rs
	pub s: u32,

	/// Rt
	pub t: u32,

	/// Rd
	pub d: u32,

	/// Func (bottom 4 bits)
	pub f: u32,
}

/// Multiplication instructions
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Inst {
	/// Multiplication
	Mult {
		/// Kind
		kind: MultKind,

		/// Mode
		mode: MultMode,

		/// Lhs argument
		lhs: Register,

		/// Rhs argument
		rhs: Register,
	},

	/// Move from
	MoveFrom {
		/// Destination
		dst: Register,

		/// Source
		src: MultReg,
	},

	/// Move to
	MoveTo {
		/// Source
		src: Register,

		/// Destination
		dst: MultReg,
	},
}

impl Decodable for Inst {
	type Raw = Raw;

	#[rustfmt::skip]
	fn decode(raw: Self::Raw) -> Option<Self> {
		Some(match raw.f {
			// 00x0
			0x0 => Self::MoveFrom { dst: Register::new(raw.d)?, src: MultReg::Hi },
			0x2 => Self::MoveFrom { dst: Register::new(raw.d)?, src: MultReg::Lo },

			// 00x1
			0x1 => Self::MoveTo { src: Register::new(raw.s)?, dst: MultReg::Hi },
			0x3 => Self::MoveTo { src: Register::new(raw.s)?, dst: MultReg::Lo },

			// 10xx
			0x8 => Self::Mult { kind: MultKind::Mult, mode: MultMode::  Signed, lhs: Register::new(raw.s)?, rhs: Register::new(raw.t)? },
			0x9 => Self::Mult { kind: MultKind::Mult, mode: MultMode::Unsigned, lhs: Register::new(raw.s)?, rhs: Register::new(raw.t)? },
			0xa => Self::Mult { kind: MultKind::Div , mode: MultMode::  Signed, lhs: Register::new(raw.s)?, rhs: Register::new(raw.t)? },
			0xb => Self::Mult { kind: MultKind::Div , mode: MultMode::Unsigned, lhs: Register::new(raw.s)?, rhs: Register::new(raw.t)? },

			_ => return None,
		})
	}
}

impl Encodable for Inst {
	fn encode(&self) -> Raw {
		match self {
			Self::Mult { kind, mode, lhs, rhs } => Raw {
				s: lhs.idx(),
				t: rhs.idx(),
				d: 0,

				f: match (kind, mode) {
					(MultKind::Mult, MultMode::Signed) => 0x8,
					(MultKind::Mult, MultMode::Unsigned) => 0x9,
					(MultKind::Div, MultMode::Signed) => 0xa,
					(MultKind::Div, MultMode::Unsigned) => 0xb,
				},
			},
			Self::MoveFrom { dst, src } => Raw {
				s: 0,
				t: 0,
				d: dst.idx(),
				f: match src {
					MultReg::Hi => 0x0,
					MultReg::Lo => 0x2,
				},
			},
			Self::MoveTo { dst, src } => Raw {
				s: src.idx(),
				t: 0,
				d: 0,
				f: match dst {
					MultReg::Hi => 0x1,
					MultReg::Lo => 0x3,
				},
			},
		}
	}
}

impl InstFmt for Inst {
	fn mnemonic(&self) -> &'static str {
		match self {
			#[rustfmt::skip]
			Self::Mult { kind, mode, .. } => match (kind, mode) {
				(MultKind::Mult, MultMode::  Signed) => "mult",
				(MultKind::Mult, MultMode::Unsigned) => "multu",
				(MultKind::Div , MultMode::  Signed) => "div",
				(MultKind::Div , MultMode::Unsigned) => "diu",
			},
			Self::MoveFrom { src, .. } => match src {
				MultReg::Hi => "mfhi",
				MultReg::Lo => "mflo",
			},
			Self::MoveTo { dst, .. } => match dst {
				MultReg::Hi => "mthi",
				MultReg::Lo => "mtlo",
			},
		}
	}

	fn fmt(&self, _pos: crate::Pos, _bytes: &[u8], f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let mnemonic = self.mnemonic();
		match self {
			#[rustfmt::skip]
			Self::Mult { lhs, rhs, .. } => write!(f, "{mnemonic} {lhs}, {rhs}"),
			Self::MoveFrom { dst: arg, .. } | Self::MoveTo { src: arg, .. } => write!(f, "{mnemonic} {arg}"),
		}
	}
}
