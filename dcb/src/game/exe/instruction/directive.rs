//! Directives

// Imports
use super::{FromRawIter, Instruction, Raw};
use crate::game::exe::Pos;
use ascii::{AsciiChar, AsciiStr, AsciiString};
use AsciiChar::Null;

/// A directive
#[derive(PartialEq, Eq, Clone, Debug)]
#[derive(derive_more::Display)]
pub enum Directive {
	/// Write word
	#[display(fmt = "dw {_0:#x}")]
	Dw(u32),

	/// Repeated words
	#[display(fmt = "dw {value:#x}, {len}")]
	DwRepeated {
		/// Value being repeated
		value: u32,

		/// Times the value was repeated
		len: usize,
	},

	/// Ascii string
	#[display(fmt = ".ascii {_0:?}")]
	Ascii(AsciiString),
}

impl Directive {
	/// Decodes a `dw` instruction
	pub fn decode_dw(first_raw: Raw, iter: &mut (impl Iterator<Item = Raw> + Clone)) -> Self {
		let mut times_repeated = 0;

		// Keep getting values until either eof or a different one
		loop {
			let mut cur_iter = iter.clone();
			match cur_iter.next().map(|next_raw| next_raw.repr == first_raw.repr) {
				// If we got a different value, keep fetching values until they're different
				Some(true) => {
					*iter = cur_iter;
					times_repeated += 1;
				},

				// If we didn't get it or we got a different value, exit
				// Note: No need t update the iterator, as it either returned `None` or
				//       a different raw.
				None | Some(false) => match times_repeated {
					// If the value didn't repeat, use a single `dw`
					0 => break Self::Dw(first_raw.repr),

					// Else return the table
					_ => {
						break Self::DwRepeated {
							value: first_raw.repr,
							len:   times_repeated + 1,
						}
					},
				},
			}
		}
	}
}


/// Helper function to check if a string has null and if everything after the first
/// null is also null (or if there were no nulls).
fn check_nulls<S: AsRef<AsciiStr>>(s: S) -> (S, usize, bool) {
	let null_idx = s
		.as_ref()
		.as_slice()
		.iter()
		.position(|&ch| ch == Null)
		.unwrap_or_else(|| s.as_ref().len());
	#[allow(clippy::indexing_slicing)] // `null_idx <= len`
	let uniform_null = s.as_ref()[null_idx..].chars().all(|ch| ch == Null);
	(s, null_idx, uniform_null)
}


impl FromRawIter for Directive {
	type Decoded = Option<(Pos, Self)>;

	fn decode<I: Iterator<Item = Raw> + Clone>(iter: &mut I) -> Self::Decoded {
		// Get the first raw
		let raw = iter.next()?;

		// If we're past all the code, there are no more strings,
		// so just decode a `dw`.
		if raw.pos >= Instruction::CODE_END {
			return Some((raw.pos, Self::decode_dw(raw, iter)));
		}

		// Try to get an ascii string from the raw and check for nulls
		match AsciiString::from_ascii(raw.repr.to_ne_bytes()).map(check_nulls) {
			// If we got a string with at least 1 non-null, but
			// at least 1 null and uniformly null, return just it
			Ok((mut ascii_string, null_idx @ 1..=3, true)) => {
				ascii_string.truncate(null_idx);
				Some((raw.pos, Self::Ascii(ascii_string)))
			},

			// If we got a string without any nulls, keep
			// filling the string until we find one.
			Ok((mut ascii_string, 4, true)) => {
				let ascii_string = loop {
					let mut cur_iter = iter.clone();
					match cur_iter.next() {
						// If we don't have a next character, return the string as-is
						// Note: No need to update the iterator, it returned `None`.
						None => break ascii_string,

						// Else try to get it as a string and check for nulls
						Some(next_raw) => match AsciiStr::from_ascii(&next_raw.repr.to_ne_bytes()).map(check_nulls) {
							// If we got it and it wasn't null, update the iterator, add it and continue
							Ok((new_ascii_str, 4, _)) => {
								*iter = cur_iter;
								ascii_string.push_str(new_ascii_str);
							},

							// If we got it, but there was a uniform null, update the iterator,
							// add the non-null parts and return.
							#[allow(clippy::indexing_slicing)] // `null_idx < len`
							Ok((new_ascii_str, null_idx, true)) => {
								*iter = cur_iter;
								ascii_string.push_str(&new_ascii_str[..null_idx]);
								break ascii_string;
							},

							// If we didn't get it or it was a non-uniform null, return the string we have so far
							// Note: We don't update the iterator, as we want to leave
							//       the next value to `dw`.
							Err(_) | Ok((_, _, false)) => break ascii_string,
						},
					}
				};

				Some((raw.pos, Self::Ascii(ascii_string)))
			},

			// Else if it was full null, non-uniformly null or non-ascii,
			// try to get a dw table
			_ => Some((raw.pos, Self::decode_dw(raw, iter))),
		}
	}
}
