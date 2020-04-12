// Crate
//--------------------------------------------------------------------------------------------------
	// Game
	use crate::game::util;
	use crate::game::Bytes;
	use crate::game::FromBytes;
	use crate::game::ToBytes;
//--------------------------------------------------------------------------------------------------

// byteorder
use byteorder::ByteOrder;
use byteorder::LittleEndian;

// Types
//--------------------------------------------------------------------------------------------------
	/// A digimon's move
	#[derive(PartialEq, Eq, Clone, Hash, Debug)]
	#[derive(serde::Serialize, serde::Deserialize)]
	pub struct Move
	{
		/// The move's name
		name: String,
		
		/// The move's power
		power: u16,
		
		/// The unknown data
		unknown: u32,
	}
	
	/// The error type thrown by `FromBytes`
	#[derive(Debug, derive_more::Display)]
	pub enum FromBytesError
	{
		/// Unable to convert name to a string
		#[display(fmt = "Unable to convert name to a string")]
		NameToString( util::ReadNullTerminatedStringError ),
	}
	
	/// The error type thrown by `ToBytes`
	#[derive(Debug, derive_more::Display)]
	pub enum ToBytesError
	{
		/// The name was too big to be written to file
		#[display(fmt = "The name \"{}\" is too long to be written to file (max is 21)", _0)]
		NameTooLong( String ),
	}
//--------------------------------------------------------------------------------------------------

// Impl
//--------------------------------------------------------------------------------------------------
	// Bytes
	impl Bytes for Move
	{
		const BUF_BYTE_SIZE : usize = 0x1c;
	}
	
	// From bytes
	impl FromBytes for Move
	{
		type Error = FromBytesError;
		
		fn from_bytes(bytes: &[u8]) -> Result<Self, Self::Error>
		{
			// And return the move
			Ok( Self {
				name   : util::read_null_terminated_string( &bytes[0x6..0x1c] ).map_err(FromBytesError::NameToString)?.to_string(),
				power  : LittleEndian::read_u16( &bytes[0x0..0x2] ),
				unknown: LittleEndian::read_u32( &bytes[0x2..0x6] ),
			})
		}
	}
	
	// To bytes
	impl ToBytes for Move
	{
		type Error = ToBytesError;
		
		fn to_bytes(&self, bytes: &mut [u8]) -> Result<(), Self::Error>
		{
			// Write the name
			bytes[0x6..0x1c].copy_from_slice( &{
				// Check if our name is too big
				if self.name.len() >= 0x16 { return Err( ToBytesError::NameTooLong( self.name.clone() ) ); }
				
				// Else make the buffer and copy everything over
				let mut buf = [0u8; 0x16];
				buf[ 0..self.name.len() ].copy_from_slice( self.name.as_bytes() );
				buf
			});
			
			// Then write the power and the unknown
			LittleEndian::write_u16(&mut bytes[0x0..0x2], self.power);
			LittleEndian::write_u32(&mut bytes[0x2..0x6], self.unknown);
			
			// And return Ok
			Ok(())
		}
	}
//--------------------------------------------------------------------------------------------------
