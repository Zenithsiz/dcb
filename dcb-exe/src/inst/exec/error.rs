//! Errors

// Imports
use crate::Pos;

/// Executing error
#[derive(Debug, thiserror::Error)]
pub enum ExecError {
	/// Memory address was out of bounds
	#[error("Memory access for {pos} is out of bounds")]
	MemoryOutOfBounds {
		/// Position the instruction tried to access
		pos: Pos,
	},

	/// Memory address was unaligned
	#[error("Memory access for {pos} was unaligned")]
	MemoryUnalignedAccess {
		/// Position which is unaligned
		pos: Pos,
	},

	/// Unable to decode instruction
	#[error("Unable to decode instruction")]
	DecodeInst,

	/// Overflow
	#[error("Overflow")]
	Overflow,

	/// Attempted to jump while jumping
	#[error("Cannot jump while jumping")]
	JumpWhileJumping,

	/// Unknown syscall
	#[error("Unknown syscall")]
	UnknownSys,
}
