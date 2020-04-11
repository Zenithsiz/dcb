//! File data-only addresses

// Address
use crate::io::address::Real;

// Types
//--------------------------------------------------------------------------------------------------
	/// A type for defining addresses on the data parts of `.bin` file.
	/// 
	/// # Details
	/// All addresses of type `Data` will represent the position
	/// within *only* the data sections on the file.
	#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
	pub struct Data(u64);
//--------------------------------------------------------------------------------------------------

// Impl
//--------------------------------------------------------------------------------------------------
	impl Data
	{
		// Constructors
		//--------------------------------------------------------------------------------------------------
			/// Constructs a data address from it's representation in u64
			/// 
			/// # Note
			/// `address` is not a real address, but a data address represented in `u64`
			pub const fn from_u64(address: u64) -> Self {
				Self( address )
			}
		//--------------------------------------------------------------------------------------------------
		
		// Conversions
		//--------------------------------------------------------------------------------------------------
			/// Returns the sector associated with this address
			pub fn sector(self) -> u64
			{
				u64::from(self) / Real::DATA_BYTE_SIZE
			}
			
			/// Returns the offset into the data section of this address
			pub fn offset(self) -> u64
			{
				u64::from(self) % Real::DATA_BYTE_SIZE
			}
		//--------------------------------------------------------------------------------------------------
	}
	
	// Conversions from and into u64
	impl From<Data> for u64  { fn from(address: Data) -> u64  { address.0     } }
	impl From<u64 > for Data { fn from(address: u64 ) -> Data { Data(address) } }
	
	// Operations
	//--------------------------------------------------------------------------------------------------
		// Data + Offset
		impl std::ops::Add<i64> for Data
		{
			type Output = Data;
			
			fn add(self, offset: i64) -> Data
			{
				if offset > 0 {
					self + (offset as u64)
				} else {
					self - (-offset as u64)
				}
			}
		}
		
		// Data += Offset
		impl std::ops::AddAssign<i64> for Data
		{
			fn add_assign(&mut self, offset: i64) { *self = *self + offset; }
		}
		
		// Data + absolute
		impl std::ops::Add<u64> for Data
		{
			type Output = Data;
			
			fn add(self, absolute: u64) -> Data {
				Self::from( self.0 + absolute )
			}
		}
		
		// Data += absolute
		impl std::ops::AddAssign<u64> for Data
		{
			fn add_assign(&mut self, absolute: u64) { *self = *self + absolute; }
		}
		
		// Data - absolute
		impl std::ops::Sub<u64> for Data
		{
			type Output = Data;
			
			fn sub(self, absolute: u64) -> Data {
				Self::from( self.0 - absolute )
			}
		}
		
		// Data -= absolute
		impl std::ops::SubAssign<u64> for Data
		{
			fn sub_assign(&mut self, absolute: u64) { *self = *self - absolute; }
		}
		
		// Data - Data
		impl std::ops::Sub<Data> for Data
		{
			type Output = i64;
			
			fn sub(self, address: Data) -> i64
			{
				// TODO: Do this another way?
				   self.0 as i64 -
				address.0 as i64
			}
		}
	//--------------------------------------------------------------------------------------------------
	
	// Display
	impl std::fmt::Display for Data
	{
		fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
		{
			write!(f, "{:x}", u64::from(*self))
		}
	}
//--------------------------------------------------------------------------------------------------
