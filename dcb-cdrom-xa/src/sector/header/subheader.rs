//! Sector subheader

/// The sector sub-header
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct SubHeader {
	/// File
	pub file: u16,

	/// Channel
	pub channel: u16,

	/// Submode
	pub submode: u16,

	/// Data type
	pub data_type: u16,
}

dcb_bytes::derive_bytes_split! {SubHeader,
	file     : u16 as LittleEndian,
	channel  : u16 as LittleEndian,
	submode  : u16 as LittleEndian,
	data_type: u16 as LittleEndian,
}
