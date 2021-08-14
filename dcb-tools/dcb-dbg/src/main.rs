//! Game debugger
//!
//! Interactive debugger for the game.

#![feature(
	try_blocks,
	format_args_capture,
	iter_map_while,
	box_syntax,
	trivial_bounds,
	slice_index_methods,
	never_type,
	label_break_value,
	stdio_locked,
	seek_stream_len
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
		InstDisplay, Register,
	},
	Pos,
};
use itertools::{Itertools, Position};
use std::{
	cell::RefCell,
	convert::TryInto,
	fmt, fs,
	io::{self, BufReader, Read, Seek, Write},
	mem,
	ops::{Index, IndexMut},
	path::Path,
	slice::SliceIndex,
};
use zutil::TryIntoAs;

fn main() -> Result<(), anyhow::Error> {
	// Initialize the logger
	simplelog::TermLogger::init(
		log::LevelFilter::Trace,
		simplelog::Config::default(),
		simplelog::TerminalMode::Stderr,
	)
	.expect("Unable to initialize logger");

	// Get all arguments
	let args = Args::new();

	// Setup memory
	let mut memory = box Memory::new();

	// Load the bios into memory
	self::load_bios(&args.bios_path, &mut memory)
		.with_context(|| format!("Unable to load bios from {}", args.bios_path.display()))?;

	// Then open the game and retrieve the root directory.
	let (mut game_file, game_root_dir) = self::load_game(&args.game_path)
		.with_context(|| format!("Unable to load game from {}", args.game_path.display()))?;

	// Note: In other executables, we should read the `SYSTEM.CNF` to
	//       determine the executable file. For `dcb` we simply know which
	//       one it is and all it's data.

	// Load the game executable into memory
	self::load_game_exec(&mut game_file, game_root_dir, &mut memory)?;

	// Create the executor
	let mut exec_state = ExecState {
		pc: Pos(0x80056270),
		regs: [0; 32],
		lo_hi_reg: [0; 2],
		memory,
		jump_target: JumpTarget::None,
		should_stop: false,
		results: RefCell::new(vec![]),
	};

	// Run the repl
	self::run_repl(&mut exec_state)?;

	Ok(())
}

/// Runs the `repl` loop
fn run_repl(exec_state: &mut ExecState) -> Result<(), anyhow::Error> {
	// Get stdin and
	let stdin = io::stdin();
	let mut stdout = io::stdout();

	let mut input_str = String::new();
	let mut input_state = InputState::BreakEvery;
	while !exec_state.should_stop {
		let run_once = |exec_state: &mut ExecState, input_state| {
			exec_state
				.exec(input_state)
				.with_context(|| format!("Failed to execute at {}", exec_state.pc()))
		};

		match input_state {
			InputState::BreakEvery => run_once(exec_state, &input_state)?,
			InputState::RunUntil { pos } => {
				while !exec_state.should_stop && Some(exec_state.pc) != pos {
					run_once(exec_state, &input_state)?;
				}
			},
		};

		let args = match self::get_user_input(&stdin, &mut stdout, &mut input_str)? {
			Some(args) => args,
			None => break,
		};
		match input_state.update(args) {
			Ok(()) => (),
			Err(err) => println!("Unable to update input state: {err:?}"),
		}
	}

	Ok(())
}

/// Retrieve user input
// TODO: Cache memory for the string read and the arguments
fn get_user_input<'a>(
	stdin: &io::Stdin, stdout: &mut io::Stdout, input_str: &'a mut String,
) -> Result<Option<impl Iterator<Item = &'a str>>, anyhow::Error> {
	// Print the input prompt and wait for input
	print!(">>> ");
	stdout.flush().context("Unable to flush stdout")?;
	input_str.clear();
	stdin.read_line(input_str).context("Unable to read input")?;

	if input_str.is_empty() {
		println!();
		return Ok(None);
	}

	Ok(Some(input_str.split_whitespace()))
}

/// Loads the game executable into memory
fn load_game_exec(
	game_file: &mut dcb_cdrom_xa::CdRomReader<impl Read + Seek>, game_root_dir: dcb_iso9660::Dir, memory: &mut Memory,
) -> Result<(), anyhow::Error> {
	/// Executable position
	const EXEC_POS: usize = 0x10000;

	// Search for the executable file
	let exec_file = game_root_dir
		.entries()
		.iter()
		.find(|entry| entry.is_file() && entry.name.as_bytes() == b"SLUS_013.28;1")
		.context("Unable to find game executable file")?;

	// Then open it and skip past the header
	// TODO: Do something with the header, like validate?
	let mut exec_file = exec_file
		.read_file(game_file)
		.context("Unable to read game executable")?;
	exec_file
		.seek(io::SeekFrom::Start(0x800))
		.context("Unable to seek past game executable header")?;

	// Finally read until eof into memory
	zutil::read_slice_until_eof(&mut exec_file, &mut memory[EXEC_POS..])
		.context("Unable to read game executable into memory")?;

	Ok(())
}

/// Loads the game file and it's root directory
fn load_game(
	path: impl AsRef<Path>,
) -> Result<(dcb_cdrom_xa::CdRomReader<BufReader<fs::File>>, dcb_iso9660::Dir), anyhow::Error> {
	// Open the file, wrap it in a buf reader and then in a cdrom reader.
	let game_file = fs::File::open(path).context("Unable to open file")?;
	let game_file = BufReader::new(game_file);
	let mut game_file = dcb_cdrom_xa::CdRomReader::new(game_file);

	// Then read the iso9660 filesystem and the root directory
	let game_fs = dcb_iso9660::FilesystemReader::new(&mut game_file).context("Unable to read iso9660 filesystem")?;
	let game_root_dir = game_fs
		.root_dir()
		.read_dir(&mut game_file)
		.context("Unable to read root directory")?;

	Ok((game_file, game_root_dir))
}

/// Loads the bis from it's path
fn load_bios(path: impl AsRef<Path>, memory: &mut Memory) -> Result<(), anyhow::Error> {
	/// Bios size
	const BIOS_SIZE: u64 = 0x80000;

	// Open the file and check that it's the correct length
	let mut file = fs::File::open(&path).context("Unable to open file")?;
	let file_len = file.stream_len().context("Unable to get file length")?;
	anyhow::ensure!(
		file_len == BIOS_SIZE,
		"Unexpected size {:#x}. Expected {BIOS_SIZE:#x}",
		file_len
	);

	// Then read it into memory
	file.read_exact(&mut memory[..(BIOS_SIZE as usize)])
		.context("Unable to read from file")?;

	Ok(())
}

/// Input state
enum InputState {
	/// Break every instruction
	BreakEvery,

	/// Run until
	RunUntil { pos: Option<Pos> },
}

impl InputState {
	/// Updates the input state from arguments
	pub fn update<S: AsRef<str>>(&mut self, mut args: impl Iterator<Item = S>) -> Result<(), anyhow::Error> {
		match args.next().as_ref().map(S::as_ref) {
			Some("n") => *self = InputState::BreakEvery,
			Some("r") => {
				let pos = args
					.next()
					.map(|pos| {
						self::parse_number(pos.as_ref())
							.context("Unable to parse position")?
							.try_into_as::<u32>()
							.map(Pos)
							.context("Position didn't fit into a `u32`")
					})
					.transpose()?;
				*self = InputState::RunUntil { pos };
			},
			Some(arg) => anyhow::bail!("Unknown argument {:?}", arg),
			None => (),
		}

		Ok(())
	}
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

	/// Results
	results: RefCell<Vec<ExecResult>>,
}

impl ExecState {
	/// Executes the next instruction
	fn exec(&mut self, input_state: &InputState) -> Result<(), ExecError> {
		// Read the next instruction
		let inst = self.read_word(self.pc)?;

		// Parse the instruction
		let inst = basic::Inst::decode(inst).ok_or(ExecError::DecodeInst)?;

		// Display it
		// TODO: Check what registers changed in the op and print them, maybe also memory locations
		//       with a countdown until the change is actually realized or something.
		let print_inst = || println!("{:010}: {}", self.pc, self::inst_display(&inst, self.pc));
		match *input_state {
			InputState::BreakEvery => print_inst(),
			InputState::RunUntil { pos }
				if Some(self.pc + 4) == pos && matches!(self.jump_target, JumpTarget::None) =>
			{
				print_inst()
			},
			_ => (),
		}

		// Then execute the instruction
		inst.exec(self)?;

		// And display the results
		for result in self.results.borrow_mut().drain(..) {
			match result {
				//ExecResult::ReadRegister { reg, value } if reg != Register::Zr => println!("[{reg}] {value:#x}"),
				//ExecResult::ReadMultRegister { reg, value } => println!("[{reg}] {value:#x}"),
				ExecResult::WroteRegister { reg, prev, value } if reg != Register::Zr && prev != value => {
					println!("[{reg}] {prev:#x} => {value:#x}")
				},
				ExecResult::WroteMultRegister { reg, prev, value } if prev != value => {
					println!("[{reg}] {prev:#x} => {value:#x}")
				},
				//ExecResult::ReadWord { pos, value } => println!("[{pos:010}] {value:#x}"),
				//ExecResult::ReadHalfWord { pos, value } => println!("[{pos:010}] {value:#x}"),
				//ExecResult::ReadByte { pos, value } => println!("[{pos:010}] {value:#x}"),
				ExecResult::WriteWord { pos, prev, value } if prev != value => {
					println!("[{pos:010}] {prev:#x} => {value:#x}")
				},
				ExecResult::WriteHalfWord { pos, prev, value } if prev != value => {
					println!("[{pos:010}] {prev:#x} => {value:#x}")
				},
				ExecResult::WriteByte { pos, prev, value } if prev != value => {
					println!("[{pos:010}] {prev:#x} => {value:#x}")
				},
				ExecResult::QueuedJump { pos } => println!("=> [{pos:010}]"),
				_ => (),
			}
		}

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

/// Execution result
pub enum ExecResult {
	/// Read from register `dst`
	ReadRegister { reg: Register, value: u32 },

	/// Read from mult register `dst`
	ReadMultRegister { reg: MultReg, value: u32 },

	/// Wrote to register `dst`
	WroteRegister { reg: Register, prev: u32, value: u32 },

	/// Wrote to mult register `dst`
	WroteMultRegister { reg: MultReg, prev: u32, value: u32 },

	/// Read a word from `pos`
	ReadWord { pos: Pos, value: u32 },

	/// Read a half-word from `pos`
	ReadHalfWord { pos: Pos, value: u16 },

	/// Read a byte from `pos`
	ReadByte { pos: Pos, value: u8 },

	/// Wrote a word to `pos`
	WriteWord { pos: Pos, prev: u32, value: u32 },

	/// Wrote a half-word to `pos`
	WriteHalfWord { pos: Pos, prev: u16, value: u16 },

	/// Wrote a byte to `pos`
	WriteByte { pos: Pos, prev: u8, value: u8 },

	/// Queued a jump to `pos`
	QueuedJump { pos: Pos },
}

impl ExecCtx for ExecState {
	fn pc(&self) -> Pos {
		self.pc
	}

	fn load_reg(&self, reg: Register) -> u32 {
		let idx: usize = reg.idx().try_into().expect("Register index didn't fit into `usize`");
		let value = self.regs[idx];
		self.results.borrow_mut().push(ExecResult::ReadRegister { reg, value });
		value
	}

	fn store_reg(&mut self, reg: Register, value: u32) {
		let idx: usize = reg.idx().try_into().expect("Register index didn't fit into `usize`");
		let prev = mem::replace(&mut self.regs[idx], value);
		self.results
			.get_mut()
			.push(ExecResult::WroteRegister { reg, prev, value });
	}

	fn load_mult_reg(&self, reg: MultReg) -> u32 {
		let value = match reg {
			MultReg::Lo => self.lo_hi_reg[0],
			MultReg::Hi => self.lo_hi_reg[1],
		};
		self.results
			.borrow_mut()
			.push(ExecResult::ReadMultRegister { reg, value });
		value
	}

	fn store_mult_reg(&mut self, reg: MultReg, value: u32) {
		let prev = match reg {
			MultReg::Lo => mem::replace(&mut self.lo_hi_reg[0], value),
			MultReg::Hi => mem::replace(&mut self.lo_hi_reg[1], value),
		};
		self.results
			.get_mut()
			.push(ExecResult::WroteMultRegister { reg, prev, value });
	}

	fn queue_jump(&mut self, pos: Pos) -> Result<(), ExecError> {
		match self.jump_target {
			JumpTarget::None => {
				self.jump_target = JumpTarget::JumpNext(pos);
				self.results.get_mut().push(ExecResult::QueuedJump { pos });
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
		let value = self
			.memory
			.get(idx..(idx + 4))
			.map(LittleEndian::read_u32)
			.ok_or(ExecError::MemoryOutOfBounds { pos })?;
		self.results.borrow_mut().push(ExecResult::ReadWord { pos, value });
		Ok(value)
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
		let value = self
			.memory
			.get(idx..(idx + 2))
			.map(LittleEndian::read_u16)
			.ok_or(ExecError::MemoryOutOfBounds { pos })?;
		self.results.borrow_mut().push(ExecResult::ReadHalfWord { pos, value });
		Ok(value)
	}

	/// Reads a byte from a memory position
	fn read_byte(&self, pos: Pos) -> Result<u8, ExecError> {
		// Ignore the top nibble
		let idx = pos.0 & 0x0FFF_FFFF;
		let idx: usize = idx.try_into().expect("Memory position didn't fit into `usize`");

		// Then read from memory
		let value = self
			.memory
			.get(idx)
			.copied()
			.ok_or(ExecError::MemoryOutOfBounds { pos })?;
		self.results.borrow_mut().push(ExecResult::ReadByte { pos, value });
		Ok(value)
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
		let prev = LittleEndian::read_u32(mem);
		LittleEndian::write_u32(mem, value);
		self.results.get_mut().push(ExecResult::WriteWord { pos, prev, value });
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
		let prev = LittleEndian::read_u16(mem);
		LittleEndian::write_u16(mem, value);
		self.results
			.get_mut()
			.push(ExecResult::WriteHalfWord { pos, prev, value });
		Ok(())
	}

	/// Writes a byte to a memory position
	fn write_byte(&mut self, pos: Pos, value: u8) -> Result<(), ExecError> {
		// Ignore the top nibble
		let idx = pos.0 & 0x0FFF_FFFF;
		let idx: usize = idx.try_into().expect("Memory position didn't fit into `usize`");

		// Then write to memory
		let mem = self.memory.get_mut(idx).ok_or(ExecError::MemoryOutOfBounds { pos })?;
		let prev = *mem;
		*mem = value;
		self.results.get_mut().push(ExecResult::WriteByte { pos, prev, value });

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
pub fn inst_display(inst: &basic::Inst, pos: Pos) -> impl fmt::Display + '_ {
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


/// Parses a number with a possible base
pub fn parse_number(s: &str) -> Result<i64, anyhow::Error> {
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

	// Parse it
	let num = i64::from_str_radix(num, base).context("Unable to parse number")?;
	let num = match is_neg {
		true => -num,
		false => num,
	};

	Ok(num)
}
