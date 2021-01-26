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

/*
// From http://mukoli.free.fr/mcf/mcf-cd.html
struct SubHeader
{
uint8 FileNumber; // Used for interleaving, 0 = no interleaving
uint8 AudioChannel;
uint8 EndOfRecord:1;
uint8 VideoData:1; // true if the sector contains video data
uint8 ADCPM:1; // Audio data encoded with ADPCM
uint8 Data.1; // true if sector contains data
uint8 TriggerOn:1; // OS dependent
uint8 Form2:1; // true => form 2, false => form 1
uint8 RealTime:1; // Sector contains real time data
uint8 EndOfFile:1; // true if last sector of file
uint8 Encoding; // Don't know what is this
};
*/

dcb_bytes::derive_bytes_split! {SubHeader,
	file     : u16 as LittleEndian,
	channel  : u16 as LittleEndian,
	submode  : u16 as LittleEndian,
	data_type: u16 as LittleEndian,
}
