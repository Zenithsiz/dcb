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

	/// Func
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
			0x10 => Self::MoveFrom { dst: Register::new(raw.d)?, src: MultReg::Hi },
			0x12 => Self::MoveFrom { dst: Register::new(raw.d)?, src: MultReg::Lo },

			0x11 => Self::MoveTo { src: Register::new(raw.s)?, dst: MultReg::Hi },
			0x13 => Self::MoveTo { src: Register::new(raw.s)?, dst: MultReg::Lo },

			0x18 => Self::Mult { kind: MultKind::Mult, mode: MultMode::  Signed, lhs: Register::new(raw.s)?, rhs: Register::new(raw.t)? },
			0x19 => Self::Mult { kind: MultKind::Mult, mode: MultMode::Unsigned, lhs: Register::new(raw.s)?, rhs: Register::new(raw.t)? },
			0x1a => Self::Mult { kind: MultKind::Div , mode: MultMode::  Signed, lhs: Register::new(raw.s)?, rhs: Register::new(raw.t)? },
			0x1b => Self::Mult { kind: MultKind::Div , mode: MultMode::Unsigned, lhs: Register::new(raw.s)?, rhs: Register::new(raw.t)? },

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
					(MultKind::Mult, MultMode::Signed) => 0x18,
					(MultKind::Mult, MultMode::Unsigned) => 0x19,
					(MultKind::Div, MultMode::Signed) => 0x1a,
					(MultKind::Div, MultMode::Unsigned) => 0x1b,
				},
			},
			Self::MoveFrom { dst, src } => Raw {
				s: 0,
				t: 0,
				d: dst.idx(),
				f: match src {
					MultReg::Hi => 0x10,
					MultReg::Lo => 0x12,
				},
			},
			Self::MoveTo { dst, src } => Raw {
				s: src.idx(),
				t: 0,
				d: 0,
				f: match dst {
					MultReg::Hi => 0x11,
					MultReg::Lo => 0x13,
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
			Self::Mult { kind, mode, lhs, rhs } => match (kind, mode) {
				(MultKind::Mult, MultMode::  Signed) => write!(f, "{mnemonic} {lhs}, {rhs}"),
				(MultKind::Mult, MultMode::Unsigned) => write!(f, "{mnemonic} {lhs}, {rhs}"),
				(MultKind::Div , MultMode::  Signed) => write!(f, "{mnemonic} {lhs}, {rhs}"),
				(MultKind::Div , MultMode::Unsigned) => write!(f, "{mnemonic} {lhs}, {rhs}"),
			},
			Self::MoveFrom { dst, src } => match src {
				MultReg::Hi => write!(f, "{mnemonic} {dst}"),
				MultReg::Lo => write!(f, "{mnemonic} {dst}"),
			},
			Self::MoveTo { dst, src } => match dst {
				MultReg::Hi => write!(f, "{mnemonic} {src}"),
				MultReg::Lo => write!(f, "{mnemonic} {src}"),
			},
		}
	}
}
