//! Raw instructions

// Lints
// #[allow(clippy::similar_names)]

// Modules
pub mod repr;

// Exports
pub use repr::RawRepr;

// Imports
use super::{FromRawIter, Pos, Raw, Register};
use crate::util::SignedHex;
use int_conv::{SignExtended, Signed};

/// Macro to declare all instructions
macro_rules! decl_instructions {
	(
		$(
			$( #[doc = $doc:literal] )*
			#[display(fmt = $fmt:literal $( , $fmt_args:expr )* $(,)?)]
			$split:pat $( if $cond:expr )? => $variant:ident {
				$( $field_name:ident : $field_type:ty $( = $field_expr: expr )? ),* $(,)?
			}
		),+ $(,)?
	) => {
		/// A raw instruction
		#[derive(PartialEq, Eq, Clone, Copy, Debug)]
		#[derive(derive_more::Display)]
		pub enum SimpleInstruction {
			$(
				$( #[doc = $doc] )*
				#[display(fmt = $fmt, $( $fmt_args, )*)]
				$variant {
					$(
						$field_name: $field_type,
					)*
				},
			)+
		}

		impl FromRawIter for SimpleInstruction {
			type Decoded = Option<(Pos, Self)>;

			#[allow(clippy::redundant_field_names)] // For uniform initialization
			fn decode<I: Iterator<Item = Raw> + Clone>(iter: &mut I) -> Self::Decoded {
				let raw = iter.next()?;
				let split = RawRepr::new(raw);

				let instruction = match split {
					$(
						$split $( if $cond )? => Some( Self::$variant {
							$(
								$field_name $( : $field_expr )?,
							)*
						} ),
					)*

					_ => None,
				};

				instruction.map(|instruction| (raw.pos, instruction))
			}
		}
	}
}

decl_instructions! {
	/// Store byte
	#[display(fmt = "sb {rt}, {offset:#x}({rs})")]
	RawRepr {
		op: 0x28,
		rs, rt, imm16,
		..
	} => Sb {
		rt: Register = Register::new(rt)?,
		rs: Register = Register::new(rs)?,
		offset: u16 = imm16,
	},

	/// Store half-word
	#[display(fmt = "sh {rt}, {offset:#x}({rs})")]
	RawRepr {
		op: 0x29,
		rs, rt, imm16,
		..
	} => Sh {
		rt: Register = Register::new(rt)?,
		rs: Register = Register::new(rs)?,
		offset: u16 = imm16,
	},

	/// Store left word
	#[display(fmt = "swl {rt}, {offset:#x}({rs})")]
	RawRepr {
		op: 0x2a,
		rs, rt, imm16,
		..
	} => Swl {
		rt: Register = Register::new(rt)?,
		rs: Register = Register::new(rs)?,
		offset: u16 = imm16,
	},

	/// Store word
	#[display(fmt = "sw {rt}, {offset:#x}({rs})")]
	RawRepr {
		op: 0x2b,
		rs, rt, imm16,
		..
	} => Sw {
		rt: Register = Register::new(rt)?,
		rs: Register = Register::new(rs)?,
		offset: u16 = imm16,
	},

	/// Store right word
	#[display(fmt = "swr {rt}, {offset:#x}({rs})")]
	RawRepr {
		op: 0x2e,
		rs, rt, imm16,
		..
	} => Swr {
		rt: Register = Register::new(rt)?,
		rs: Register = Register::new(rs)?,
		offset: u16 = imm16,
	},



	/// Load byte
	#[display(fmt = "lb {rt}, {offset:#x}({rs})")]
	RawRepr {
		op: 0x20,
		rs, rt, imm16,
		..
	} => Lb {
		rt: Register = Register::new(rt)?,
		rs: Register = Register::new(rs)?,
		offset: u16 = imm16,
	},

	/// Load byte unsigned
	#[display(fmt = "lbu {rt}, {offset:#x}({rs})")]
	RawRepr {
		op: 0x24,
		rs, rt, imm16,
		..
	} => Lbu {
		rt: Register = Register::new(rt)?,
		rs: Register = Register::new(rs)?,
		offset: u16 = imm16,
	},

	/// Load half-word
	#[display(fmt = "lh {rt}, {offset:#x}({rs})")]
	RawRepr {
		op: 0x21,
		rs, rt, imm16,
		..
	} => Lh {
		rt: Register = Register::new(rt)?,
		rs: Register = Register::new(rs)?,
		offset: u16 = imm16,
	},

	/// Load half-word unsigned
	#[display(fmt = "lhu {rt}, {offset:#x}({rs})")]
	RawRepr {
		op: 0x25,
		rs, rt, imm16,
		..
	} => Lhu {
		rt: Register = Register::new(rt)?,
		rs: Register = Register::new(rs)?,
		offset: u16 = imm16,
	},

	/// Load left word
	#[display(fmt = "lwl {rt}, {offset:#x}({rs})")]
	RawRepr {
		op: 0x22,
		rs, rt, imm16,
		..
	} => Lwl {
		rt: Register = Register::new(rt)?,
		rs: Register = Register::new(rs)?,
		offset: u16 = imm16,
	},

	/// Load word
	#[display(fmt = "lw {rt}, {offset:#x}({rs})")]
	RawRepr {
		op: 0x23,
		rs, rt, imm16,
		..
	} => Lw {
		rt: Register = Register::new(rt)?,
		rs: Register = Register::new(rs)?,
		offset: u16 = imm16,
	},

	/// Load right word
	#[display(fmt = "lwr {rt}, {offset:#x}({rs})")]
	RawRepr {
		op: 0x26,
		rs, rt, imm16,
		..
	} => Lwr {
		rt: Register = Register::new(rt)?,
		rs: Register = Register::new(rs)?,
		offset: u16 = imm16,
	},

	/// Add
	#[display(fmt = "add {rd}, {rs}, {rt}")]
	RawRepr {
		op: 0x00, op2: 0x20,
		rd, rs, rt,
		..
	} => Add {
		rd: Register = Register::new(rd)?,
		rs: Register = Register::new(rs)?,
		rt: Register = Register::new(rt)?,
	},

	/// Add unsigned
	#[display(fmt = "addu {rd}, {rs}, {rt}")]
	RawRepr {
		op: 0x00, op2: 0x21,
		rd, rs, rt,
		..
	} => Addu {
		rd: Register = Register::new(rd)?,
		rs: Register = Register::new(rs)?,
		rt: Register = Register::new(rt)?,
	},

	/// Sub
	#[display(fmt = "sub {rd}, {rs}, {rt}")]
	RawRepr {
		op: 0x00, op2: 0x22,
		rd, rs, rt,
		..
	} => Sub {
		rd: Register = Register::new(rd)?,
		rs: Register = Register::new(rs)?,
		rt: Register = Register::new(rt)?,
	},

	/// Sub unsigned
	#[display(fmt = "subu {rd}, {rs}, {rt}")]
	RawRepr {
		op: 0x00, op2: 0x23,
		rd, rs, rt,
		..
	} => Subu {
		rd: Register = Register::new(rd)?,
		rs: Register = Register::new(rs)?,
		rt: Register = Register::new(rt)?,
	},

	/// Add immediate
	#[display(fmt = "addi {rt}, {rs}, {:#x}", "SignedHex(imm)")]
	RawRepr {
		op: 0x08,
		rt, rs, imm16,
		..
	} => Addi {
		rt: Register = Register::new(rt)?,
		rs: Register = Register::new(rs)?,
		imm: i16 = imm16.as_signed(),
	},

	/// Add immediate sign-extended
	/// Note: _NOT_ Unsigned.
	#[display(fmt = "addiu {rt}, {rs}, {:#x}", "SignedHex(imm)")]
	RawRepr {
		op: 0x09,
		rt, rs, imm16,
		..
	} => Addiu {
		rt: Register = Register::new(rt)?,
		rs: Register = Register::new(rs)?,
		imm: i16 = imm16.as_signed(),
	},

	/// Set less than
	#[display(fmt = "slt {rd}, {rs}, {rt}")]
	RawRepr {
		op: 0x00, op2: 0x2a,
		rd, rs, rt,
		..
	} => Slt {
		rd: Register = Register::new(rd)?,
		rs: Register = Register::new(rs)?,
		rt: Register = Register::new(rt)?,
	},

	/// Set less than unsigned
	#[display(fmt = "sltu {rd}, {rs}, {rt}")]
	RawRepr {
		op: 0x00, op2: 0x2b,
		rd, rs, rt,
		..
	} => Sltu {
		rd: Register = Register::new(rd)?,
		rs: Register = Register::new(rs)?,
		rt: Register = Register::new(rt)?,
	},

	/// Set less than immediate
	#[display(fmt = "slti {rt}, {rs}, {:#x}", "SignedHex(imm)")]
	RawRepr {
		op: 0x0a,
		rt, rs, imm16,
		..
	} => Slti {
		rt: Register = Register::new(rt)?,
		rs: Register = Register::new(rs)?,
		imm: i16 = imm16.as_signed(),
	},

	/// Set less than immediate unsigned
	#[display(fmt = "sltiu {rt}, {rs}, {imm:#x}")]
	RawRepr {
		op: 0x0b,
		rt, rs, imm16,
		..
	} => Sltiu {
		rt: Register = Register::new(rt)?,
		rs: Register = Register::new(rs)?,
		imm: u16 = imm16,
	},

	/// And
	#[display(fmt = "and {rd}, {rs}, {rt}")]
	RawRepr {
		op: 0x00, op2: 0x24,
		rd, rs, rt,
		..
	} => And {
		rd: Register = Register::new(rd)?,
		rs: Register = Register::new(rs)?,
		rt: Register = Register::new(rt)?,
	},

	/// Or
	#[display(fmt = "or {rd}, {rs}, {rt}")]
	RawRepr {
		op: 0x00, op2: 0x25,
		rd, rs, rt,
		..
	} => Or {
		rd: Register = Register::new(rd)?,
		rs: Register = Register::new(rs)?,
		rt: Register = Register::new(rt)?,
	},

	/// Xor
	#[display(fmt = "xor {rd}, {rs}, {rt}")]
	RawRepr {
		op: 0x00, op2: 0x26,
		rd, rs, rt,
		..
	} => Xor {
		rd: Register = Register::new(rd)?,
		rs: Register = Register::new(rs)?,
		rt: Register = Register::new(rt)?,
	},

	/// Nor
	#[display(fmt = "nor {rd}, {rs}, {rt}")]
	RawRepr {
		op: 0x00, op2: 0x27,
		rd, rs, rt,
		..
	} => Nor {
		rd: Register = Register::new(rd)?,
		rs: Register = Register::new(rs)?,
		rt: Register = Register::new(rt)?,
	},

	/// And immediate
	#[display(fmt = "andi {rt}, {rs}, {imm:#x}")]
	RawRepr {
		op: 0x0c,
		rt, rs, imm16,
		..
	} => Andi {
		rt: Register = Register::new(rt)?,
		rs: Register = Register::new(rs)?,
		imm: u16 = imm16,
	},

	/// Or immediate
	#[display(fmt = "ori {rt}, {rs}, {imm:#x}")]
	RawRepr {
		op: 0x0d,
		rt, rs, imm16,
		..
	} => Ori {
		rt: Register = Register::new(rt)?,
		rs: Register = Register::new(rs)?,
		imm: u16 = imm16,
	},

	/// Xor immediate
	#[display(fmt = "xori {rt}, {rs}, {imm:#x}")]
	RawRepr {
		op: 0x0e,
		rt, rs, imm16,
		..
	} => Xori {
		rt: Register = Register::new(rt)?,
		rs: Register = Register::new(rs)?,
		imm: u16 = imm16,
	},

	/// Shift left logical variable
	#[display(fmt = "sllv {rd}, {rt}, {rs}")]
	RawRepr {
		op: 0x00, op2: 0x04,
		rd, rs, rt,
		..
	} => Sllv {
		rd: Register = Register::new(rd)?,
		rt: Register = Register::new(rt)?,
		rs: Register = Register::new(rs)?,
	},

	/// Shift right logical variable
	#[display(fmt = "srlv {rd}, {rt}, {rs}")]
	RawRepr {
		op: 0x00, op2: 0x06,
		rd, rs, rt,
		..
	} => Srlv {
		rd: Register = Register::new(rd)?,
		rt: Register = Register::new(rt)?,
		rs: Register = Register::new(rs)?,
	},

	/// Shift right arithmetic variable
	#[display(fmt = "srav {rd}, {rt}, {rs}")]
	RawRepr {
		op: 0x00, op2: 0x07,
		rd, rs, rt,
		..
	} => Srav {
		rd: Register = Register::new(rd)?,
		rt: Register = Register::new(rt)?,
		rs: Register = Register::new(rs)?,
	},

	/// Shift left logical
	#[display(fmt = "sll {rd}, {rt}, {imm:#x}")]
	RawRepr {
		op: 0x00, op2: 0x00,
		rd, rt, imm5,
		..
	} => Sll {
		rd: Register = Register::new(rd)?,
		rt: Register = Register::new(rt)?,
		imm: u8 = imm5,
	},

	/// Shift right logical
	#[display(fmt = "srl {rd}, {rt}, {imm:#x}")]
	RawRepr {
		op: 0x00, op2: 0x02,
		rd, rt, imm5,
		..
	} => Srl {
		rd: Register = Register::new(rd)?,
		rt: Register = Register::new(rt)?,
		imm: u8 = imm5,
	},

	/// Shift right arithmetic
	#[display(fmt = "sra {rd}, {rt}, {imm:#x}")]
	RawRepr {
		op: 0x00, op2: 0x03,
		rd, rt, imm5,
		..
	} => Sra {
		rd: Register = Register::new(rd)?,
		rt: Register = Register::new(rt)?,
		imm: u8 = imm5,
	},

	/// Load upper immediate
	#[display(fmt = "lui {rt}, {imm:#x}")]
	RawRepr {
		op: 0x0f,
		rt, imm16,
		..
	} => Lui {
		rt: Register = Register::new(rt)?,
		imm: u16 = imm16,
	},

	/// Multiply
	#[display(fmt = "mult {rs}, {rt}")]
	RawRepr {
		op: 0x00, op2: 0x18,
		rs, rt,
		..
	} => Mult {
		rs: Register = Register::new(rs)?,
		rt: Register = Register::new(rt)?,
	},

	/// Multiply unsigned
	#[display(fmt = "multu {rs}, {rt}")]
	RawRepr {
		op: 0x00, op2: 0x19,
		rs, rt,
		..
	} => Multu {
		rs: Register = Register::new(rs)?,
		rt: Register = Register::new(rt)?,
	},

	/// Divide
	#[display(fmt = "div {rs}, {rt}")]
	RawRepr {
		op: 0x00, op2: 0x1a,
		rs, rt,
		..
	} => Div {
		rs: Register = Register::new(rs)?,
		rt: Register = Register::new(rt)?,
	},

	/// Multiply unsigned
	#[display(fmt = "divu {rs}, {rt}")]
	RawRepr {
		op: 0x00, op2: 0x1b,
		rs, rt,
		..
	} => Divu {
		rs: Register = Register::new(rs)?,
		rt: Register = Register::new(rt)?,
	},

	/// Move from hi
	#[display(fmt = "mfhi {rd}")]
	RawRepr {
		op: 0x00, op2: 0x10,
		rd,
		..
	} => Mfhi {
		rd: Register = Register::new(rd)?,
	},

	/// Move from lo
	#[display(fmt = "mflo {rd}")]
	RawRepr {
		op: 0x00, op2: 0x11,
		rd,
		..
	} => Mflo {
		rd: Register = Register::new(rd)?,
	},

	/// Move to hi
	#[display(fmt = "mthi {rd}")]
	RawRepr {
		op: 0x00, op2: 0x12,
		rd,
		..
	} => Mthi {
		rd: Register = Register::new(rd)?,
	},

	/// Move to lo
	#[display(fmt = "mtlo {rd}")]
	RawRepr {
		op: 0x00, op2: 0x13,
		rd,
		..
	} => Mtlo {
		rd: Register = Register::new(rd)?,
	},

	/// Jump
	#[display(fmt = "j {target:#x}")]
	RawRepr {
		op: 0x02,
		imm26, pos,
		..
	} => J {
		target: u32 = i32::as_unsigned(u32::as_signed(pos & 0xF000_0000) + imm26.as_signed() * 4),
	},

	/// Jump and link
	#[display(fmt = "jal {target:#x}")]
	RawRepr {
		op: 0x03,
		imm26, pos,
		..
	} => Jal {
		target: u32 = i32::as_unsigned(u32::as_signed(pos & 0xF000_0000) + imm26.as_signed() * 4),
	},

	/// Jump register
	#[display(fmt = "jr {rs}")]
	RawRepr {
		op: 0x00, op2: 0x08,
		rs,
		..
	} => Jr {
		rs: Register = Register::new(rs)?,
	},

	/// Jump and link register
	#[display(fmt = "jalr {rd}, {rs}")]
	RawRepr {
		op: 0x00, op2: 0x09,
		rd, rs,
		..
	} => Jalr {
		rd: Register = Register::new(rd)?,
		rs: Register = Register::new(rs)?,
	},

	/// Branch if equal
	#[display(fmt = "beq {rs}, {rt}, {target:#x}")]
	RawRepr {
		op: 0x04,
		rs, rt, imm16, pos,
		..
	} => Beq {
		rs: Register = Register::new(rs)?,
		rt: Register = Register::new(rt)?,
		target: u32 = i32::as_unsigned(pos.as_signed() + 4 + imm16.as_signed().sign_extended::<i32>() * 4),
	},

	/// Branch if not equal
	#[display(fmt = "bne {rs}, {rt}, {target:#x}")]
	RawRepr {
		op: 0x05,
		rs, rt, imm16, pos,
		..
	} => Bne {
		rs: Register = Register::new(rs)?,
		rt: Register = Register::new(rt)?,
		target: u32 = i32::as_unsigned(pos.as_signed() + 4 + imm16.as_signed().sign_extended::<i32>() * 4),
	},

	/// Branch if less than zero
	#[display(fmt = "bltz {rs}, {target:#x}")]
	RawRepr {
		op: 0x01, rt: 0x00,
		rs, imm16, pos,
		..
	} => Bltz {
		rs: Register = Register::new(rs)?,
		target: u32 = i32::as_unsigned(pos.as_signed() + 4 + imm16.as_signed().sign_extended::<i32>() * 4)
	},

	/// Branch if greater or equal to zero
	#[display(fmt = "bgez {rs}, {target:#x}")]
	RawRepr {
		op: 0x01, rt: 0x01,
		rs, imm16, pos,
		..
	} => Bgez {
		rs: Register = Register::new(rs)?,
		target: u32 = i32::as_unsigned(pos.as_signed() + 4 + imm16.as_signed().sign_extended::<i32>() * 4)
	},

	/// Branch if greater than zero
	#[display(fmt = "bgtz {rs}, {target:#x}")]
	RawRepr {
		op: 0x07,
		rs, imm16, pos,
		..
	} => Bgtz {
		rs: Register = Register::new(rs)?,
		target: u32 = i32::as_unsigned(pos.as_signed() + 4 + imm16.as_signed().sign_extended::<i32>() * 4)
	},

	/// Branch if less or equal to zero
	#[display(fmt = "blez {rs}, {target:#x}")]
	RawRepr {
		op: 0x06,
		rs, imm16, pos,
		..
	} => Blez {
		rs: Register = Register::new(rs)?,
		target: u32 = i32::as_unsigned(pos.as_signed() + 4 + imm16.as_signed().sign_extended::<i32>() * 4)
	},

	/// Branch if less than zero and link
	#[display(fmt = "bltzal {rs}, {target:#x}")]
	RawRepr {
		op: 0x01, rt: 0x10,
		rs, imm16, pos,
		..
	} => Bltzal {
		rs: Register = Register::new(rs)?,
		target: u32 = i32::as_unsigned(pos.as_signed() + imm16.as_signed().sign_extended::<i32>() * 4)
	},

	/// Branch if greater or equal to zero and link
	#[display(fmt = "bgezal {rs}, {target:#x}")]
	RawRepr {
		op: 0x01, rt: 0x11,
		rs, imm16, pos,
		..
	} => Bgezal {
		rs: Register = Register::new(rs)?,
		target: u32 = i32::as_unsigned(pos.as_signed() + imm16.as_signed().sign_extended::<i32>() * 4)
	},

	/// Save co-processor data registers
	#[display(fmt = "mfc{n} {rt}, {rd}")]
	RawRepr {
		co_op: 0b0100, co_rs0: 0, co_rs1: 0b0000,
		co_n, rt, rd,
		..
	} => MfcN {
		n : u8       = co_n,
		rt: Register = Register::new(rt)?,
		rd: Register = Register::new(rd)?,
	},

	/// Save co-processor control registers
	#[display(fmt = "cfc{n} {rt}, {rd}")]
	RawRepr {
		co_op: 0b0100, co_rs0: 0, co_rs1: 0b0010,
		co_n, rt, rd,
		..
	} => CfcN {
		n : u8       = co_n,
		rt: Register = Register::new(rt)?,
		rd: Register = Register::new(rd)?,
	},

	/// Load co-processor data registers
	#[display(fmt = "mtc{n} {rt}, {rd}")]
	RawRepr {
		co_op: 0b0100, co_rs0: 0, co_rs1: 0b0100,
		co_n, rt, rd,
		..
	} => MtcN {
		n : u8       = co_n,
		rt: Register = Register::new(rt)?,
		rd: Register = Register::new(rd)?,
	},

	/// Load co-processor control registers
	#[display(fmt = "ctc{n} {rt}, {rd}")]
	RawRepr {
		co_op: 0b0100, co_rs0: 0, co_rs1: 0b0110,
		co_n, rt, rd,
		..
	} => CtcN {
		n : u8       = co_n,
		rt: Register = Register::new(rt)?,
		rd: Register = Register::new(rd)?,
	},

	// TODO: Check how to calculate actual targets for these jumps
	//       Docs say `$+disp`, not sure if a typo or what, no 4
	//       multiple either, are co-processor instructions 1 byte?

	/// Branch co-processor if false
	#[display(fmt = "bc{n}f {target:#x} # Raw target")]
	RawRepr {
		co_op: 0b0100, co_rs0: 0, co_rs1: 0b1000, rt: 0b00000,
		co_n, imm16,
		..
	} => BcNf {
		n: u8 = co_n,
		target: u16 = imm16,
	},

	/// Branch co-processor if true
	#[display(fmt = "bc{n}t {target:#x} # Raw target")]
	RawRepr {
		co_op: 0b0100, co_rs0: 0, co_rs1: 0b1000, rt: 0b00001,
		co_n, imm16,
		..
	} => BcNt {
		n: u8 = co_n,
		target: u16 = imm16,
	},

	/// Exec immediate co-processor
	#[display(fmt = "cop{n} {imm:#x}")]
	RawRepr {
		co_op: 0b0100, co_rs0: 1,
		co_n, imm25,
		..
	} => CopN {
		n: u8 = co_n,
		imm: u32 = imm25,
	},

	/// Load word co-processor
	#[display(fmt = "lwc{n} {rt}, {imm:#x}({rs})")]
	RawRepr {
		co_op: 0b1100,
		co_n, rs, rt, imm16,
		..
	} => LwcN {
		n: u8 = co_n,
		rs: Register = Register::new(rs)?,
		rt: Register = Register::new(rt)?,
		imm: u16 = imm16,
	},

	/// Store word co-processor
	#[display(fmt = "swc{n} {rt}, {imm:#x}({rs})")]
	RawRepr {
		co_op: 0b1110,
		co_n, rs, rt, imm16,
		..
	} => SwcN {
		n: u8 = co_n,
		rs: Register = Register::new(rs)?,
		rt: Register = Register::new(rt)?,
		imm: u16 = imm16,
	},
}
