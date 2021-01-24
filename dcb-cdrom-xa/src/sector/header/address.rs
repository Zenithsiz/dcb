//! Sector address

/// Sector address
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Address {
	/// Minutes
	pub min: u8,

	/// Seconds
	pub sec: u8,

	/// Block
	pub block: u8,
}

dcb_bytes::derive_bytes_split! {Address,
	min  : u8 as LittleEndian,
	sec  : u8 as LittleEndian,
	block: u8 as LittleEndian,
}
