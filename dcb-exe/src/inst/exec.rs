//! Execution

// Imports
use crate::{
	inst::{
		basic::{self, mult::MultReg, Decode},
		Register,
	},
	Pos,
};
use byteorder::{ByteOrder, LittleEndian};
use std::{
	convert::TryInto,
	ops::{Index, IndexMut},
};

/// Execution state
pub struct ExecState {
	/// Program counter
	pc: Pos,

	/// Registers
	regs: [u32; 32],

	/// Lo / Hi
	lo_hi_reg: [u32; 2],

	/// Memory
	memory: Box<[u8]>,

	/// Jump target
	jump_target: JumpTargetState,
}

impl ExecState {
	/// Creates a new execution state
	#[must_use]
	pub fn new(memory: Box<[u8]>, pc: Pos) -> Self {
		Self {
			pc,
			regs: [0; 32],
			lo_hi_reg: [0; 2],
			memory,
			jump_target: JumpTargetState::None,
		}
	}

	/// Executes the next instruction
	pub fn exec(&mut self) -> Result<(), ExecError> {
		// Read the next instruction
		let inst = self.read_word(self.pc)?;

		// Parse the instruction
		let inst = basic::Inst::decode(inst).ok_or(ExecError::DecodeInst)?;

		// Then execute it
		inst.exec(self)?;

		// Then update our pc depending on whether we have a jump
		self.pc = match self.jump_target {
			JumpTargetState::None => self.pc + 4u32,
			JumpTargetState::JumpNext(pos) => {
				self.jump_target = JumpTargetState::JumpNow(pos);
				self.pc + 4u32
			},
			JumpTargetState::JumpNow(pos) => {
				self.jump_target = JumpTargetState::None;
				pos
			},
		};

		// And increment out program counter
		self.pc += 4u32;

		Ok(())
	}

	/// Returns the current program counter
	#[must_use]
	pub const fn pc(&self) -> Pos {
		self.pc
	}

	/// Sets a jump to happen
	pub fn set_jump(&mut self, pos: Pos) -> Result<(), ExecError> {
		match self.jump_target {
			JumpTargetState::None => {
				self.jump_target = JumpTargetState::JumpNext(pos);
				Ok(())
			},
			_ => Err(ExecError::JumpWhileJumping),
		}
	}

	/// Reads a word from a memory position
	pub fn read_word(&self, pos: Pos) -> Result<u32, ExecError> {
		// If the position isn't aligned, return Err
		if !pos.is_word_aligned() {
			return Err(ExecError::MemoryUnalignedAccess { pos });
		}

		// Ignore the top 3 bits
		let idx = pos.0 & 0x7FFF_FFFF;
		let idx = idx.try_into().expect("Memory position didn't fit into `usize`");

		// Then read from memory
		let mem = self
			.memory
			.get(idx..(idx + 4))
			.ok_or(ExecError::MemoryOutOfBounds { pos })?;
		Ok(LittleEndian::read_u32(mem))
	}

	/// Reads a half-word from a memory position
	pub fn read_half_word(&self, pos: Pos) -> Result<u16, ExecError> {
		// If the position isn't aligned, return Err
		if !pos.is_half_word_aligned() {
			return Err(ExecError::MemoryUnalignedAccess { pos });
		}

		// Ignore the top 3 bits
		let idx = pos.0 & 0x7FFF_FFFF;
		let idx = idx.try_into().expect("Memory position didn't fit into `usize`");

		// Then read from memory
		let mem = self
			.memory
			.get(idx..(idx + 2))
			.ok_or(ExecError::MemoryOutOfBounds { pos })?;
		Ok(LittleEndian::read_u16(mem))
	}

	/// Reads a byte from a memory position
	pub fn read_byte(&self, pos: Pos) -> Result<u8, ExecError> {
		// Ignore the top 3 bits
		let idx = pos.0 & 0x7FFF_FFFF;
		let idx: usize = idx.try_into().expect("Memory position didn't fit into `usize`");

		// Then read from memory
		self.memory
			.get(idx)
			.copied()
			.ok_or(ExecError::MemoryOutOfBounds { pos })
	}

	/// Stores a word to a memory position
	pub fn write_word(&mut self, pos: Pos, value: u32) -> Result<(), ExecError> {
		// If the position isn't aligned, return Err
		if !pos.is_word_aligned() {
			return Err(ExecError::MemoryUnalignedAccess { pos });
		}

		// Ignore the top 3 bits
		let idx = pos.0 & 0x7FFF_FFFF;
		let idx = idx.try_into().expect("Memory position didn't fit into `usize`");

		// Then write to memory
		let mem = self
			.memory
			.get_mut(idx..(idx + 4))
			.ok_or(ExecError::MemoryOutOfBounds { pos })?;
		LittleEndian::write_u32(mem, value);
		Ok(())
	}

	/// Writes a half-word to a memory position
	pub fn write_half_word(&mut self, pos: Pos, value: u16) -> Result<(), ExecError> {
		// If the position isn't aligned, return Err
		if !pos.is_half_word_aligned() {
			return Err(ExecError::MemoryUnalignedAccess { pos });
		}

		// Ignore the top 3 bits
		let idx = pos.0 & 0x7FFF_FFFF;
		let idx = idx.try_into().expect("Memory position didn't fit into `usize`");

		// Then write to memory
		let mem = self
			.memory
			.get_mut(idx..(idx + 2))
			.ok_or(ExecError::MemoryOutOfBounds { pos })?;
		LittleEndian::write_u16(mem, value);
		Ok(())
	}

	/// Writes a byte to a memory position
	pub fn write_byte(&mut self, pos: Pos, value: u8) -> Result<(), ExecError> {
		// Ignore the top 3 bits
		let idx = pos.0 & 0x7FFF_FFFF;
		let idx: usize = idx.try_into().expect("Memory position didn't fit into `usize`");

		// Then write to memory
		let mem = self.memory.get_mut(idx).ok_or(ExecError::MemoryOutOfBounds { pos })?;
		*mem = value;

		Ok(())
	}
}

impl Index<Register> for ExecState {
	type Output = u32;

	fn index(&self, reg: Register) -> &Self::Output {
		let idx: usize = reg.idx().try_into().expect("Register index didn't fit into `usize`");
		&self.regs[idx]
	}
}

impl IndexMut<Register> for ExecState {
	fn index_mut(&mut self, reg: Register) -> &mut Self::Output {
		let idx: usize = reg.idx().try_into().expect("Register index didn't fit into `usize`");
		&mut self.regs[idx]
	}
}

impl Index<MultReg> for ExecState {
	type Output = u32;

	fn index(&self, reg: MultReg) -> &Self::Output {
		match reg {
			MultReg::Lo => &self.lo_hi_reg[0],
			MultReg::Hi => &self.lo_hi_reg[1],
		}
	}
}

impl IndexMut<MultReg> for ExecState {
	fn index_mut(&mut self, reg: MultReg) -> &mut Self::Output {
		match reg {
			MultReg::Lo => &mut self.lo_hi_reg[0],
			MultReg::Hi => &mut self.lo_hi_reg[1],
		}
	}
}

/// An executable instruction
pub trait Executable {
	/// Executes this instruction in `state`
	fn exec(&self, state: &mut ExecState) -> Result<(), ExecError>;
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
}

/// Jump target state
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum JumpTargetState {
	/// No jump
	None,

	/// Jump next
	JumpNext(Pos),

	/// Jump now
	JumpNow(Pos),
}
