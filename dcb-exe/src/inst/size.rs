//! Instruction sizes

/// Trait to report the size of an instruction
pub trait InstSize {
	/// Returns the size of this instruction, in bytes.
	fn size(&self) -> usize;
}
