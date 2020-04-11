//! Addressing modes of the game file
//! 
//! The game file, as explained in `GameFile`, is separated
//! into real addresses, which correspond to actual file
//! offsets, and data addresses, which correspond to offsets
//! inside the data section of each sector.

// Modules
pub mod real;
pub mod data;

// Exports
pub use real::Real;
pub use data::Data;

/// Error type for `TryFrom<Real> for Data`
#[derive(Debug, derive_more::Display)]
pub enum RealToDataError
{
	/// Occurs when the Real is outside of the data section of the sector
	#[display(fmt = "The real address {} could not be converted to a data address as it is not in the data section", _0)]
	OutsideDataSection(Real),
}

impl std::error::Error for RealToDataError {
	fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
		match self {
			Self::OutsideDataSection(..) => None,
		}
	}
}

// Real -> Data
impl std::convert::TryFrom<Real> for Data
{
	type Error = RealToDataError;
	
	fn try_from(real_address: Real) -> Result<Data, Self::Error>
	{
		// If the real address isn't in the data section, then return err
		if !real_address.in_data_section() { return Err( Self::Error::OutsideDataSection(real_address) ); }
		
		// Else get the sector and offset
		let real_sector        = real_address.sector();
		let real_sector_offset = real_address.offset();
		
		// The data address is just converting the real_sector
		// to a data_sector and subtracting the header from the
		// real offset to get the data offset 
		Ok( Data::from(
			Real::SECTOR_BYTE_SIZE * real_sector +      // Base of data sector
			real_sector_offset - Real::HEADER_BYTE_SIZE // Data offset
		))
	}
}

// Data -> Real
impl From<Data> for Real
{
	fn from(data_address: Data) -> Real
	{
		// Get the sector and offset
		let data_sector        = data_address.sector();
		let data_sector_offset = data_address.offset();
		
		// Then the real address is just convering the data_sector
		// to a real_sector and adding the header plus the offset
		Real::from(
			Real::SECTOR_BYTE_SIZE * data_sector + // Base of real sector
			Real::HEADER_BYTE_SIZE               + // Skip header
			data_sector_offset                     // Offset inside data sector
		)
	}
}
