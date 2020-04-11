//! File real addresses

/// A type for defining addresses on the `.bin` file.
/// 
/// # Details
/// All addresses of type `Real` will represent the *real* position
/// on the file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Real(u64);

// Constants
impl Real
{
	/// The number of bytes within a whole sector
	pub const SECTOR_BYTE_SIZE: u64 = 2352;
	
	/// The number of bytes the data section takes up in the sector
	pub const DATA_BYTE_SIZE: u64 = 2048;
	
	/// The number of bytes the header takes up in the sector
	pub const HEADER_BYTE_SIZE: u64 = 24;
	
	/// The number of bytes the footer takes up in the sector
	pub const FOOTER_BYTE_SIZE: u64 = 280;
	
	/// The start of the data section
	pub const DATA_START: u64 = Self::HEADER_BYTE_SIZE;
	
	/// The end of the data section (one-past)
	pub const DATA_END: u64 = Self::HEADER_BYTE_SIZE + Self::DATA_BYTE_SIZE;
	
	/// The range of the data section
	pub const DATA_RANGE: std::ops::Range<u64> = Self::DATA_START .. Self::DATA_END;
}

impl Real
{
	/// Returns the real sector associated with this address
	pub fn sector(self) -> u64 {
		u64::from(self) / Self::SECTOR_BYTE_SIZE
	}
	
	/// Returns the real offset into the sector of this address
	pub fn offset(self) -> u64 {
		u64::from(self) % Self::SECTOR_BYTE_SIZE
	}
	
	/// Returns the real end address of the data section
	pub fn data_section_end(self) -> Self {
		// Get the sector
		let real_sector = self.sector();
		
		// The end of the real data section is after the header and data sections
		Self::from(
			Real::SECTOR_BYTE_SIZE * real_sector + // Beginning of sector
			Real::HEADER_BYTE_SIZE               + // Skip Header
			Real::  DATA_BYTE_SIZE                 // Skip Data
		)
	}
	
	/// Checks if a real address lies within the data section
	pub fn in_data_section(self) -> bool {
		// If our offset is within the data range
		Self::DATA_RANGE.contains( &self.offset() )
	}
}

// Conversions from and into u64
impl From<Real> for u64  { fn from(address: Real) -> u64  { address.0     } }
impl From<u64 > for Real { fn from(address: u64 ) -> Real { Real(address) } }

// Conversions from and into i64
impl From<Real> for i64  { fn from(address: Real) -> i64  {  u64::from(address       ) as i64 } }
impl From<i64 > for Real { fn from(address: i64 ) -> Real { Real::from(address as u64)        } }

// Operations
//--------------------------------------------------------------------------------------------------
	// Real + Offset
	impl std::ops::Add<i64> for Real
	{
		type Output = Real;
		
		fn add(self, offset: i64) -> Real
		{
			Self::from( i64::from(self) + offset )
		}
	}
	
	// Real += Offset
	impl std::ops::AddAssign<i64> for Real
	{
		fn add_assign(&mut self, offset: i64) { *self = *self + offset; }
	}
	
	// Real - Real
	impl std::ops::Sub<Real> for Real
	{
		type Output = i64;
		
		fn sub(self, address: Real) -> i64
		{
			i64::from(self) - i64::from(address)
		}
	}
//--------------------------------------------------------------------------------------------------


// Display
impl std::fmt::Display for Real
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
	{
		write!(f, "{:x}", u64::from(*self))
	}
}
