//! Raw instruction representation

// Imports
use super::Raw;
use int_conv::Truncate;

/// An instruction's raw representation, including
/// it's current address.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[allow(clippy::missing_docs_in_private_items)]
pub struct RawRepr {
	pub op:    u8,
	pub rs:    u8,
	pub rt:    u8,
	pub rd:    u8,
	pub imm5:  u8,
	pub op2:   u8,
	pub imm16: u16,
	pub imm25: u32,
	pub imm26: u32,

	/// Co-processor opcode
	pub co_op: u8,

	/// Co-processor number
	pub co_n: u8,

	/// Co-processor highest `rs` bit.
	pub co_rs0: u8,

	/// Co-processor lowest `rs` bits.
	pub co_rs1: u8,

	/// Position of the instruction
	pub pos: u32,
}

#[allow(clippy::inconsistent_digit_grouping)] // We're grouping 6-5-5-5-5-6 as per docs.
impl RawRepr {
	/// Creates a new split instruction
	#[must_use]
	#[rustfmt::skip]
	pub fn new(Raw {repr, pos}: Raw) -> Self {
		Self {
			op    : ((repr & 0b111111_00000_00000_00000_00000_000000) >> 26).truncate(),
			rs    : ((repr & 0b000000_11111_00000_00000_00000_000000) >> 21).truncate(),
			rt    : ((repr & 0b000000_00000_11111_00000_00000_000000) >> 16).truncate(),
			rd    : ((repr & 0b000000_00000_00000_11111_00000_000000) >> 11).truncate(),
			imm5  : ((repr & 0b000000_00000_00000_00000_11111_000000) >> 6 ).truncate(),
			op2   : ((repr & 0b000000_00000_00000_00000_00000_111111) >> 0 ).truncate(),
			imm16 : ((repr & 0b000000_00000_00000_11111_11111_111111) >> 0 ).truncate(),
			imm25 : ((repr & 0b000000_01111_11111_11111_11111_111111) >> 0 ),
			imm26 : ((repr & 0b000000_11111_11111_11111_11111_111111) >> 0 ),
			co_op : ((repr & 0b111100_00000_00000_00000_00000_000000) >> 28).truncate(),
			co_rs0: ((repr & 0b000000_10000_00000_00000_00000_000000) >> 25).truncate(),
			co_rs1: ((repr & 0b000000_01111_00000_00000_00000_000000) >> 21).truncate(),
			co_n  : ((repr & 0b000011_00000_00000_00000_00000_000000) >> 26).truncate(),
			pos: pos.0,
		}
	}
}
