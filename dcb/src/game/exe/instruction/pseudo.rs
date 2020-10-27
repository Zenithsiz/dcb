//! Pseudo instructions

// Imports
use super::{FromRawIter, Raw, Register, SimpleInstruction};
use crate::{game::exe::Pos, util::SignedHex};
use int_conv::{Join, SignExtended, Signed, ZeroExtended};

/// A pseudo instruction
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[derive(derive_more::Display)]
#[allow(clippy::missing_docs_in_private_items)] // Mostly just register names and immediates.
pub enum PseudoInstruction {
	/// No-op
	/// Alias for `sll $zr,$zr,0`
	#[display(fmt = "nop")]
	Nop,

	/// Move register
	/// Alias for `{add|addu|sub|subu|and|or|xor|sllv|srlv|srav} $.., $.., $zr` or
	/// `{addi|addiu|andi|ori|xori|sll|srl|sra} $.., $.., 0`
	#[display(fmt = "move {rx}, {ry}")]
	MovReg { rx: Register, ry: Register },

	/// Load byte immediate
	/// Alias for `lui $rx, {offset-hi} / lb $rx, {offset-lo}($rx)`
	#[display(fmt = "lb {rx}, {offset:#x}")]
	LbImm { rx: Register, offset: u32 },

	/// Load byte unsigned immediate
	/// Alias for `lui $rx, {offset-hi} / lbu $rx, {offset-lo}($rx)`
	#[display(fmt = "lbu {rx}, {offset:#x}")]
	LbuImm { rx: Register, offset: u32 },

	/// Load half-word immediate
	/// Alias for `lui $rx, {offset-hi} / lh $rx, {offset-lo}($rx)`
	#[display(fmt = "lh {rx}, {offset:#x}")]
	LhImm { rx: Register, offset: u32 },

	/// Load half-word unsigned immediate
	/// Alias for `lui $rx, {offset-hi} / lhu $rx, {offset-lo}($rx)`
	#[display(fmt = "lh {rx}, {offset:#x}")]
	LhuImm { rx: Register, offset: u32 },

	/// Load left word immediate
	/// Alias for `lui $rx, {offset-hi} / lwl $rx, {offset-lo}($rx)`
	#[display(fmt = "lwl {rx}, {offset:#x}")]
	LwlImm { rx: Register, offset: u32 },

	/// Load word immediate
	/// Alias for `lui $rx, {offset-hi} / lw $rx, {offset-lo}($rx)`
	#[display(fmt = "lw {rx}, {offset:#x}")]
	LwImm { rx: Register, offset: u32 },

	/// Load right word immediate
	/// Alias for `lui $rx, {offset-hi} / lwr $rx, {offset-lo}($rx)`
	#[display(fmt = "lwr {rx}, {offset:#x}")]
	LwrImm { rx: Register, offset: u32 },

	/// Store byte immediate
	/// Alias for `lui $at, {offset-hi} / sb $rx, {offset-lo}($at)`
	#[display(fmt = "sb {rx}, {offset:#x}")]
	SbImm { rx: Register, offset: u32 },

	/// Store half-word immediate
	/// Alias for `lui $at, {offset-hi} / sh $rx, {offset-lo}($at)`
	#[display(fmt = "sh {rx}, {offset:#x}")]
	ShImm { rx: Register, offset: u32 },

	/// Store left word immediate
	/// Alias for `lui $at, {offset-hi} / swl $rx, {offset-lo}($at)`
	#[display(fmt = "swl {rx}, {offset:#x}")]
	SwlImm { rx: Register, offset: u32 },

	/// Store word immediate
	/// Alias for `lui $at, {offset-hi} / sw $rx, {offset-lo}($at)`
	#[display(fmt = "sw {rx}, {offset:#x}")]
	SwImm { rx: Register, offset: u32 },

	/// Store right word immediate
	/// Alias for `lui $at, {offset-hi} / swr $rx, {offset-lo}($at)`
	#[display(fmt = "swr {rx}, {offset:#x}")]
	SwrImm { rx: Register, offset: u32 },

	/// Load address
	/// Alias for `lui $rx, {target-hi} / addiu $rx, $rx, {target-lo}`
	#[display(fmt = "la {rx}, {target:#x}")]
	La { rx: Register, target: u32 },

	/// Load immediate 32-bit
	/// Alias for `lui $rx, {imm-hi} / ori $rx, $rx, {imm-lo}`
	#[display(fmt = "li {rx}, {imm:#x}")]
	Li32 { rx: Register, imm: u32 },

	/// Load unsigned immediate 16-bit
	/// Alias for `ori $rx, $zr, imm`
	#[display(fmt = "li {rx}, {imm:#x}")]
	LiU16 { rx: Register, imm: u16 },

	/// Load signed immediate negative 16-bit
	/// Alias for `addiu $rx, $zr, imm`
	#[display(fmt = "li {rx}, {:#x}", "SignedHex(imm)")]
	LiI16 { rx: Register, imm: i16 },

	/// Load immediate upper 16-bits
	/// Alias for `lui 0x1000 * imm`
	#[display(fmt = "li {rx}, {:#x}", "imm.zero_extended::<u32>() << 16")]
	LiUpper16 { rx: Register, imm: u16 },

	/// Add assign
	/// Alias for `add $rx, $rx, $rt`
	#[display(fmt = "add {rx}, {rt}")]
	AddAssign { rx: Register, rt: Register },

	/// Add unsigned assign
	/// Alias for `addu $rx, $rx, $rt`
	#[display(fmt = "addu {rx}, {rt}")]
	AdduAssign { rx: Register, rt: Register },

	/// Sub assign
	/// Alias for `sub $rx, $rx, $rt`
	#[display(fmt = "sub {rx}, {rt}")]
	SubAssign { rx: Register, rt: Register },

	/// Sub unsigned assign
	/// Alias for `subu $rx, $rx, $rt`
	#[display(fmt = "subu {rx}, {rt}")]
	SubuAssign { rx: Register, rt: Register },

	/// And assign
	/// Alias for `and $rx, $rx, $rt`
	#[display(fmt = "and {rx}, {rt}")]
	AndAssign { rx: Register, rt: Register },

	/// Or assign
	/// Alias for `or $rx, $rx, $rt`
	#[display(fmt = "or {rx}, {rt}")]
	OrAssign { rx: Register, rt: Register },

	/// Xor assign
	/// Alias for `xor $rx, $rx, $rt`
	#[display(fmt = "xor {rx}, {rt}")]
	XorAssign { rx: Register, rt: Register },

	/// Nor assign
	/// Alias for `nor $rx, $rx, $rt`
	#[display(fmt = "nor {rx}, {rt}")]
	NorAssign { rx: Register, rt: Register },

	/// Add immediate assign
	/// Alias for `addi $rx, $rx, imm`
	#[display(fmt = "addi {rx}, {:#x}", "SignedHex(imm)")]
	AddiAssign { rx: Register, imm: i16 },

	/// Add immediate sign-extended assign
	/// Alias for `addiu $rx, $rx, imm`
	#[display(fmt = "addiu {rx}, {:#x}", "SignedHex(imm)")]
	AddiuAssign { rx: Register, imm: i16 },

	/// And immediate assign
	/// Alias for `andi $rx, $rx, imm`
	#[display(fmt = "andi {rx}, {imm:#x}")]
	AndiAssign { rx: Register, imm: u16 },

	/// Or immediate assign
	/// Alias for `ori $rx, $rx, imm`
	#[display(fmt = "ori {rx}, {imm:#x}")]
	OriAssign { rx: Register, imm: u16 },

	/// Xor immediate assign
	/// Alias for `xori $rx, $rx, imm`
	#[display(fmt = "xori {rx}, {imm:#x}")]
	XoriAssign { rx: Register, imm: u16 },

	/// Shift left logical variable assign
	/// Alias for `sllv $rx, $rx, $rs`
	#[display(fmt = "sllv {rx} {rs}")]
	SllvAssign { rx: Register, rs: Register },

	/// Shift right logical variable assign
	/// Alias for `srlv $rx, $rx, $rs`
	#[display(fmt = "srlv {rx} {rs}")]
	SrlvAssign { rx: Register, rs: Register },

	/// Shift right arithmetic variable assign
	/// Alias for `srav $rx, $rx, $rs`
	#[display(fmt = "srav {rx} {rs}")]
	SravAssign { rx: Register, rs: Register },

	/// Shift left logical assign
	/// Alias for `sll $rx, $rx, imm`
	#[display(fmt = "sll {rx} {imm:#x}")]
	SllAssign { rx: Register, imm: u8 },

	/// Shift right logical assign
	/// Alias for `srl $rx, $rx, imm`
	#[display(fmt = "srl {rx} {imm:#x}")]
	SrlAssign { rx: Register, imm: u8 },

	/// Shift right arithmetic assign
	/// Alias for `sla $rx, $rx, imm`
	#[display(fmt = "sra {rx} {imm:#x}")]
	SraAssign { rx: Register, imm: u8 },

	/// Jump and link with return address
	/// Alias for `jalr $ra, $rx`
	#[display(fmt = "jalr {rx}")]
	JalrRa { rx: Register },

	/// Subtract immediate
	/// Alias for `addi $rt, $rs, -imm`
	#[display(fmt = "subi {rt}, {rs}, {imm:#x}")]
	Subi { rt: Register, rs: Register, imm: u32 },

	/// Subtract immediate sign-extended
	/// Alias for `addiu $rt, $rs, -imm`
	#[display(fmt = "subiu {rt}, {rs}, {imm:#x}")]
	Subiu { rt: Register, rs: Register, imm: u32 },

	/// Subtract immediate assign
	/// Alias for `subi $rx, $rx, imm`
	#[display(fmt = "subi {rx}, {imm:#x}")]
	SubiAssign { rx: Register, imm: u32 },

	/// Subtract immediate sign-extended assign
	/// Alias for `subiu $rx, $rx, imm`
	#[display(fmt = "subiu {rx}, {imm:#x}")]
	SubiuAssign { rx: Register, imm: u32 },

	/// Branch if equal to zero
	/// Alias for `beq $rx, $zr, target`
	#[display(fmt = "beqz {rx}, {target:#x}")]
	Beqz { rx: Register, target: Pos },

	/// Branch if different from zero
	/// Alias for `bne $rx, $zr, target`
	#[display(fmt = "bnez {rx}, {target:#x}")]
	Bnez { rx: Register, target: Pos },

	/// Jump relative
	/// Alias for `beq $zr, $zr, target`
	#[display(fmt = "b {target:#x}")]
	B { target: Pos },
	// TODO: Push / Pop
}

impl FromRawIter for PseudoInstruction {
	type Decoded = Option<(Pos, Self)>;

	#[allow(clippy::similar_names)] // With register names, this happens too much
	#[allow(clippy::too_many_lines, clippy::clippy::cognitive_complexity)] // We can't separate this into several functions, it's just 1 big match
	#[allow(clippy::enum_glob_use)] // This reduces the amount of typing for simple instructions and registers
	fn decode<I: Iterator<Item = Raw> + Clone>(iter: &mut I) -> Self::Decoded {
		use Register::*;
		use SimpleInstruction::*;

		// Get the first instruction
		let (pos, instruction) = SimpleInstruction::decode(iter)?;
		let pseudo = match instruction {
			Lui { imm: imm_hi, rt: prev_rt } => {
				let iter_before = iter.clone();
				match SimpleInstruction::decode(iter)?.1 {
					Addiu { imm: imm_lo, rt, rs } if rt == prev_rt && rs == prev_rt => Self::La {
						rx:     prev_rt,
						// Note: `imm_lo` is signed
						target: (imm_hi.zero_extended::<u32>().as_signed() + imm_lo.sign_extended::<i32>()).as_unsigned(),
					},
					Ori { imm: imm_lo, rt, rs } if rt == prev_rt && rs == prev_rt => Self::Li32 {
						rx:  prev_rt,
						imm: u32::join(imm_lo, imm_hi),
					},

					Lb { offset: imm_lo, rt, rs } if rt == prev_rt && rs == prev_rt => Self::LbImm {
						rx:     prev_rt,
						offset: (imm_hi.zero_extended::<u32>().as_signed() + imm_lo.sign_extended::<i32>()).as_unsigned(),
					},
					Lbu { offset: imm_lo, rt, rs } if rt == prev_rt && rs == prev_rt => Self::LbuImm {
						rx:     prev_rt,
						offset: (imm_hi.zero_extended::<u32>().as_signed() + imm_lo.sign_extended::<i32>()).as_unsigned(),
					},
					Lh { offset: imm_lo, rt, rs } if rt == prev_rt && rs == prev_rt => Self::LhImm {
						rx:     prev_rt,
						offset: (imm_hi.zero_extended::<u32>().as_signed() + imm_lo.sign_extended::<i32>()).as_unsigned(),
					},
					Lhu { offset: imm_lo, rt, rs } if rt == prev_rt && rs == prev_rt => Self::LhuImm {
						rx:     prev_rt,
						offset: (imm_hi.zero_extended::<u32>().as_signed() + imm_lo.sign_extended::<i32>()).as_unsigned(),
					},
					Lwl { offset: imm_lo, rt, rs } if rt == prev_rt && rs == prev_rt => Self::LwlImm {
						rx:     prev_rt,
						offset: (imm_hi.zero_extended::<u32>().as_signed() + imm_lo.sign_extended::<i32>()).as_unsigned(),
					},
					Lw { offset: imm_lo, rt, rs } if rt == prev_rt && rs == prev_rt => Self::LwImm {
						rx:     prev_rt,
						offset: (imm_hi.zero_extended::<u32>().as_signed() + imm_lo.sign_extended::<i32>()).as_unsigned(),
					},
					Lwr { offset: imm_lo, rt, rs } if rt == prev_rt && rs == prev_rt => Self::LwrImm {
						rx:     prev_rt,
						offset: (imm_hi.zero_extended::<u32>().as_signed() + imm_lo.sign_extended::<i32>()).as_unsigned(),
					},

					Sb { offset: imm_lo, rt, rs } if prev_rt == At && rs == At => Self::SbImm {
						rx:     rt,
						offset: (imm_hi.zero_extended::<u32>().as_signed() + imm_lo.sign_extended::<i32>()).as_unsigned(),
					},
					Sh { offset: imm_lo, rt, rs } if prev_rt == At && rs == At => Self::ShImm {
						rx:     rt,
						offset: (imm_hi.zero_extended::<u32>().as_signed() + imm_lo.sign_extended::<i32>()).as_unsigned(),
					},
					Swl { offset: imm_lo, rt, rs } if prev_rt == At && rs == At => Self::SwlImm {
						rx:     rt,
						offset: (imm_hi.zero_extended::<u32>().as_signed() + imm_lo.sign_extended::<i32>()).as_unsigned(),
					},
					Sw { offset: imm_lo, rt, rs } if prev_rt == At && rs == At => Self::SwImm {
						rx:     rt,
						offset: (imm_hi.zero_extended::<u32>().as_signed() + imm_lo.sign_extended::<i32>()).as_unsigned(),
					},
					Swr { offset: imm_lo, rt, rs } if prev_rt == At && rs == At => Self::SwrImm {
						rx:     rt,
						offset: (imm_hi.zero_extended::<u32>().as_signed() + imm_lo.sign_extended::<i32>()).as_unsigned(),
					},
					// Since we don't use the value, reset the iterator to it's previous value.
					_ => {
						*iter = iter_before;
						Self::LiUpper16 { rx: prev_rt, imm: imm_hi }
					},
				}
			},

			Sll { rd: Zr, rt: Zr, imm: 0 } => Self::Nop,

			#[rustfmt::skip]
			Add   { rd: rx, rs: ry, rt: Zr } |
			Addu  { rd: rx, rs: ry, rt: Zr } |
			Sub   { rd: rx, rs: ry, rt: Zr } |
			Subu  { rd: rx, rs: ry, rt: Zr } |
			And   { rd: rx, rs: ry, rt: Zr } |
			Or    { rd: rx, rs: ry, rt: Zr } |
			Xor   { rd: rx, rs: ry, rt: Zr } |
			Sllv  { rd: rx, rt: ry, rs: Zr } |
			Srlv  { rd: rx, rt: ry, rs: Zr } |
			Srav  { rd: rx, rt: ry, rs: Zr } |
			Addi  { rt: rx, rs: ry, imm: 0 } |
			Addiu { rt: rx, rs: ry, imm: 0 } |
			Andi  { rt: rx, rs: ry, imm: 0 } |
			Ori   { rt: rx, rs: ry, imm: 0 } |
			Xori  { rt: rx, rs: ry, imm: 0 } |
			Sll   { rd: rx, rt: ry, imm: 0 } |
			Srl   { rd: rx, rt: ry, imm: 0 } |
			Sra   { rd: rx, rt: ry, imm: 0 } => Self::MovReg { rx, ry },

			Ori { rt: rx, rs: Zr, imm } => Self::LiU16 { rx, imm },
			Addiu { rt: rx, rs: Zr, imm } => Self::LiI16 { rx, imm },

			#[rustfmt::skip]
			Addi { rt, rs, imm: imm @ i16::MIN..0 } => match rt == rs {
				true => Self::SubiAssign {
					rx:  rt,
					imm: imm.sign_extended::<i32>().abs().as_unsigned(),
				},
				false => Self::Subi {
					rt, rs,
					imm: imm.sign_extended::<i32>().abs().as_unsigned(),
				},
			},

			#[rustfmt::skip]
			Addiu { rt, rs, imm: imm @ i16::MIN..0 } => match rt == rs {
				true => Self::SubiuAssign {
					rx:  rt,
					imm: imm.sign_extended::<i32>().abs().as_unsigned(),
				},
				false => Self::Subiu {
					rt,
					rs,
					imm: imm.sign_extended::<i32>().abs().as_unsigned(),
				},
			},

			Add { rd, rs, rt } if rd == rs => Self::AddAssign { rx: rd, rt },
			Addu { rd, rs, rt } if rd == rs => Self::AdduAssign { rx: rd, rt },
			Sub { rd, rs, rt } if rd == rs => Self::SubAssign { rx: rd, rt },
			Subu { rd, rs, rt } if rd == rs => Self::SubuAssign { rx: rd, rt },

			And { rd, rs, rt } if rd == rs => Self::AndAssign { rx: rd, rt },
			Or { rd, rs, rt } if rd == rs => Self::OrAssign { rx: rd, rt },
			Xor { rd, rs, rt } if rd == rs => Self::XorAssign { rx: rd, rt },
			Nor { rd, rs, rt } if rd == rs => Self::NorAssign { rx: rd, rt },

			Addi { rt, rs, imm } if rt == rs => Self::AddiAssign { rx: rt, imm },
			Addiu { rt, rs, imm } if rt == rs => Self::AddiuAssign { rx: rt, imm },

			Andi { rt, rs, imm } if rt == rs => Self::AndiAssign { rx: rt, imm },
			Ori { rt, rs, imm } if rt == rs => Self::OriAssign { rx: rt, imm },
			Xori { rt, rs, imm } if rt == rs => Self::XoriAssign { rx: rt, imm },

			Sllv { rd, rt, rs } if rd == rt => Self::SllvAssign { rx: rd, rs },
			Srlv { rd, rt, rs } if rd == rt => Self::SrlvAssign { rx: rd, rs },
			Srav { rd, rt, rs } if rd == rt => Self::SravAssign { rx: rd, rs },

			Sll { rd, rt, imm } if rd == rt => Self::SllAssign { rx: rd, imm },
			Srl { rd, rt, imm } if rd == rt => Self::SrlAssign { rx: rd, imm },
			Sra { rd, rt, imm } if rd == rt => Self::SraAssign { rx: rd, imm },

			Jalr { rd: Ra, rs: rx } => Self::JalrRa { rx },

			Beq { rs: Zr, rt: Zr, target } => Self::B { target },
			Beq { rs: rx, rt: Zr, target } => Self::Beqz { rx, target },
			Bne { rs: rx, rt: Zr, target } => Self::Bnez { rx, target },

			// Note: No need to reset iterator, it returned `None`.
			_ => return None,
		};

		Some((pos, pseudo))
	}
}
