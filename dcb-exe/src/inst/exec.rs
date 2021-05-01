//! Execution

// Imports
use crate::{
	inst::{basic::mult::MultReg, Register},
	Pos,
};
use std::ops::{Index, IndexMut};

/// Executable context
pub trait ExecCtx:
	Index<Register, Output = u32> + IndexMut<Register> + Index<MultReg, Output = u32> + IndexMut<MultReg>
{
	/// Returns the current program counter
	fn pc(&self) -> Pos;

	/// Queues a jump
	fn queue_jump(&mut self, pos: Pos) -> Result<(), ExecError>;

	/// Reads a word
	fn read_word(&self, pos: Pos) -> Result<u32, ExecError>;

	/// Reads a half-word
	fn read_half_word(&self, pos: Pos) -> Result<u16, ExecError>;

	/// Reads a byte
	fn read_byte(&self, pos: Pos) -> Result<u8, ExecError>;

	/// Writes a word
	fn write_word(&mut self, pos: Pos, value: u32) -> Result<(), ExecError>;

	/// Writes a half-word
	fn write_half_word(&mut self, pos: Pos, value: u16) -> Result<(), ExecError>;

	/// Writes a byte
	fn write_byte(&mut self, pos: Pos, value: u8) -> Result<(), ExecError>;
}

/// An executable instruction
pub trait Executable {
	/// Executes this instruction in `state`
	fn exec<Ctx: ExecCtx>(&self, state: &mut Ctx) -> Result<(), ExecError>;
}

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
