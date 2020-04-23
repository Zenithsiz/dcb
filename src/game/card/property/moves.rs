//! A digimon's move
//! 
//! This module contains the [`Move`] struct, which describes a generic move.
//! 
//! # Layout
//! Each move has a size of `0x1c` bytes, and it's layout is the following:
//! 
//! | Offset | Size | Type                 | Name                      | Location               | Details                           |
//! |--------|------|----------------------|---------------------------|------------------------|-----------------------------------|
//! | 0x0    | 0x2  | `u16`                | Power                     | `power`                |                                   |
//! | 0x2    | 0x4  | `u32`                | Unknown                   | `unknown`              | Most likely stores animation data |
//! | 0x4    | 0x16 | `[char; 0x16]`       | Name                      | `name`                 | Null-terminated                   |

// byteorder
use byteorder::{ByteOrder, LittleEndian};

// Crate
use crate::game::{util, Bytes};

/// A digimon's move
#[derive(PartialEq, Eq, Clone, Hash, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Move
{
	/// The move's name
	name: arrayvec::ArrayVec<[ascii::AsciiChar; 21]>,
	
	/// The move's power
	power: u16,
	
	/// The unknown data
	unknown: u32,
}

/// Error type for [`Bytes::FromBytes`]
#[derive(Debug, derive_more::Display, err_impl::Error)]
pub enum FromBytesError
{
	/// Unable to read the move name
	#[display(fmt = "Unable to read the move name")]
	Name( #[error(source)] util::ReadNullAsciiStringError ),
}

/// Error type for [`Bytes::ToBytes`]
#[derive(Debug, derive_more::Display, err_impl::Error)]
pub enum ToBytesError
{
	/// The name was too big to be written to file
	#[display(fmt = "The name \"{}\" is too long to be written to file (max is 21)", _0)]
	NameTooLong( String ),
}

// Bytes
impl Bytes for Move
{
	type ByteArray = [u8; 0x1c];
	
	type FromError = FromBytesError;
	fn from_bytes(bytes: &Self::ByteArray) -> Result<Self, Self::FromError>
	{
		// And return the move
		Ok( Self {
			name   : util::read_null_ascii_string( &bytes[0x0..0x15] )
				.map_err(FromBytesError::Name)?
				.chars().collect(),
			power  : LittleEndian::read_u16( &bytes[0x0..0x2] ),
			unknown: LittleEndian::read_u32( &bytes[0x2..0x6] ),
		})
	}
	
	type ToError = !;
	fn to_bytes(&self, bytes: &mut Self::ByteArray) -> Result<(), Self::ToError>
	{
		// Get all byte arrays we need
		util::array_split_mut!(bytes,
			0x0..0x02 => power,
			0x2..0x04 => unknown,
			0x4..0x1c => name,
		);
		
		// Write the name
		name.copy_from_slice(
			// Note: `self.name` is at most [char; 21], this cannot fail
			util::write_null_ascii_string(self.name.as_ref().as_ref(), &mut [0u8; 22])
				.expect("Name was too large for output buffer")
		);
		
		// Then write the power and the unknown
		LittleEndian::write_u16(power  , self.power  );
		LittleEndian::write_u32(unknown, self.unknown);
		
		// And return Ok
		Ok(())
	}
}
