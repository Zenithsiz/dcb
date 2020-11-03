//! Basic instructions
//!
//! This modules defines all the basic instructions from the psx.
//! They are all 1 word (`u32`) long.

// Modules
pub mod alu_imm;
pub mod cond;
pub mod jmp;
pub mod load;
pub mod special;
pub mod store;

// Exports
pub use alu_imm::{AluImmInst, AluImmRaw};
pub use cond::{CondInst, CondRaw};
pub use jmp::{JmpInst, JmpRaw};
pub use load::{LoadInst, LoadRaw};
pub use special::{SpecialInst, SpecialRaw};
pub use store::{StoreInst, StoreRaw};

// Imports
use super::{FromRawIter, Raw, Register};
use crate::{game::exe::Pos, util::SignedHex};
use bitmatch::bitmatch;
use int_conv::{SignExtended, Signed, Truncate, Truncated, ZeroExtended};

/// All basic instructions
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[derive(derive_more::Display)]
pub enum BasicInst {
	/// Store
	Store(StoreInst),

	/// Load
	Load(LoadInst),

	/// Special
	Special(SpecialInst),

	/// Condition
	Cond(CondInst),

	/// Jump
	Jmp(JmpInst),

	/// Alu immediate
	AluImm(AluImmInst),

	/// Load upper immediate
	#[display(fmt = "lui {dest}, {value:#x}")]
	Lui {
		/// Destination
		dest: Register,

		/// Value
		value: u16,
	},
}

impl BasicInst {
	// TODO: MAybe extract the strings if the bitmatch macro allows for it.

	/// Decodes this instruction
	#[must_use]
	#[bitmatch::bitmatch]
	#[allow(clippy::many_single_char_names)] // `bitmatch` can only output single character names.
	pub fn decode(raw: u32) -> Option<Self> {
		Some(
			#[bitmatch]
			#[allow(unused_variables)] // TODO: Remove once `CopInst` is implemented.
			match raw {
				"000000_sssss_ttttt_ddddd_iiiii_ffffff" => Self::Special(SpecialInst::decode(SpecialRaw { s, t, d, i, f })?),
				"00001p_iiiii_iiiii_iiiii_iiiii_iiiiii" => Self::Jmp(JmpInst::decode(JmpRaw { p, i })),
				"000ppp_sssss_ttttt_iiiii_iiiii_iiiiii" => Self::Cond(CondInst::decode(CondRaw { p, s, t, i })?),
				"001111_?????_ttttt_iiiii_iiiii_iiiiii" => Self::Lui {
					dest:  Register::new(t)?,
					value: i.truncated::<u16>(),
				},
				"001ppp_sssss_ttttt_iiiii_iiiii_iiiiii" => Self::AluImm(AluImmInst::decode(AluImmRaw { p, s, t, i })?),
				"100ppp_sssss_ttttt_iiiii_iiiii_iiiiii" => Self::Store(StoreInst::decode(StoreRaw { p, s, t, i })?),
				"101ppp_sssss_ttttt_iiiii_iiiii_iiiiii" => Self::Load(LoadInst::decode(LoadRaw { p, s, t, i })?),

				/*
				"0100nn_1iiii_iiiii_iiiii_iiiii_iiiiii" => Self::Cop(),
				"0100nn_00000_ttttt_ddddd_?????_000000" => Self::Cop(),
				"0100nn_00010_ttttt_ddddd_?????_000000" => Self::Cop(),
				"0100nn_00100_ttttt_ddddd_?????_000000" => Self::Cop(),
				"0100nn_00110_ttttt_ddddd_?????_000000" => Self::Cop(),
				"0100nn_01000_00000_iiiii_iiiii_iiiiii" => Self::Cop(),
				"0100nn_01000_00001_iiiii_iiiii_iiiiii" => Self::Cop(),
				"1100nn_sssss_ttttt_iiiii_iiiii_iiiiii" => Self::Cop(),
				"1110nn_sssss_ttttt_iiiii_iiiii_iiiiii" => Self::Cop(),
				*/
				_ => return None,
			},
		)
	}

	/// Encodes this instruction
	#[must_use]
	#[bitmatch]
	pub fn encode(self) -> u32 {
		#[rustfmt::skip]
		match self {
			Self::Special(inst) => { let SpecialRaw {    s, t, d, i, f } = inst.encode(); bitpack!("000000_sssss_ttttt_ddddd_iiiii_ffffff") },
			Self::Jmp    (inst) => { let     JmpRaw { p,          i    } = inst.encode(); bitpack!("00001p_iiiii_iiiii_iiiii_iiiii_iiiiii") },
			Self::Cond   (inst) => { let    CondRaw { p, s, t,    i    } = inst.encode(); bitpack!("000ppp_sssss_ttttt_iiiii_iiiii_iiiiii") },
			Self::AluImm (inst) => { let  AluImmRaw { p, s, t,    i    } = inst.encode(); bitpack!("001ppp_sssss_ttttt_iiiii_iiiii_iiiiii") },
			Self::Store  (inst) => { let   StoreRaw { p, s, t,    i    } = inst.encode(); bitpack!("100ppp_sssss_ttttt_iiiii_iiiii_iiiiii") },
			Self::Load   (inst) => { let    LoadRaw { p, s, t,    i    } = inst.encode(); bitpack!("101ppp_sssss_ttttt_iiiii_iiiii_iiiiii") },

			Self::Lui { dest, value } => {
				let t = dest.idx();
				let i = value.zero_extended::<u32>();
				bitpack!("001111_?????_ttttt_iiiii_iiiii_iiiiii")
			},
		}
	}
}


/// All simple instructions
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[derive(derive_more::Display)]
#[allow(clippy::missing_docs_in_private_items)] // They're mostly register and immediate names.
pub enum BasicInstruction {
	/// Store byte
	#[display(fmt = "sb {rt}, {:#x}({rs})", "SignedHex(offset)")]
	Sb { rt: Register, rs: Register, offset: i16 },

	/// Store half-word
	#[display(fmt = "sh {rt}, {:#x}({rs})", "SignedHex(offset)")]
	Sh { rt: Register, rs: Register, offset: i16 },

	/// Store left word
	#[display(fmt = "swl {rt}, {:#x}({rs})", "SignedHex(offset)")]
	Swl { rt: Register, rs: Register, offset: i16 },

	/// Store word
	#[display(fmt = "sw {rt}, {:#x}({rs})", "SignedHex(offset)")]
	Sw { rt: Register, rs: Register, offset: i16 },

	/// Store right word
	#[display(fmt = "swr {rt}, {:#x}({rs})", "SignedHex(offset)")]
	Swr { rt: Register, rs: Register, offset: i16 },

	/// Load byte
	#[display(fmt = "lb {rt}, {:#x}({rs})", "SignedHex(offset)")]
	Lb { rt: Register, rs: Register, offset: i16 },

	/// Load byte unsigned
	#[display(fmt = "lbu {rt}, {:#x}({rs})", "SignedHex(offset)")]
	Lbu { rt: Register, rs: Register, offset: i16 },

	/// Load half-word
	#[display(fmt = "lh {rt}, {:#x}({rs})", "SignedHex(offset)")]
	Lh { rt: Register, rs: Register, offset: i16 },

	/// Load half-word unsigned
	#[display(fmt = "lhu {rt}, {:#x}({rs})", "SignedHex(offset)")]
	Lhu { rt: Register, rs: Register, offset: i16 },

	/// Load left word
	#[display(fmt = "lwl {rt}, {:#x}({rs})", "SignedHex(offset)")]
	Lwl { rt: Register, rs: Register, offset: i16 },

	/// Load word
	#[display(fmt = "lw {rt}, {:#x}({rs})", "SignedHex(offset)")]
	Lw { rt: Register, rs: Register, offset: i16 },

	/// Load right word
	#[display(fmt = "lwr {rt}, {:#x}({rs})", "SignedHex(offset)")]
	Lwr { rt: Register, rs: Register, offset: i16 },

	/// Add
	#[display(fmt = "add {rd}, {rs}, {rt}")]
	Add { rd: Register, rs: Register, rt: Register },

	/// Add unsigned
	#[display(fmt = "addu {rd}, {rs}, {rt}")]
	Addu { rd: Register, rs: Register, rt: Register },

	/// Sub
	#[display(fmt = "sub {rd}, {rs}, {rt}")]
	Sub { rd: Register, rs: Register, rt: Register },

	/// Sub unsigned
	#[display(fmt = "subu {rd}, {rs}, {rt}")]
	Subu { rd: Register, rs: Register, rt: Register },

	/// Add immediate
	#[display(fmt = "addi {rt}, {rs}, {:#x}", "SignedHex(imm)")]
	Addi { rt: Register, rs: Register, imm: i16 },

	/// Add immediate sign-extended
	/// Note: _NOT_ Unsigned.
	#[display(fmt = "addiu {rt}, {rs}, {:#x}", "SignedHex(imm)")]
	Addiu { rt: Register, rs: Register, imm: i16 },

	/// Set less than
	#[display(fmt = "slt {rd}, {rs}, {rt}")]
	Slt { rd: Register, rs: Register, rt: Register },

	/// Set less than unsigned
	#[display(fmt = "sltu {rd}, {rs}, {rt}")]
	Sltu { rd: Register, rs: Register, rt: Register },

	/// Set less than immediate
	#[display(fmt = "slti {rt}, {rs}, {:#x}", "SignedHex(imm)")]
	Slti { rt: Register, rs: Register, imm: i16 },

	/// Set less than immediate unsigned
	#[display(fmt = "sltiu {rt}, {rs}, {:#x}", "SignedHex(imm)")]
	Sltiu { rt: Register, rs: Register, imm: i16 },

	/// And
	#[display(fmt = "and {rd}, {rs}, {rt}")]
	And { rd: Register, rs: Register, rt: Register },

	/// Or
	#[display(fmt = "or {rd}, {rs}, {rt}")]
	Or { rd: Register, rs: Register, rt: Register },

	/// Xor
	#[display(fmt = "xor {rd}, {rs}, {rt}")]
	Xor { rd: Register, rs: Register, rt: Register },

	/// Nor
	#[display(fmt = "nor {rd}, {rs}, {rt}")]
	Nor { rd: Register, rs: Register, rt: Register },

	/// And immediate
	#[display(fmt = "andi {rt}, {rs}, {imm:#x}")]
	Andi { rt: Register, rs: Register, imm: u16 },

	/// Or immediate
	#[display(fmt = "ori {rt}, {rs}, {imm:#x}")]
	Ori { rt: Register, rs: Register, imm: u16 },

	/// Xor immediate
	#[display(fmt = "xori {rt}, {rs}, {imm:#x}")]
	Xori { rt: Register, rs: Register, imm: u16 },

	/// Shift left logical variable
	#[display(fmt = "sllv {rd}, {rt}, {rs}")]
	Sllv { rd: Register, rt: Register, rs: Register },

	/// Shift right logical variable
	#[display(fmt = "srlv {rd}, {rt}, {rs}")]
	Srlv { rd: Register, rt: Register, rs: Register },

	/// Shift right arithmetic variable
	#[display(fmt = "srav {rd}, {rt}, {rs}")]
	Srav { rd: Register, rt: Register, rs: Register },

	/// Shift left logical
	#[display(fmt = "sll {rd}, {rt}, {imm:#x}")]
	Sll { rd: Register, rt: Register, imm: u8 },

	/// Shift right logical
	#[display(fmt = "srl {rd}, {rt}, {imm:#x}")]
	Srl { rd: Register, rt: Register, imm: u8 },

	/// Shift right arithmetic
	#[display(fmt = "sra {rd}, {rt}, {imm:#x}")]
	Sra { rd: Register, rt: Register, imm: u8 },

	/// Load upper immediate
	#[display(fmt = "lui {rt}, {imm:#x}")]
	Lui { rt: Register, imm: u16 },

	/// Multiply
	#[display(fmt = "mult {rs}, {rt}")]
	Mult { rs: Register, rt: Register },

	/// Multiply unsigned
	#[display(fmt = "multu {rs}, {rt}")]
	Multu { rs: Register, rt: Register },

	/// Divide
	#[display(fmt = "div {rs}, {rt}")]
	Div { rs: Register, rt: Register },

	/// Multiply unsigned
	#[display(fmt = "divu {rs}, {rt}")]
	Divu { rs: Register, rt: Register },

	/// Move from hi
	#[display(fmt = "mfhi {rd}")]
	Mfhi { rd: Register },

	/// Move from lo
	#[display(fmt = "mflo {rd}")]
	Mflo { rd: Register },

	/// Move to hi
	#[display(fmt = "mthi {rs}")]
	Mthi { rs: Register },

	/// Move to lo
	#[display(fmt = "mtlo {rs}")]
	Mtlo { rs: Register },

	/// Jump
	#[display(fmt = "j {target:#x}")]
	J { target: Pos },

	/// Jump and link
	#[display(fmt = "jal {target:#x}")]
	Jal { target: Pos },

	/// Jump register
	#[display(fmt = "jr {rs}")]
	Jr { rs: Register },

	/// Jump and link register
	#[display(fmt = "jalr {rd}, {rs}")]
	Jalr { rd: Register, rs: Register },

	/// Branch if equal
	#[display(fmt = "beq {rs}, {rt}, {target:#x}")]
	Beq { rs: Register, rt: Register, target: Pos },

	/// Branch if not equal
	#[display(fmt = "bne {rs}, {rt}, {target:#x}")]
	Bne { rs: Register, rt: Register, target: Pos },

	/// Branch if less than zero
	#[display(fmt = "bltz {rs}, {target:#x}")]
	Bltz { rs: Register, target: Pos },

	/// Branch if greater or equal to zero
	#[display(fmt = "bgez {rs}, {target:#x}")]
	Bgez { rs: Register, target: Pos },

	/// Branch if greater than zero
	#[display(fmt = "bgtz {rs}, {target:#x}")]
	Bgtz { rs: Register, target: Pos },

	/// Branch if less or equal to zero
	#[display(fmt = "blez {rs}, {target:#x}")]
	Blez { rs: Register, target: Pos },

	/// Branch if less than zero and link
	#[display(fmt = "bltzal {rs}, {target:#x}")]
	Bltzal { rs: Register, target: Pos },

	/// Branch if greater or equal to zero and link
	#[display(fmt = "bgezal {rs}, {target:#x}")]
	Bgezal { rs: Register, target: Pos },

	/// Save co-processor data registers
	#[display(fmt = "mfc{n} {rt}, {rd}")]
	MfcN { n: u8, rt: Register, rd: Register },

	/// Save co-processor control registers
	#[display(fmt = "cfc{n} {rt}, {rd}")]
	CfcN { n: u8, rt: Register, rd: Register },

	/// Load co-processor data registers
	#[display(fmt = "mtc{n} {rt}, {rd}")]
	MtcN { n: u8, rt: Register, rd: Register },

	/// Load co-processor control registers
	#[display(fmt = "ctc{n} {rt}, {rd}")]
	CtcN { n: u8, rt: Register, rd: Register },

	// TODO: Check how to calculate actual targets for these jumps
	//       Docs say `$+disp`, not sure if a typo or what, no 4
	//       multiple either, are co-processor instructions 1 byte?
	/// Branch co-processor if false
	#[display(fmt = "bc{n}f {target:#x} # Raw target")]
	BcNf { n: u8, target: u16 },

	/// Branch co-processor if true
	#[display(fmt = "bc{n}t {target:#x} # Raw target")]
	BcNt { n: u8, target: u16 },

	/// Exec immediate co-processor
	#[display(fmt = "cop{n} {imm:#x}")]
	CopN { n: u8, imm: u32 },

	/// Load word co-processor
	#[display(fmt = "lwc{n} {rt}, {imm:#x}({rs})")]
	LwcN { n: u8, rs: Register, rt: Register, imm: u16 },

	/// Store word co-processor
	#[display(fmt = "swc{n} {rt}, {imm:#x}({rs})")]
	SwcN { n: u8, rs: Register, rt: Register, imm: u16 },

	/// Syscall
	#[display(fmt = "sys {imm:#x}")]
	Syscall { imm: u32 },

	/// Break
	#[display(fmt = "break {imm:#x}")]
	Break { imm: u32 },
}

impl BasicInstruction {
	/// Decodes an instruction from it's raw representation
	#[bitmatch]
	#[allow(clippy::cognitive_complexity)] // It's just a big match, not much we can do about it.
	fn decode_repr(raws: &[u32], pos: Pos) -> Option<(Self, &[u32])> {
		#[allow(clippy::enum_glob_use)] // It's local to this function and REALLY reduces on the noise
		use BasicInstruction::*;

		/// Alias for `Register::new`
		fn reg(idx: u32) -> Option<Register> {
			Register::new(idx.truncated())
		}

		let (raw, raws) = match raws {
			[raw, raws @ ..] => (raw, raws),
			_ => return None,
		};

		#[rustfmt::skip]
		let instruction = #[bitmatch]
		match raw {
			"000000_?????_ttttt_ddddd_iiiii_000000" => Sll  { rd: reg(d)?, rt: reg(t)?, imm: i.truncated()},
			"000000_?????_ttttt_ddddd_iiiii_000010" => Srl  { rd: reg(d)?, rt: reg(t)?, imm: i.truncated()},
			"000000_?????_ttttt_ddddd_iiiii_000011" => Sra  { rd: reg(d)?, rt: reg(t)?, imm: i.truncated()},

			"000000_sssss_ttttt_ddddd_?????_000100" => Sllv { rd: reg(d)?, rt: reg(t)?, rs: reg(s)? },
			"000000_sssss_ttttt_ddddd_?????_000110" => Srlv { rd: reg(d)?, rt: reg(t)?, rs: reg(s)? },
			"000000_sssss_ttttt_ddddd_?????_000111" => Srav { rd: reg(d)?, rt: reg(t)?, rs: reg(s)? },

			"000000_sssss_?????_?????_?????_001000" => Jr      { rs: reg(s)? },
			"000000_sssss_?????_ddddd_?????_001001" => Jalr    { rd: reg(d)?, rs: reg(s)? },

			"000000_iiiii_iiiii_iiiii_iiiii_001100" => Syscall { imm: i },
			"000000_iiiii_iiiii_iiiii_iiiii_001101" => Break   { imm: i },

			"000000_?????_?????_ddddd_?????_010000" => Mfhi  { rd: reg(d)? },
			"000000_sssss_?????_?????_?????_010001" => Mthi  { rs: reg(s)? },
			"000000_?????_?????_ddddd_?????_010010" => Mflo  { rd: reg(d)? },
			"000000_sssss_?????_?????_?????_010011" => Mtlo  { rs: reg(s)? },

			"000000_sssss_ttttt_?????_?????_011000" => Mult  { rs: reg(s)?, rt: reg(t)? },
			"000000_sssss_ttttt_?????_?????_011001" => Multu { rs: reg(s)?, rt: reg(t)? },
			"000000_sssss_ttttt_?????_?????_011010" => Div   { rs: reg(s)?, rt: reg(t)? },
			"000000_sssss_ttttt_?????_?????_011011" => Divu  { rs: reg(s)?, rt: reg(t)? },

			"000000_sssss_ttttt_ddddd_?????_100000" => Add  { rd: reg(d)?, rs: reg(s)?, rt: reg(t)? },
			"000000_sssss_ttttt_ddddd_?????_100001" => Addu { rd: reg(d)?, rs: reg(s)?, rt: reg(t)? },
			"000000_sssss_ttttt_ddddd_?????_100010" => Sub  { rd: reg(d)?, rs: reg(s)?, rt: reg(t)? },
			"000000_sssss_ttttt_ddddd_?????_100011" => Subu { rd: reg(d)?, rs: reg(s)?, rt: reg(t)? },
			"000000_sssss_ttttt_ddddd_?????_100100" => And  { rd: reg(d)?, rs: reg(s)?, rt: reg(t)? },
			"000000_sssss_ttttt_ddddd_?????_100101" => Or   { rd: reg(d)?, rs: reg(s)?, rt: reg(t)? },
			"000000_sssss_ttttt_ddddd_?????_100110" => Xor  { rd: reg(d)?, rs: reg(s)?, rt: reg(t)? },
			"000000_sssss_ttttt_ddddd_?????_100111" => Nor  { rd: reg(d)?, rs: reg(s)?, rt: reg(t)? },

			"000000_sssss_ttttt_ddddd_?????_101010" => Slt  { rd: reg(d)?, rs: reg(s)?, rt: reg(t)? },
			"000000_sssss_ttttt_ddddd_?????_101011" => Sltu { rd: reg(d)?, rs: reg(s)?, rt: reg(t)? },

			"000001_sssss_?????_iiiii_iiiii_iiiiii" => Bltz   { rs: reg(s)?, target: pos + (i.truncated::<u16>().as_signed().sign_extended::<i32>() + 1) * 4 },
			"000001_sssss_?????_iiiii_iiiii_iiiiii" => Bgez   { rs: reg(s)?, target: pos + (i.truncated::<u16>().as_signed().sign_extended::<i32>() + 1) * 4 },
			"000001_sssss_?????_iiiii_iiiii_iiiiii" => Bltzal { rs: reg(s)?, target: pos + (i.truncated::<u16>().as_signed().sign_extended::<i32>() + 1) * 4 },
			"000001_sssss_?????_iiiii_iiiii_iiiiii" => Bgezal { rs: reg(s)?, target: pos + (i.truncated::<u16>().as_signed().sign_extended::<i32>() + 1) * 4 },

			"000010_iiiii_iiiii_iiiii_iiiii_iiiiii" => J      { target: (pos & 0xf000_0000) + i * 4 },
			"000011_iiiii_iiiii_iiiii_iiiii_iiiiii" => Jal    { target: (pos & 0xf000_0000) + i * 4 },

			"000100_sssss_ttttt_iiiii_iiiii_iiiiii" => Beq    { rs: reg(s)?, rt: reg(t)?, target: pos + (i.truncated::<u16>().as_signed().sign_extended::<i32>() + 1) * 4 },
			"000101_sssss_ttttt_iiiii_iiiii_iiiiii" => Bne    { rs: reg(s)?, rt: reg(t)?, target: pos + (i.truncated::<u16>().as_signed().sign_extended::<i32>() + 1) * 4 },
			"000110_sssss_?????_iiiii_iiiii_iiiiii" => Blez   { rs: reg(s)?                 , target: pos + (i.truncated::<u16>().as_signed().sign_extended::<i32>() + 1) * 4 },
			"000111_sssss_?????_iiiii_iiiii_iiiiii" => Bgtz   { rs: reg(s)?                 , target: pos + (i.truncated::<u16>().as_signed().sign_extended::<i32>() + 1) * 4 },

			"001000_sssss_ttttt_iiiii_iiiii_iiiiii" => Addi  { rt: reg(t)?, rs: reg(s)?, imm: i.truncated::<u16>().as_signed() },
			"001001_sssss_ttttt_iiiii_iiiii_iiiiii" => Addiu { rt: reg(t)?, rs: reg(s)?, imm: i.truncated::<u16>().as_signed() },
			"001010_sssss_ttttt_iiiii_iiiii_iiiiii" => Slti  { rt: reg(t)?, rs: reg(s)?, imm: i.truncated::<u16>().as_signed() },
			"001011_sssss_ttttt_iiiii_iiiii_iiiiii" => Sltiu { rt: reg(t)?, rs: reg(s)?, imm: i.truncated::<u16>().as_signed() },
			"001100_sssss_ttttt_iiiii_iiiii_iiiiii" => Andi  { rt: reg(t)?, rs: reg(s)?, imm: i.truncated() },
			"001101_sssss_ttttt_iiiii_iiiii_iiiiii" => Ori   { rt: reg(t)?, rs: reg(s)?, imm: i.truncated() },
			"001110_sssss_ttttt_iiiii_iiiii_iiiiii" => Xori  { rt: reg(t)?, rs: reg(s)?, imm: i.truncated() },
			"001111_?????_ttttt_iiiii_iiiii_iiiiii" => Lui   { rt: reg(t)?                 , imm: i.truncated() },

			"0100nn_1iiii_iiiii_iiiii_iiiii_iiiiii" => CopN { n: n.truncate(), imm: i},

			"0100nn_00000_ttttt_ddddd_?????_000000" => MfcN { n: n.truncate(), rt: reg(t)?, rd: reg(d)? },
			"0100nn_00010_ttttt_ddddd_?????_000000" => CfcN { n: n.truncate(), rt: reg(t)?, rd: reg(d)? },
			"0100nn_00100_ttttt_ddddd_?????_000000" => MtcN { n: n.truncate(), rt: reg(t)?, rd: reg(d)? },
			"0100nn_00110_ttttt_ddddd_?????_000000" => CtcN { n: n.truncate(), rt: reg(t)?, rd: reg(d)? },
			"0100nn_01000_00000_iiiii_iiiii_iiiiii" => BcNf { n: n.truncate(), target: i.truncate() },
			"0100nn_01000_00001_iiiii_iiiii_iiiiii" => BcNt { n: n.truncate(), target: i.truncate() },

			"100000_sssss_ttttt_iiiii_iiiii_iiiiii" => Lb  { rt: reg(t)?, rs: reg(s)?, offset: i.truncated::<u16>().as_signed() },
			"100001_sssss_ttttt_iiiii_iiiii_iiiiii" => Lh  { rt: reg(t)?, rs: reg(s)?, offset: i.truncated::<u16>().as_signed() },
			"100010_sssss_ttttt_iiiii_iiiii_iiiiii" => Lwl { rt: reg(t)?, rs: reg(s)?, offset: i.truncated::<u16>().as_signed() },
			"100011_sssss_ttttt_iiiii_iiiii_iiiiii" => Lw  { rt: reg(t)?, rs: reg(s)?, offset: i.truncated::<u16>().as_signed() },
			"100100_sssss_ttttt_iiiii_iiiii_iiiiii" => Lbu { rt: reg(t)?, rs: reg(s)?, offset: i.truncated::<u16>().as_signed() },
			"100101_sssss_ttttt_iiiii_iiiii_iiiiii" => Lhu { rt: reg(t)?, rs: reg(s)?, offset: i.truncated::<u16>().as_signed() },
			"100110_sssss_ttttt_iiiii_iiiii_iiiiii" => Lwr { rt: reg(t)?, rs: reg(s)?, offset: i.truncated::<u16>().as_signed() },

			"101000_sssss_ttttt_iiiii_iiiii_iiiiii" => Sb  { rt: reg(t)?, rs: reg(s)?, offset: i.truncated::<u16>().as_signed() },
			"101001_sssss_ttttt_iiiii_iiiii_iiiiii" => Sh  { rt: reg(t)?, rs: reg(s)?, offset: i.truncated::<u16>().as_signed() },
			"101010_sssss_ttttt_iiiii_iiiii_iiiiii" => Swl { rt: reg(t)?, rs: reg(s)?, offset: i.truncated::<u16>().as_signed() },
			"101011_sssss_ttttt_iiiii_iiiii_iiiiii" => Sw  { rt: reg(t)?, rs: reg(s)?, offset: i.truncated::<u16>().as_signed() },
			"101110_sssss_ttttt_iiiii_iiiii_iiiiii" => Swr { rt: reg(t)?, rs: reg(s)?, offset: i.truncated::<u16>().as_signed() },

			"1100nn_sssss_ttttt_iiiii_iiiii_iiiiii" => LwcN { n: n.truncate(), rs: reg(s)?, rt: reg(t)?, imm: i.truncate() },
			"1110nn_sssss_ttttt_iiiii_iiiii_iiiiii" => SwcN { n: n.truncate(), rs: reg(s)?, rt: reg(t)?, imm: i.truncate() },

			_ => return None,
		};

		Some((instruction, raws))
	}

	/// Encodes an instruction.
	#[bitmatch]
	#[must_use]
	pub fn encode_repr(self, pos: Pos) -> Raw {
		#[allow(clippy::enum_glob_use)] // It's local to this function and REALLY reduces on the noise
		use BasicInstruction::*;

		#[rustfmt::skip]
		let repr = match self {
			Sll { rd, rt, imm: i } => { let (d, t) = (rd.idx(), rt.idx()); bitpack!("000000_00000_ttttt_ddddd_iiiii_000000") },

			Srl  { rd, rt, imm: i } => { let (d, t) = (rd.idx(), rt.idx()); bitpack!("000000_00000_ttttt_ddddd_iiiii_000010") },
			Sra  { rd, rt, imm: i } => { let (d, t) = (rd.idx(), rt.idx()); bitpack!("000000_00000_ttttt_ddddd_iiiii_000011") },

			Sllv { rd, rt, rs } => { let (d, t, s) = (rd.idx(), rt.idx(), rs.idx()); bitpack!("000000_sssss_ttttt_ddddd_00000_000100") },
			Srlv { rd, rt, rs } => { let (d, t, s) = (rd.idx(), rt.idx(), rs.idx()); bitpack!("000000_sssss_ttttt_ddddd_00000_000110") },
			Srav { rd, rt, rs } => { let (d, t, s) = (rd.idx(), rt.idx(), rs.idx()); bitpack!("000000_sssss_ttttt_ddddd_00000_000111") },

			Jr   {     rs } => { let     s  =            rs.idx() ; bitpack!("000000_sssss_00000_00000_00000_001000") },
			Jalr { rd, rs } => { let (d, s) = (rd.idx(), rs.idx()); bitpack!("000000_sssss_00000_ddddd_00000_001001") },

			Syscall { imm: i } => { bitpack!("000000_iiiii_iiiii_iiiii_iiiii_001100") },
			Break   { imm: i } => { bitpack!("000000_iiiii_iiiii_iiiii_iiiii_001101") },

			Mfhi  { rd } => { let d = rd.idx(); bitpack!("000000_00000_00000_ddddd_00000_010000") },
			Mthi  { rs } => { let s = rs.idx(); bitpack!("000000_sssss_00000_00000_00000_010001") },
			Mflo  { rd } => { let d = rd.idx(); bitpack!("000000_00000_00000_ddddd_00000_010010") },
			Mtlo  { rs } => { let s = rs.idx(); bitpack!("000000_sssss_00000_00000_00000_010011") },

			Mult  { rs, rt } => { let (t, s) = (rt.idx(), rs.idx()); bitpack!("000000_sssss_ttttt_00000_00000_011000") },
			Multu { rs, rt } => { let (t, s) = (rt.idx(), rs.idx()); bitpack!("000000_sssss_ttttt_00000_00000_011001") },
			Div   { rs, rt } => { let (t, s) = (rt.idx(), rs.idx()); bitpack!("000000_sssss_ttttt_00000_00000_011010") },
			Divu  { rs, rt } => { let (t, s) = (rt.idx(), rs.idx()); bitpack!("000000_sssss_ttttt_00000_00000_011011") },

			Add  { rd, rs, rt } => { let (d, t, s) = (rd.idx(), rt.idx(), rs.idx()); bitpack!("000000_sssss_ttttt_ddddd_00000_100000") },
			Addu { rd, rs, rt } => { let (d, t, s) = (rd.idx(), rt.idx(), rs.idx()); bitpack!("000000_sssss_ttttt_ddddd_00000_100001") },
			Sub  { rd, rs, rt } => { let (d, t, s) = (rd.idx(), rt.idx(), rs.idx()); bitpack!("000000_sssss_ttttt_ddddd_00000_100010") },
			Subu { rd, rs, rt } => { let (d, t, s) = (rd.idx(), rt.idx(), rs.idx()); bitpack!("000000_sssss_ttttt_ddddd_00000_100011") },
			And  { rd, rs, rt } => { let (d, t, s) = (rd.idx(), rt.idx(), rs.idx()); bitpack!("000000_sssss_ttttt_ddddd_00000_100100") },
			Or   { rd, rs, rt } => { let (d, t, s) = (rd.idx(), rt.idx(), rs.idx()); bitpack!("000000_sssss_ttttt_ddddd_00000_100101") },
			Xor  { rd, rs, rt } => { let (d, t, s) = (rd.idx(), rt.idx(), rs.idx()); bitpack!("000000_sssss_ttttt_ddddd_00000_100110") },
			Nor  { rd, rs, rt } => { let (d, t, s) = (rd.idx(), rt.idx(), rs.idx()); bitpack!("000000_sssss_ttttt_ddddd_00000_100111") },

			Slt  { rd, rs, rt } => { let (d, t, s) = (rd.idx(), rt.idx(), rs.idx()); bitpack!("000000_sssss_ttttt_ddddd_00000_101010") },
			Sltu { rd, rs, rt } => { let (d, t, s) = (rd.idx(), rt.idx(), rs.idx()); bitpack!("000000_sssss_ttttt_ddddd_00000_101011") },

			Bltz   { rs, target } => { let s = rs.idx(); let i = (target - pos) / 4 - 1; bitpack!("000001_sssss_00000_iiiii_iiiii_iiiiii") },
			Bgez   { rs, target } => { let s = rs.idx(); let i = (target - pos) / 4 - 1; bitpack!("000001_sssss_00000_iiiii_iiiii_iiiiii") },
			Bltzal { rs, target } => { let s = rs.idx(); let i = (target - pos) / 4 - 1; bitpack!("000001_sssss_00000_iiiii_iiiii_iiiiii") },
			Bgezal { rs, target } => { let s = rs.idx(); let i = (target - pos) / 4 - 1; bitpack!("000001_sssss_00000_iiiii_iiiii_iiiiii") },

			J      { target } => { let i = (target - (pos & 0x0fff_ffff)) / 4; bitpack!("000010_iiiii_iiiii_iiiii_iiiii_iiiiii") },
			Jal    { target } => { let i = (target - (pos & 0x0fff_ffff)) / 4; bitpack!("000011_iiiii_iiiii_iiiii_iiiii_iiiiii") },

			Beq    { rs, rt, target } => { let (s, t) = (rs.idx(), rt.idx()); let i = (target - pos) / 4 - 1; bitpack!("000100_sssss_ttttt_iiiii_iiiii_iiiiii") },
			Bne    { rs, rt, target } => { let (s, t) = (rs.idx(), rt.idx()); let i = (target - pos) / 4 - 1; bitpack!("000101_sssss_ttttt_iiiii_iiiii_iiiiii") },
			Blez   { rs    , target } => { let  s     =  rs.idx()           ; let i = (target - pos) / 4 - 1; bitpack!("000110_sssss_00000_iiiii_iiiii_iiiiii") },
			Bgtz   { rs    , target } => { let  s     =  rs.idx()           ; let i = (target - pos) / 4 - 1; bitpack!("000111_sssss_00000_iiiii_iiiii_iiiiii") },

			Addi  { rt, rs, imm: i } => { let (t, s) = (rt.idx(), rs.idx()); bitpack!("001000_sssss_ttttt_iiiii_iiiii_iiiiii") },
			Addiu { rt, rs, imm: i } => { let (t, s) = (rt.idx(), rs.idx()); bitpack!("001001_sssss_ttttt_iiiii_iiiii_iiiiii") },
			Slti  { rt, rs, imm: i } => { let (t, s) = (rt.idx(), rs.idx()); bitpack!("001010_sssss_ttttt_iiiii_iiiii_iiiiii") },
			Sltiu { rt, rs, imm: i } => { let (t, s) = (rt.idx(), rs.idx()); bitpack!("001011_sssss_ttttt_iiiii_iiiii_iiiiii") },
			Andi  { rt, rs, imm: i } => { let (t, s) = (rt.idx(), rs.idx()); bitpack!("001100_sssss_ttttt_iiiii_iiiii_iiiiii") },
			Ori   { rt, rs, imm: i } => { let (t, s) = (rt.idx(), rs.idx()); bitpack!("001101_sssss_ttttt_iiiii_iiiii_iiiiii") },
			Xori  { rt, rs, imm: i } => { let (t, s) = (rt.idx(), rs.idx()); bitpack!("001110_sssss_ttttt_iiiii_iiiii_iiiiii") },
			Lui   { rt,     imm: i } => { let  t     =  rt.idx();            bitpack!("001111_00000_ttttt_iiiii_iiiii_iiiiii") },

			CopN { n, imm: i } => { bitpack!("0100nn_1iiii_iiiii_iiiii_iiiii_iiiiii") },

			MfcN { n, rt, rd } => { let (t, d) = (rt.idx(), rd.idx()); bitpack!("0100nn_00000_ttttt_ddddd_00000_000000") },
			CfcN { n, rt, rd } => { let (t, d) = (rt.idx(), rd.idx()); bitpack!("0100nn_00010_ttttt_ddddd_00000_000000") },
			MtcN { n, rt, rd } => { let (t, d) = (rt.idx(), rd.idx()); bitpack!("0100nn_00100_ttttt_ddddd_00000_000000") },
			CtcN { n, rt, rd } => { let (t, d) = (rt.idx(), rd.idx()); bitpack!("0100nn_00110_ttttt_ddddd_00000_000000") },
			BcNf { n, target: i } => { bitpack!("0100nn_01000_00000_iiiii_iiiii_iiiiii") },
			BcNt { n, target: i } => { bitpack!("0100nn_01000_00001_iiiii_iiiii_iiiiii") },

			Lb  { rt, rs, offset: i } => { let (t, s) = (rt.idx(), rs.idx()); bitpack!("100000_sssss_ttttt_iiiii_iiiii_iiiiii") },
			Lh  { rt, rs, offset: i } => { let (t, s) = (rt.idx(), rs.idx()); bitpack!("100001_sssss_ttttt_iiiii_iiiii_iiiiii") },
			Lwl { rt, rs, offset: i } => { let (t, s) = (rt.idx(), rs.idx()); bitpack!("100010_sssss_ttttt_iiiii_iiiii_iiiiii") },
			Lw  { rt, rs, offset: i } => { let (t, s) = (rt.idx(), rs.idx()); bitpack!("100011_sssss_ttttt_iiiii_iiiii_iiiiii") },
			Lbu { rt, rs, offset: i } => { let (t, s) = (rt.idx(), rs.idx()); bitpack!("100100_sssss_ttttt_iiiii_iiiii_iiiiii") },
			Lhu { rt, rs, offset: i } => { let (t, s) = (rt.idx(), rs.idx()); bitpack!("100101_sssss_ttttt_iiiii_iiiii_iiiiii") },
			Lwr { rt, rs, offset: i } => { let (t, s) = (rt.idx(), rs.idx()); bitpack!("100110_sssss_ttttt_iiiii_iiiii_iiiiii") },

			Sb  { rt, rs, offset: i } => { let (t, s) = (rt.idx(), rs.idx()); bitpack!("101000_sssss_ttttt_iiiii_iiiii_iiiiii") },
			Sh  { rt, rs, offset: i } => { let (t, s) = (rt.idx(), rs.idx()); bitpack!("101001_sssss_ttttt_iiiii_iiiii_iiiiii") },
			Swl { rt, rs, offset: i } => { let (t, s) = (rt.idx(), rs.idx()); bitpack!("101010_sssss_ttttt_iiiii_iiiii_iiiiii") },
			Sw  { rt, rs, offset: i } => { let (t, s) = (rt.idx(), rs.idx()); bitpack!("101011_sssss_ttttt_iiiii_iiiii_iiiiii") },
			Swr { rt, rs, offset: i } => { let (t, s) = (rt.idx(), rs.idx()); bitpack!("101110_sssss_ttttt_iiiii_iiiii_iiiiii") },

			LwcN { n, rs, rt, imm: i } => { let (t, s) = (rt.idx(), rs.idx()); bitpack!("1100nn_sssss_ttttt_iiiii_iiiii_iiiiii") },
			SwcN { n, rs, rt, imm: i } => { let (t, s) = (rt.idx(), rs.idx()); bitpack!("1110nn_sssss_ttttt_iiiii_iiiii_iiiiii") },
		};

		Raw { repr, pos }
	}
}

impl FromRawIter for BasicInstruction {
	type Decoded = Option<(Pos, Self)>;

	fn decode<I: Iterator<Item = Raw> + Clone>(iter: &mut I) -> Self::Decoded {
		let raw = iter.next()?;
		let (instruction, _) = Self::decode_repr(std::slice::from_ref(&raw.repr), raw.pos)?;
		Some((raw.pos, instruction))
	}
}
