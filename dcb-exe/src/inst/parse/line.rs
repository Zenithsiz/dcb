//! Line parsing

// Modules
pub mod error;

use std::str::FromStr;

// Exports
pub use error::{ParseLineError, ReadArgError, ReadFuncError, ReadLiteralError, ReadNameError};

// Imports
use crate::inst::Register;

/// A line
#[derive(PartialEq, Clone, Debug)]
pub struct Line {
	/// Labels
	pub labels: Vec<LineLabel>,

	/// Instruction
	pub inst: Option<LineInst>,
}

impl Line {
	/// Parses a line from a string
	pub fn parse(line: &str) -> Result<Self, ParseLineError> {
		let mut line = line.trim();

		// Read all labels and then the mnemonic
		let mut labels = vec![];
		let mnemonic = loop {
			// If the line starts with a comment or is empty, return all labels
			if line.starts_with('#') || line.is_empty() {
				return Ok(Self { labels, inst: None });
			}

			// Read a name
			let (name, rest) = self::read_name(line)?;

			// Check the character after the name
			let mut rest = rest.chars();
			match rest.next() {
				// If we got ':', add a label and continue
				Some(':') => {
					line = rest.as_str().trim_start();
					let label = LineLabel { name: name.to_owned() };
					labels.push(label);
					continue;
				},

				// If we got '#' or eof, we got a mnemonic with no arguments
				Some('#') | None => {
					return Ok(Self {
						labels,
						inst: Some(LineInst {
							mnemonic: name.to_owned(),
							args:     vec![],
						}),
					});
				},

				// If we got a space or eof, we found the mnemonic.
				// On a space, break and read arguments
				Some(' ') => {
					line = rest.as_str().trim_start();
					break name.to_owned();
				},
				_ => return Err(ParseLineError::InvalidNameSuffix),
			}
		};

		// Then read all arguments
		let mut args = vec![];
		loop {
			// Read an argument
			let (arg, rest) = self::read_arg(line)?;
			args.push(arg);

			// Check the character after the argument
			let rest = rest.trim_start();
			let mut rest = rest.chars();
			match rest.next() {
				// If we got ',', continue reading
				Some(',') => {
					line = rest.as_str().trim_start();
					continue;
				},

				// If we got eof or a comment, return
				Some('#') | None => {
					let inst = Some(LineInst { mnemonic, args });
					return Ok(Self { labels, inst });
				},

				_ => return Err(ParseLineError::InvalidArgSuffix),
			}
		}
	}
}

/// Line label
#[derive(PartialEq, Clone, Debug)]
pub struct LineLabel {
	/// Name
	pub name: String,
}

/// Line instruction
#[derive(PartialEq, Clone, Debug)]
pub struct LineInst {
	/// Mnemonic
	pub mnemonic: String,

	/// Args
	pub args: Vec<LineArg>,
}

/// Line argument
#[derive(PartialEq, Clone, Debug)]
pub enum LineArg {
	/// String
	/// `"<string>"`
	String(String),

	/// Register
	/// `<reg>`
	Register(Register),

	/// Mnemonic
	/// `^<mnemonic>`
	Mnemonic(String),

	/// Register offset
	/// `<offset>(<reg>)`
	RegisterOffset {
		/// The register
		register: Register,

		/// The offset
		offset: LineArgExpr,
	},

	/// Expression
	Expr(LineArgExpr),
}

/// Line argument expression
#[derive(PartialEq, Clone, Debug)]
pub enum LineArgExpr {
	/// Literal
	Literal(i64),

	/// Label
	/// `<name>(`+<offset>`)?(@<func>)?`
	Label {
		/// The label
		label: String,

		/// The offset
		offset: Option<i64>,

		/// The function
		func: Option<LineLabelFunc>,
	},
}

/// Line label functions
#[derive(PartialEq, Clone, Debug)]
pub enum LineLabelFunc {
	/// Lower 16 bits of address
	AddrLo,

	/// Higher 16 bits of address
	AddrHi,
}

/// Reads a name
fn read_name(s: &str) -> Result<(&str, &str), ReadNameError> {
	// Make sure the first character is valid
	let mut chars = s.char_indices();
	match chars.next() {
		Some((_, c)) if self::is_valid_first_name_char(c) => (),
		Some(_) => return Err(ReadNameError::StartChar),
		None => return Err(ReadNameError::Empty),
	}

	// Then keep consuming until we get a non-valid continuation character
	let idx = loop {
		match chars.next() {
			Some((_, c)) if self::is_valid_cont_name_char(c) => continue,
			Some((idx, _)) => break idx,
			None => break s.len(),
		};
	};

	Ok((&s[..idx], &s[idx..]))
}

/// Reads an argument
fn read_arg(s: &str) -> Result<(LineArg, &str), ReadArgError> {
	let mut chars = s.char_indices();
	match chars.next() {
		// If we got '$', it's a register
		Some((_, '$')) => self::read_reg(s).map(|(reg, rest)| (LineArg::Register(reg), rest)),

		// If we got '"', it's a string
		Some((_, '"')) => self::read_string(s).map(|(string, rest)| (LineArg::String(string), rest)),

		// If we got '^', it's a mnemonic
		Some((_, '^')) => self::read_name(chars.as_str())
			.map(|(name, rest)| (LineArg::Mnemonic(name.to_owned()), rest))
			.map_err(ReadArgError::ReadLabel),

		// Else try to read an expression
		Some(_) => {
			// Read the expression
			let (expr, rest) = self::read_expr(s)?;

			// Then check if we have a register
			let rest = rest.trim_start();
			match rest.strip_prefix('(') {
				// If the rest starts with '(', read it as a register offset
				Some(rest) => match rest.split_once(')') {
					Some((reg, rest)) => {
						// Parse the register
						// If we have leftover tokens after reading it, return Err
						let reg = reg.trim();
						let (reg, reg_rest) = self::read_reg(reg)?;
						if !reg_rest.is_empty() {
							return Err(ReadArgError::RegisterOffsetLeftoverTokens);
						}

						Ok((
							LineArg::RegisterOffset {
								register: reg,
								offset:   expr,
							},
							rest,
						))
					},
					None => Err(ReadArgError::MissingRegisterOffsetDelimiter),
				},
				None => Ok((LineArg::Expr(expr), rest)),
			}
		},

		None => Err(ReadArgError::Empty),
	}
}

/// Reads an expression
pub fn read_expr(s: &str) -> Result<(LineArgExpr, &str), ReadArgError> {
	let mut chars = s.char_indices();
	match chars.next() {
		// If it's numeric, 0..9 or '+' / '-', it's a simple literal
		Some((_, '0'..='9' | '+' | '-')) => self::read_literal(s)
			.map(|(num, rest)| (LineArgExpr::Literal(num), rest))
			.map_err(ReadArgError::ReadLiteral),

		// If it starts with a label char, it's a label
		Some((_, c)) if self::is_valid_first_name_char(c) => {
			// Read the label
			let (label, rest) = self::read_name(s).map_err(ReadArgError::ReadLabel)?;

			// If there's a '+' after, read an offset too
			let (offset, rest) = match rest.strip_prefix('+') {
				Some(rest) => self::read_literal(rest)
					.map(|(num, rest)| (Some(num), rest))
					.map_err(ReadArgError::ReadLabelOffset)?,
				None => (None, rest),
			};

			// If there's a '@' after, read a function too
			let (func, rest) = match rest.strip_prefix('@') {
				Some(rest) => self::read_func(rest)
					.map(|(func, rest)| (Some(func), rest))
					.map_err(ReadArgError::ReadLabelFunc)?,
				None => (None, rest),
			};

			let label = LineArgExpr::Label {
				label: label.to_owned(),
				offset,
				func,
			};

			Ok((label, rest))
		},

		// Else it's an invalid char
		Some(_) => Err(ReadArgError::InvalidStartChar),

		None => Err(ReadArgError::Empty),
	}
}

/// Reads a register
fn read_reg(s: &str) -> Result<(Register, &str), ReadArgError> {
	match s.get(..3) {
		Some(reg) => match Register::from_str(reg) {
			Ok(reg) => Ok((reg, &s[3..])),
			Err(()) => Err(ReadArgError::UnknownRegister),
		},
		None => Err(ReadArgError::ExpectedRegister),
	}
}

/// Reads a func
fn read_func(s: &str) -> Result<(LineLabelFunc, &str), ReadFuncError> {
	None.or_else(|| s.strip_prefix("addr_hi").map(|rest| (LineLabelFunc::AddrHi, rest)))
		.or_else(|| s.strip_prefix("addr_lo").map(|rest| (LineLabelFunc::AddrLo, rest)))
		.ok_or(ReadFuncError::Unknown)
}

/// Reads a string
fn read_string(s: &str) -> Result<(String, &str), ReadArgError> {
	let mut is_escaping = false;
	let mut in_multi_escape = false;
	let mut chars = s.char_indices();
	assert_matches!(chars.next(), Some((_, '"')));
	loop {
		match chars.next() {
			// If we get '\', start escaping
			Some((_, '\\')) if !is_escaping => is_escaping = true,

			// If we get '{' while escaping, start multi-escaping
			Some((_, '{')) if is_escaping => in_multi_escape = true,

			// During multi escape, ignore everything except '}'
			Some(_) if in_multi_escape => (),

			// If we get '}' during multi-escape, stop escaping and multi escaping
			Some((_, '}')) if in_multi_escape => {
				in_multi_escape = false;
				is_escaping = false;
			},

			// Else if we get anything during single escape, stop escaping
			Some(_) if is_escaping => is_escaping = false,

			// If we get '"' while not escaping, return
			Some((idx, '"')) if !is_escaping => {
				let (string, rest) = s.split_at(idx + 1);

				// Note: For whatever reason 'snailquote' requires the quotes to be included in `string`
				let string = snailquote::unescape(string).map_err(ReadArgError::UnescapeString)?;

				break Ok((string, rest));
			},

			// Else just continue
			Some(_) => continue,

			None => break Err(ReadArgError::MissingClosingDelimiterString),
		};
	}
}

/// Reads a literal from a string and returns the rest
fn read_literal(s: &str) -> Result<(i64, &str), ReadLiteralError> {
	// Check if it's negative
	let (is_neg, num) = match s.chars().next() {
		Some('+') => (false, &s[1..]),
		Some('-') => (true, &s[1..]),
		_ => (false, s),
	};

	// Check if we have a base
	let (base, num) = match num.as_bytes() {
		[b'0', b'x', ..] => (16, &num[2..]),
		[b'0', b'o', ..] => (8, &num[2..]),
		[b'0', b'b', ..] => (2, &num[2..]),
		_ => (10, num),
	};

	// Returns if 'c' is a valid digit for the current base
	let is_valid_digit = |c| match base {
		16 => ('0'..='9').contains(&c) || ('a'..='f').contains(&c),
		10 => ('0'..='9').contains(&c),
		8 => ('0'..='8').contains(&c),
		2 => ('0'..='1').contains(&c),
		_ => todo!("Unsupported base"),
	};

	// Then check where the number ends
	let (num, rest) = match num.find(|c| !is_valid_digit(c)).map(|idx| num.split_at(idx)) {
		Some((num, rest)) => (num, rest),
		None => (num, ""),
	};

	// Parse it
	let num = i64::from_str_radix(num, base).map_err(ReadLiteralError::Parse)?;
	let num = match is_neg {
		true => -num,
		false => num,
	};

	Ok((num, rest))
}

/// Returns if `c` is a valid mnemonic first character
fn is_valid_first_name_char(c: char) -> bool {
	c.is_ascii_alphabetic() || ['.', '_'].contains(&c)
}

/// Returns if `c` is a valid mnemonic continuation character
fn is_valid_cont_name_char(c: char) -> bool {
	c.is_ascii_alphanumeric() || ['.', '_'].contains(&c)
}
