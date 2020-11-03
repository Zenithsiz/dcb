//! Multiplications

// Imports
use crate::game::exe::instruction::Register;
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
		dest: Register,

		/// Source
		source: MultReg,
	},

	/// Move to
	MoveTo {
		/// Source
		source: Register,

		/// Destination
		dest: MultReg,
	},
}

impl MultInst {
	/// Decodes this instruction
	#[must_use]
	pub fn decode(raw: MultRaw) -> Option<Self> {
		#[rustfmt::skip]
		Some(match raw.f {
			0x10 => Self::MoveFrom { dest: Register::new(raw.d)?, source: MultReg::Hi },
			0x12 => Self::MoveFrom { dest: Register::new(raw.d)?, source: MultReg::Lo },

			0x11 => Self::MoveTo { source: Register::new(raw.s)?, dest: MultReg::Hi },
			0x13 => Self::MoveTo { source: Register::new(raw.s)?, dest: MultReg::Lo },

			0x18 => Self::Mult { kind: MultKind::Mult, mode: MultMode::  Signed, lhs: Register::new(raw.s)?, rhs: Register::new(raw.t)? },
			0x19 => Self::Mult { kind: MultKind::Mult, mode: MultMode::Unsigned, lhs: Register::new(raw.s)?, rhs: Register::new(raw.t)? },
			0x1a => Self::Mult { kind: MultKind::Div , mode: MultMode::  Signed, lhs: Register::new(raw.s)?, rhs: Register::new(raw.t)? },
			0x1b => Self::Mult { kind: MultKind::Div , mode: MultMode::Unsigned, lhs: Register::new(raw.s)?, rhs: Register::new(raw.t)? },

			_ => return None,
		})
	}

	/// Encodes this instruction
	#[must_use]
	pub fn encode(self) -> MultRaw {
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
			Self::MoveFrom { dest, source } => MultRaw {
				s: 0,
				t: 0,
				d: dest.idx(),
				f: match source {
					MultReg::Hi => 0x10,
					MultReg::Lo => 0x12,
				},
			},
			Self::MoveTo { source, dest } => MultRaw {
				s: source.idx(),
				t: 0,
				d: 0,
				f: match dest {
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
			Self::MoveFrom { dest, source } => match source {
				MultReg::Hi => write!(f, "mfhi {dest}"),
				MultReg::Lo => write!(f, "mflo {dest}"),
			},
			Self::MoveTo { source, dest } => match dest {
				MultReg::Hi => write!(f, "mthi {source}"),
				MultReg::Lo => write!(f, "mtlo {source}"),
			},
		}
	}
}
