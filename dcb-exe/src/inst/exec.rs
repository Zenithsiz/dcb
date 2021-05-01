//! Execution

// Modules
pub mod error;

// Exports
pub use error::ExecError;

// Imports
use super::basic;
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

	/// Executes a syscall
	fn sys(&mut self, inst: basic::sys::Inst) -> Result<(), ExecError>;
}

/// An executable instruction
pub trait Executable {
	/// Executes this instruction in `state`
	fn exec<Ctx: ExecCtx>(&self, state: &mut Ctx) -> Result<(), ExecError>;
}
