//! Decompiler

#![feature(
	try_blocks,
	format_args_capture,
	iter_map_while,
	box_syntax,
	trivial_bounds,
	slice_index_methods,
	never_type
)]

// Modules
mod args;

// Imports
use crate::args::Args;
use anyhow::Context;
use byteorder::{ByteOrder, LittleEndian};
use dcb_exe::{
	inst::{
		self,
		basic::{self, mult::MultReg, Decode},
		exec::{ExecCtx, ExecError, Executable},
		InstDisplay, InstFmtArg, Register,
	},
	Pos,
};
use itertools::{Itertools, Position};
use std::{
	convert::TryInto,
	fmt, fs,
	io::{self, BufReader, Read, Seek},
	ops::{Index, IndexMut},
	slice::SliceIndex,
};

fn main() -> Result<(), anyhow::Error> {
	// Initialize the logger
	simplelog::TermLogger::init(
		log::LevelFilter::Info,
		simplelog::Config::default(),
		simplelog::TerminalMode::Stderr,
	)
	.expect("Unable to initialize logger");

	// Get all arguments
	let args = Args::new();

	// Setup memory
	let mut memory = box Memory::new();

	// Read the bios and write it into memory
	let bios_contents = fs::read("SCPH1001.BIN").context("Unable to read bios")?;
	anyhow::ensure!(
		bios_contents.len() == 0x80000,
		"Bios had an unexpected size: {:#x}",
		bios_contents.len()
	);
	memory[..0x80000].copy_from_slice(&bios_contents);

	// Open the input file and unbin it
	let game_file = fs::File::open(&args.game_path).context("Unable to open game file")?;
	let game_file = BufReader::new(game_file);
	let mut game_file = dcb_cdrom_xa::CdRomReader::new(game_file);
	let game_fs = dcb_iso9660::FilesystemReader::new(&mut game_file).context("Unable to open game file as iso9660")?;
	let game_root_dir = game_fs
		.root_dir()
		.read_dir(&mut game_file)
		.context("Unable to read game files")?;

	// Note: In other executables, we should read the `SYSTEM.CNF` to
	//       determine the executable file. For `dcb` we simply know which
	//       one it is and all it's data.

	// Read the executable file, skipping the header
	let exec_file = game_root_dir
		.entries()
		.iter()
		.find(|entry| entry.is_file() && entry.name.as_bytes() == b"SLUS_013.28;1")
		.context("Unable to find game executable file")?;
	let mut exec_file = exec_file
		.read_file(&mut game_file)
		.context("Unable to read game executable")?;

	let exec_contents = {
		let mut bytes = Vec::with_capacity(exec_file.size().try_into().expect("`u64` didn't fit into `usize`"));
		exec_file
			.seek(io::SeekFrom::Start(0x800))
			.context("Unable to seek past game executable header")?;
		exec_file
			.read_to_end(&mut bytes)
			.context("Unable to read all of game executable")?;
		bytes
	};

	// Then write it into memory at `0x10000`
	memory[0x10000..(0x10000 + exec_contents.len())].copy_from_slice(&exec_contents);

	// Create the executor
	let mut exec_state = ExecState {
		pc: Pos(0x80056270),
		regs: [0; 32],
		lo_hi_reg: [0; 2],
		memory,
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

/// Memory
// TODO: Have this work with `u32`s externally.
pub struct Memory {
	/// All bytes
	bytes: [u8; 0x200000],
}

impl Memory {
	/// Creates a new memory chunk
	#[allow(clippy::new_without_default)] // We want an explicit constructor
	pub fn new() -> Self {
		Self { bytes: [0; 0x200000] }
	}

	/// Returns the bytes at `index`
	pub fn get<I>(&self, index: I) -> Option<&I::Output>
	where
		I: SliceIndex<[u8]>,
	{
		index.get(&self.bytes)
	}

	/// Returns the bytes at `index` mutably
	pub fn get_mut<I>(&mut self, index: I) -> Option<&mut I::Output>
	where
		I: SliceIndex<[u8]>,
	{
		index.get_mut(&mut self.bytes)
	}
}

impl<I> Index<I> for Memory
where
	[u8]: Index<I>,
{
	type Output = <[u8] as Index<I>>::Output;

	fn index(&self, index: I) -> &Self::Output {
		&self.bytes[index]
	}
}

impl<I> IndexMut<I> for Memory
where
	[u8]: IndexMut<I>,
{
	fn index_mut(&mut self, index: I) -> &mut Self::Output {
		&mut self.bytes[index]
	}
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
	memory: Box<Memory>,

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

		// Display it
		println!("{:010}: {}", self.pc, self::inst_display(&inst, self, self.pc));

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

		// Ignore the top nibble
		let idx = pos.0 & 0x0FFF_FFFF;
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

		// Ignore the top nibble
		let idx = pos.0 & 0x0FFF_FFFF;
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
		// Ignore the top nibble
		let idx = pos.0 & 0x0FFF_FFFF;
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

		// Ignore the top nibble
		let idx = pos.0 & 0x0FFF_FFFF;
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

		// Ignore the top nibble
		let idx = pos.0 & 0x0FFF_FFFF;
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
		// Ignore the top nibble
		let idx = pos.0 & 0x0FFF_FFFF;
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


/// Returns a display-able for an instruction inside a possible function
#[must_use]
pub fn inst_display<'a>(inst: &'a basic::Inst, state: &'a ExecState, pos: Pos) -> impl fmt::Display + 'a {
	// Overload the target of as many as possible using `inst_target`.
	zutil::DisplayWrapper::new(move |f| {
		// Build the context and get the mnemonic + args
		let ctx = DisplayCtx { pos };
		let mnemonic = inst.mnemonic(&ctx);

		write!(f, "{mnemonic}")?;
		for arg in inst.args(&ctx).with_position() {
			// Write ',' if it's first, then a space
			match &arg {
				Position::First(_) | Position::Only(_) => write!(f, " "),
				_ => write!(f, ", "),
			}?;
			let arg = arg.into_inner();

			// Then write the argument
			arg.write(f, &ctx)?
		}

		for arg in inst.args(&ctx) {
			match arg {
				InstFmtArg::Register(reg) => write!(f, "# {reg} = {:#x}", state[reg])?,
				InstFmtArg::RegisterOffset { register, .. } => write!(f, "# {register} = {:#x}", state[register])?,
				_ => (),
			}
		}

		Ok(())
	})
}

/// Displaying context for instructions.
pub struct DisplayCtx {
	/// Current Position
	pos: Pos,
}

impl inst::DisplayCtx for DisplayCtx {
	type Label = !;

	fn cur_pos(&self) -> Pos {
		self.pos
	}

	fn pos_label(&self, _pos: Pos) -> Option<(Self::Label, i64)> {
		None
	}
}
