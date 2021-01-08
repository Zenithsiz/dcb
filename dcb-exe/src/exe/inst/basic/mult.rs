//! Multiplications

// Imports
use crate::exe::inst::Register;
use std::fmt;

/// Operation func
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
pub struct MultRaw {
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
pub enum MultInst {
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
		dst: Register,

		/// Destination
		src: MultReg,
	},
}

impl MultInst {
	/// Decodes this instruction
	#[must_use]
	#[rustfmt::skip]
	pub fn decode(raw: MultRaw) -> Option<Self> {
		Some(match raw.f {
			0x10 => Self::MoveFrom { dst: Register::new(raw.d)?, src: MultReg::Hi },
			0x12 => Self::MoveFrom { dst: Register::new(raw.d)?, src: MultReg::Lo },

			0x11 => Self::MoveTo { dst: Register::new(raw.s)?, src: MultReg::Hi },
			0x13 => Self::MoveTo { dst: Register::new(raw.s)?, src: MultReg::Lo },

			0x18 => Self::Mult { kind: MultKind::Mult, mode: MultMode::  Signed, lhs: Register::new(raw.s)?, rhs: Register::new(raw.t)? },
			0x19 => Self::Mult { kind: MultKind::Mult, mode: MultMode::Unsigned, lhs: Register::new(raw.s)?, rhs: Register::new(raw.t)? },
			0x1a => Self::Mult { kind: MultKind::Div , mode: MultMode::  Signed, lhs: Register::new(raw.s)?, rhs: Register::new(raw.t)? },
			0x1b => Self::Mult { kind: MultKind::Div , mode: MultMode::Unsigned, lhs: Register::new(raw.s)?, rhs: Register::new(raw.t)? },

			_ => return None,
		})
	}

	/// Encodes this instruction
	#[must_use]
	pub const fn encode(self) -> MultRaw {
		match self {
			Self::Mult { kind, mode, lhs, rhs } => MultRaw {
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
			Self::MoveFrom { dst, src } => MultRaw {
				s: 0,
				t: 0,
				d: dst.idx(),
				f: match src {
					MultReg::Hi => 0x10,
					MultReg::Lo => 0x12,
				},
			},
			Self::MoveTo { dst: src, src: dst } => MultRaw {
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

impl fmt::Display for MultInst {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			#[rustfmt::skip]
			Self::Mult { kind, mode, lhs, rhs } => match (kind, mode) {
				(MultKind::Mult, MultMode::  Signed) => write!(f, "mult {lhs}, {rhs}"),
				(MultKind::Mult, MultMode::Unsigned) => write!(f, "multu {lhs}, {rhs}"),
				(MultKind::Div , MultMode::  Signed) => write!(f, "div {lhs}, {rhs}"),
				(MultKind::Div , MultMode::Unsigned) => write!(f, "diu {lhs}, {rhs}"),
			},
			Self::MoveFrom { dst, src } => match src {
				MultReg::Hi => write!(f, "mfhi {dst}"),
				MultReg::Lo => write!(f, "mflo {dst}"),
			},
			Self::MoveTo { dst: src, src: dst } => match dst {
				MultReg::Hi => write!(f, "mthi {src}"),
				MultReg::Lo => write!(f, "mtlo {src}"),
			},
		}
	}
}
