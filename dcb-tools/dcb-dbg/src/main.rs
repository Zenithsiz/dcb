//! Decompiler

#![feature(try_blocks, format_args_capture, iter_map_while, box_syntax)]

// Modules
mod cli;

// Imports
use anyhow::Context;
use byteorder::{ByteOrder, LittleEndian};
use dcb_exe::{
	inst::{
		basic::{self, mult::MultReg, Decode},
		exec::{ExecCtx, ExecError, Executable},
		Register,
	},
	Pos,
};
use std::{
	convert::TryInto,
	fs,
	ops::{Index, IndexMut},
};

fn main() -> Result<(), anyhow::Error> {
	// Initialize the logger
	simplelog::TermLogger::init(
		log::LevelFilter::Info,
		simplelog::Config::default(),
		simplelog::TerminalMode::Stderr,
	)
	.expect("Unable to initialize logger");

	// Get all data from cli
	let cli = cli::CliData::new();

	// Open the input file
	let input_bytes = fs::read(&cli.input_path).context("Unable to read input file")?;

	// Then put them at `0x10000`
	let mut memory = vec![0; 0x10000];
	memory.extend_from_slice(&input_bytes[0x800..]);

	// Create the executor
	let mut exec_state = ExecState {
		pc:          Pos(0x80010000),
		regs:        [0; 32],
		lo_hi_reg:   [0; 2],
		memory:      memory.into(),
		jump_target: JumpTarget::None,
		should_stop: false,
	};

	while !exec_state.should_stop {
		exec_state
			.exec()
			.with_context(|| format!("Failed to execute at {}", exec_state.pc()))?;
	}


	Ok(())
}

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
	jump_target: JumpTarget,

	/// If the processor should stop
	should_stop: bool,
}

impl ExecState {
	/// Executes the next instruction
	fn exec(&mut self) -> Result<(), ExecError> {
		// Read the next instruction
		let inst = self.read_word(self.pc)?;

		// Parse the instruction
		let inst = basic::Inst::decode(inst).ok_or(ExecError::DecodeInst)?;

		// Then execute the instruction
		inst.exec(self)?;

		// Then update our pc depending on whether we have a jump
		self.pc = match self.jump_target {
			JumpTarget::None => self.pc + 4u32,
			JumpTarget::JumpNext(pos) => {
				self.jump_target = JumpTarget::JumpNow(pos);
				self.pc + 4u32
			},
			JumpTarget::JumpNow(pos) => {
				self.jump_target = JumpTarget::None;
				pos
			},
		};

		Ok(())
	}
}

impl ExecCtx for ExecState {
	fn pc(&self) -> Pos {
		self.pc
	}

	fn queue_jump(&mut self, pos: Pos) -> Result<(), ExecError> {
		match self.jump_target {
			JumpTarget::None => {
				self.jump_target = JumpTarget::JumpNext(pos);
				Ok(())
			},
			_ => Err(ExecError::JumpWhileJumping),
		}
	}

	/// Reads a word from a memory position
	fn read_word(&self, pos: Pos) -> Result<u32, ExecError> {
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

	fn read_half_word(&self, pos: Pos) -> Result<u16, ExecError> {
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
	fn read_byte(&self, pos: Pos) -> Result<u8, ExecError> {
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
	fn write_word(&mut self, pos: Pos, value: u32) -> Result<(), ExecError> {
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
	fn write_half_word(&mut self, pos: Pos, value: u16) -> Result<(), ExecError> {
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
	fn write_byte(&mut self, pos: Pos, value: u8) -> Result<(), ExecError> {
		// Ignore the top 3 bits
		let idx = pos.0 & 0x7FFF_FFFF;
		let idx: usize = idx.try_into().expect("Memory position didn't fit into `usize`");

		// Then write to memory
		let mem = self.memory.get_mut(idx).ok_or(ExecError::MemoryOutOfBounds { pos })?;
		*mem = value;

		Ok(())
	}

	fn sys(&mut self, inst: basic::sys::Inst) -> Result<(), ExecError> {
		match inst.comment {
			0x0 => {
				self.should_stop = true;
			},
			0x1 => {
				// Print whatever string is in `$v0`
				let ptr = Pos(self[Register::V0]);

				for n in 0u32.. {
					match self.read_byte(ptr + n)? {
						0 => break,
						b => print!("{}", char::from(b)),
					}
				}
			},
			0x2 => {
				// Print all registers
				for &reg in &Register::ALL_REGISTERS {
					println!("{}: {:#x}", reg, self[reg]);
				}
			},
			comment => return Err(ExecError::UnknownSys { comment }),
		}

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

/// Jump target state
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum JumpTarget {
	/// No jump
	None,

	/// Jump next
	JumpNext(Pos),

	/// Jump now
	JumpNow(Pos),
}
