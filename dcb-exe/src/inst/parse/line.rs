//! Line parsing

// # TODO: Refactor this whole module.

// Modules
pub mod error;

// Exports
pub use error::ParseError;

// Imports
use crate::inst::Register;
use std::{
	borrow::Borrow,
	io::{self, Lines},
	str::FromStr,
};

/// Instruction parser
#[derive(Debug)]
pub struct InstParser<R: io::BufRead> {
	/// Bytes
	lines: Lines<R>,
}

impl<R: io::BufRead> InstParser<R> {
	/// Creates a new instruction parser
	pub fn new(reader: R) -> Self {
		let lines = reader.lines();
		Self { lines }
	}

	/// Parses an instruction from a line
	// TODO: Avoid allocations where possible
	// TODO: Remove everything around this, make it just this function
	#[allow(clippy::too_many_lines)] // TODO: Refactor this
	#[allow(clippy::shadow_unrelated)] // We can't do this in only one place, fix naming instead
	pub fn parse_from_line(line: &str) -> Result<Line, ParseError> {
		// Trim the line we read
		let line = line.trim();

		// If it starts with a comment or it's empty, return an empty line
		if line.starts_with('#') || line.is_empty() {
			return Ok(Line { label: None, inst: None });
		}

		// Name character validator
		let is_valid_name_char = |c: char| c.is_alphanumeric() || ['.', '_'].contains(&c);

		// Literal first character validator
		let is_valid_literal_first_char = |c: char| ('0'..='9').contains(&c) || ['+', '-'].contains(&c);

		// Read a name
		let (name, rest) = match line.find(|c| !is_valid_name_char(c)).map(|idx| line.split_at(idx)) {
			// If we got it, remove any whitespace from rest.
			Some((name, rest)) => (name, rest.trim_start()),
			// If the whole line was a name, return a 0-argument instruction
			None => {
				return Ok(Line {
					label: None,
					inst:  Some(LineInst {
						mnemonic: line.to_owned(),
						args:     vec![],
					}),
				});
			},
		};

		// If the rest was a comment, or empty, return a 0-argument instruction
		if rest.starts_with('#') || rest.is_empty() {
			return Ok(Line {
				label: None,
				inst:  Some(LineInst {
					mnemonic: name.to_owned(),
					args:     vec![],
				}),
			});
		}

		// Check if we have a label
		let (label_name, mnemonic, mut args) = match rest.strip_prefix(':').map(str::trim_start) {
			// If it started with `:`, read a name after it and return it.
			Some(rest) => {
				let label = name;

				// If it starts with a comment or it's empty, return only the label
				if rest.starts_with('#') || rest.is_empty() {
					return Ok(Line {
						label: Some(LineLabel { name: label.to_owned() }),
						inst:  None,
					});
				}

				// Ge the mnemonic and arguments
				let (mnemonic, args) = match rest.find(|c| !is_valid_name_char(c)).map(|idx| rest.split_at(idx)) {
					// If we got it, remove any whitespace from rest.
					Some((mnemonic, rest)) => (mnemonic, rest.trim_start()),
					// If everything after the label was a name, return a 0-argument label
					None => {
						return Ok(Line {
							label: Some(LineLabel { name: label.to_owned() }),
							inst:  Some(LineInst {
								mnemonic: rest.to_owned(),
								args:     vec![],
							}),
						});
					},
				};

				(Some(label), mnemonic, args)
			},
			// Else we have no label
			None => (None, name, rest),
		};

		// Else read arguments
		let mut parsed_args = vec![];
		loop {
			// If the remaining arguments were a comment, or empty, break
			if args.starts_with('#') || args.is_empty() {
				break;
			}
			// Else if it starts with a '"', read a string
			else if args.starts_with('"') {
				// Returns `true` for the first non-escaped '"' in a string
				let mut escaping = false;
				let find_first_non_escaped_quotes = move |c| match c {
					// If we found an escaping character, toggle escape
					// Note: If we weren't escaping, it's the start of an escape,
					//       else it's the '\\' escape.
					'\\' => {
						escaping ^= true;
						false
					},
					// If we found a '"' while not escaping, finish
					'"' if !escaping => true,

					// Else set us as not escaping and return
					// Note: This is fine even for multi-character escapes, as
					//       '"' must be escaped with a single character.
					_ => {
						escaping = false;
						false
					},
				};

				// Find the first non-escaped '"'
				// Note: The `+2` can never panic.
				let string = match args[1..].find(find_first_non_escaped_quotes).map(|idx| args.split_at(idx + 2)) {
					Some((string, rest)) => {
						args = rest.trim_start();
						string
					},
					None => return Err(ParseError::UnterminatedString),
				};

				// Create the string and push it
				// Note: For whatever reason 'snailquote' requires the quotes to be included in `string`
				let string = snailquote::unescape(string).map_err(ParseError::StringUnescape)?;
				parsed_args.push(LineArg::String(string));
			}
			// Else if it starts with a number (possibly negative), read a literal
			else if args.starts_with(is_valid_literal_first_char) {
				// Try to parse a number
				let (num, rest) = self::parse_literal(args)?;
				args = rest.trim_start();

				// If we got a '(' after this literal, read a RegisterOffset instead
				if args.starts_with('(') {
					// Trim the '(' and whitespace
					args = args[1..].trim_start();

					// Then make sure it's a register
					if !args.starts_with('$') {
						return Err(ParseError::ExpectedRegister);
					}

					// Read it and update the arguments
					let reg = args.get(0..=2).ok_or(ParseError::ExpectedRegister)?;
					args = args[3..].trim_start();

					// If it doesn't end with a ')', return Err
					match args.strip_prefix(')') {
						Some(rest) => args = rest.trim_start(),
						None => return Err(ParseError::UnterminatedRegisterOffset),
					}

					// Then parse it and push the offset
					let register = Register::from_str(reg).map_err(|()| ParseError::UnknownRegister)?;
					parsed_args.push(LineArg::RegisterOffset { register, offset: num });
				}
				// Else simply add the literal
				else {
					parsed_args.push(LineArg::Literal(num));
				}
			}
			// Else if it starts with '$', it's a register
			else if args.starts_with('$') {
				// Try to get the 3 characters forming the register and update the remaining args
				let reg = args.get(0..=2).ok_or(ParseError::ExpectedRegister)?;
				args = args[3..].trim_start();

				// Then parse it and add it
				let reg = Register::from_str(reg).map_err(|()| ParseError::UnknownRegister)?;
				parsed_args.push(LineArg::Register(reg));
			}
			// Else try to read it as a label with a possible offset
			else {
				// Read a label name
				let (label, offset) = match args.find(|c| !is_valid_name_char(c)).map(|idx| args.split_at(idx)) {
					Some((name, rest)) => {
						// If the next character is '+'
						args = rest.trim_start();
						match args.strip_prefix('+') {
							Some(rest) => {
								// Try to parse a number
								let (num, rest) = self::parse_literal(rest)?;
								args = rest.trim_start();

								// And return the offset
								(name, Some(num))
							},
							None => (name, None),
						}
					},
					// Else the whole rest was just the label
					// Note: This can't be empty
					None => {
						let label = args;
						args = "";
						(label, None)
					},
				};

				// And add it
				let label = match offset {
					Some(offset) => LineArg::LabelOffset {
						label: label.to_owned(),
						offset,
					},
					None => LineArg::Label(label.to_owned()),
				};
				parsed_args.push(label);
			}

			// If we find a ',', consume and try out the next argument,
			// else make sure there are no arguments after this
			match args.strip_prefix(',') {
				Some(rest) => args = rest.trim_start(),
				None => {
					// If there's anything remaining, return Err
					if !args.starts_with('#') && !args.is_empty() {
						return Err(ParseError::ExpectedCommaBetweenArgs);
					}

					// Else break, we're done
					break;
				},
			}
		}


		Ok(Line {
			label: label_name.map(|name| LineLabel { name: name.to_owned() }),
			inst:  Some(LineInst {
				mnemonic: mnemonic.to_owned(),
				args:     parsed_args,
			}),
		})
	}
}

impl<R: io::BufRead> Iterator for InstParser<R> {
	type Item = Result<Line, ParseError>;

	fn next(&mut self) -> Option<Self::Item> {
		// Get the next line
		let line = match self.lines.next()? {
			Ok(line) => line,
			Err(err) => return Some(Err(ParseError::ReadLine(err))),
		};

		// Then parse it
		Self::parse_from_line(&line).map(Some).transpose()
	}
}

/// An instruction line
#[derive(PartialEq, Clone, Debug)]
pub struct Line {
	/// Label
	pub label: Option<LineLabel>,

	/// Instruction
	pub inst: Option<LineInst>,
}

/// A label
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Debug)]
pub struct LineLabel {
	/// Name
	pub name: String,
}

impl Borrow<String> for LineLabel {
	fn borrow(&self) -> &String {
		&self.name
	}
}

/// An instructions
#[derive(PartialEq, Clone, Debug)]
pub struct LineInst {
	/// Mnemonic
	pub mnemonic: String,

	/// Components
	pub args: Vec<LineArg>,
}

/// An argument
#[derive(PartialEq, Clone, Debug)]
pub enum LineArg {
	/// String
	String(String),

	/// Register
	Register(Register),

	/// Register offset
	RegisterOffset {
		/// The register
		register: Register,

		/// The offset
		offset: i64,
	},

	/// Literal
	Literal(i64),

	/// Label
	Label(String),

	/// LabelOffset
	LabelOffset {
		/// The label
		label: String,

		/// The offset
		offset: i64,
	},
}

impl LineArg {
	/// Returns this argument as a string
	#[must_use]
	pub fn as_string(&self) -> Option<&str> {
		match self {
			Self::String(string) => Some(string),
			_ => None,
		}
	}

	/// Returns this argument as a register
	#[must_use]
	pub const fn as_register(&self) -> Option<Register> {
		match *self {
			Self::Register(reg) => Some(reg),
			_ => None,
		}
	}

	/// Returns this argument as a register offset
	#[must_use]
	pub const fn as_register_offset(&self) -> Option<(Register, i64)> {
		match *self {
			Self::RegisterOffset { register, offset } => Some((register, offset)),
			_ => None,
		}
	}

	/// Returns this argument as a literal
	#[must_use]
	pub const fn as_literal(&self) -> Option<i64> {
		match *self {
			Self::Literal(literal) => Some(literal),
			_ => None,
		}
	}

	/// Returns this argument as a label
	#[must_use]
	pub fn as_label(&self) -> Option<&str> {
		match self {
			Self::Label(label) => Some(label),
			_ => None,
		}
	}

	/// Returns this argument as a label offset
	#[must_use]
	pub fn as_label_offset(&self) -> Option<(&str, i64)> {
		match self {
			Self::LabelOffset { label, offset } => Some((label, *offset)),
			_ => None,
		}
	}
}


/// Parses a literal from a string and returns the rest
fn parse_literal(s: &str) -> Result<(i64, &str), ParseError> {
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
	let num = i64::from_str_radix(num, base).map_err(ParseError::ParseLiteral)?;
	let num = match is_neg {
		true => -num,
		false => num,
	};

	Ok((num, rest))
}
