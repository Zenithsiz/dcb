//! Raw instructions

// Imports
use crate::exe::Pos;

/// A raw instruction
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Raw {
	/// The raw encoding of the instruction
	pub repr: u32,

	/// The position of this instruction
	pub pos: Pos,
}

/// Raw instruction decoding
///
/// Implementors should be atomic about the consumed and
/// returned iterator, that is, consume the least possible
/// input in order to produce an atomic part of themselves.
pub trait FromRawIter: Sized {
	/// Returned iterator from [`decode`].
	type Decoded: IntoIterator<Item = (Pos, Self)>;

	/// Attempts to decode an instruction from an iterator of raw instructions
	fn decode<I: Iterator<Item = Raw> + Clone>(iter: &mut I) -> Self::Decoded;
}
