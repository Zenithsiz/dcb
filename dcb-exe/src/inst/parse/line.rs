//! Line parsing

// Modules
pub mod error;
#[cfg(test)]
pub mod test;

// Exports
pub use error::{ParseArgError, ParseFuncError, ParseLineError, ParseLiteralError, ParseNameError};

// Imports
use crate::inst::Register;
use std::str::FromStr;

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

		// Parse all labels and then the mnemonic
		let mut labels = vec![];
		let mnemonic = loop {
			// If the line starts with a comment or is empty, return all labels
			if line.starts_with('#') || line.is_empty() {
				return Ok(Self { labels, inst: None });
			}

			// Parse a name
			let (name, rest) = self::parse_name(line)?;

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
				// On a space, break and parse arguments
				Some(' ') => {
					line = rest.as_str().trim_start();
					break name.to_owned();
				},
				_ => return Err(ParseLineError::InvalidNameSuffix),
			}
		};

		// Then parse all arguments
		let mut args = vec![];
		loop {
			// Parse an argument
			let (arg, rest) = self::parse_arg(line)?;
			args.push(arg);

			// Check the character after the argument
			let rest = rest.trim_start();
			let mut rest = rest.chars();
			match rest.next() {
				// If we got ',', continue parsing
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

impl LineArgExpr {
	/// Parses an expression
	pub fn parse(s: &str) -> Result<(LineArgExpr, &str), ParseArgError> {
		let mut chars = s.char_indices();
		match chars.next() {
			// If it's numeric, 0..9 or '+' / '-', it's a simple literal
			Some((_, '0'..='9' | '+' | '-')) => self::parse_literal(s)
				.map(|(num, rest)| (LineArgExpr::Literal(num), rest))
				.map_err(ParseArgError::Literal),

			// If it starts with a label char, it's a label
			Some((_, c)) if self::is_valid_first_name_char(c) => {
				// Parse the label
				let (label, rest) = self::parse_name(s).map_err(ParseArgError::Label)?;

				// If there's a '+' after, parse an offset too
				let (offset, rest) = match rest.strip_prefix('+') {
					Some(rest) => self::parse_literal(rest)
						.map(|(num, rest)| (Some(num), rest))
						.map_err(ParseArgError::LabelOffset)?,
					None => (None, rest),
				};

				// If there's a '@' after, parse a function too
				let (func, rest) = match rest.strip_prefix('@') {
					Some(rest) => self::parse_func(rest)
						.map(|(func, rest)| (Some(func), rest))
						.map_err(ParseArgError::LabelFunc)?,
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
			Some(_) => Err(ParseArgError::InvalidStartChar),

			None => Err(ParseArgError::Empty),
		}
	}
}

/// Line label functions
#[allow(clippy::pub_enum_variant_names)] // We'll have other functions eventually
#[derive(PartialEq, Clone, Debug)]
pub enum LineLabelFunc {
	/// Address lower 16-bits
	AddrLo,

	/// Address higher 16-bits
	AddrHi,
}

/// Parses a name
pub fn parse_name(s: &str) -> Result<(&str, &str), ParseNameError> {
	// Make sure the first character is valid
	let mut chars = s.char_indices();
	match chars.next() {
		Some((_, c)) if self::is_valid_first_name_char(c) => (),
		Some(_) => return Err(ParseNameError::StartChar),
		None => return Err(ParseNameError::Empty),
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

/// Parses an argument
pub fn parse_arg(s: &str) -> Result<(LineArg, &str), ParseArgError> {
	let mut chars = s.char_indices();
	match chars.next() {
		// If we got '$', it's a register
		Some((_, '$')) => self::parse_reg(s).map(|(reg, rest)| (LineArg::Register(reg), rest)),

		// If we got '"', it's a string
		Some((_, '"')) => self::parse_string(s).map(|(string, rest)| (LineArg::String(string), rest)),

		// If we got '^', it's a mnemonic
		Some((_, '^')) => self::parse_name(chars.as_str())
			.map(|(name, rest)| (LineArg::Mnemonic(name.to_owned()), rest))
			.map_err(ParseArgError::Label),

		// Else try to parse an expression
		Some(_) => {
			// Parse the expression
			let (expr, rest) = LineArgExpr::parse(s)?;

			// Then check if we have a register
			let rest = rest.trim_start();
			match rest.strip_prefix('(') {
				// If the rest starts with '(', parse it as a register offset
				Some(rest) => match rest.split_once(')') {
					Some((reg, rest)) => {
						// Parse the register
						// If we have leftover tokens after parsing it, return Err
						let reg = reg.trim();
						let (reg, reg_rest) = self::parse_reg(reg)?;
						if !reg_rest.is_empty() {
							return Err(ParseArgError::RegisterOffsetLeftoverTokens);
						}

						Ok((
							LineArg::RegisterOffset {
								register: reg,
								offset:   expr,
							},
							rest,
						))
					},
					None => Err(ParseArgError::MissingRegisterOffsetDelimiter),
				},
				None => Ok((LineArg::Expr(expr), rest)),
			}
		},

		None => Err(ParseArgError::Empty),
	}
}

/// Parse a register
pub fn parse_reg(s: &str) -> Result<(Register, &str), ParseArgError> {
	match s.get(..3) {
		Some(reg) => match Register::from_str(reg) {
			Ok(reg) => Ok((reg, &s[3..])),
			Err(()) => Err(ParseArgError::UnknownRegister),
		},
		None => Err(ParseArgError::ExpectedRegister),
	}
}

/// Parses a func
pub fn parse_func(s: &str) -> Result<(LineLabelFunc, &str), ParseFuncError> {
	None.or_else(|| s.strip_prefix("addr_hi").map(|rest| (LineLabelFunc::AddrHi, rest)))
		.or_else(|| s.strip_prefix("addr_lo").map(|rest| (LineLabelFunc::AddrLo, rest)))
		.ok_or(ParseFuncError::Unknown)
}

/// Parses a string
///
/// # Panics if `s[0]` isn't '"'.
pub fn parse_string(s: &str) -> Result<(String, &str), ParseArgError> {
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
				let string = snailquote::unescape(string).map_err(ParseArgError::UnescapeString)?;

				break Ok((string, rest));
			},

			// Else just continue
			Some(_) => continue,

			None => break Err(ParseArgError::MissingClosingDelimiterString),
		};
	}
}

/// Parses a literal from a string and returns the rest
pub fn parse_literal(s: &str) -> Result<(i64, &str), ParseLiteralError> {
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
	let num = i64::from_str_radix(num, base).map_err(ParseLiteralError::Parse)?;
	let num = match is_neg {
		true => -num,
		false => num,
	};

	Ok((num, rest))
}

/// Returns if `c` is a valid name first character
#[must_use]
fn is_valid_first_name_char(c: char) -> bool {
	c.is_ascii_alphabetic() || ['.', '_'].contains(&c)
}

/// Returns if `c` is a valid name continuation character
#[must_use]
fn is_valid_cont_name_char(c: char) -> bool {
	c.is_ascii_alphanumeric() || ['.', '_'].contains(&c)
}
