//! Game structure informations and serialization / deserialization

// Std
use std::{
	io::{Read, Write, Seek},
	error::Error,
};

// Crate
use crate::io::GameFile;

/// Trait that stores information about a game structure
pub trait Structure
where
	Self: Sized
{
	/// Error type for [`Self::serialize`]
	type SerializeError: Error;
	
	/// Error type for [`Self::deserialize`]
	type DeserializeError: Error;
	
	/// Returns the size of this structure
	fn size() -> (usize, Option<usize>);
	
	/// Attempts to deserialize this data structure from a game file
	fn deserialize<R: Read + Write + Seek>(file: &mut GameFile<R>) -> Result<Self, Self::DeserializeError>;
	
	/// Attempts to serialize the structure to the game file
	fn serialize<R: Read + Write + Seek>(&self, file: &mut GameFile<R>) -> Result<(), Self::SerializeError>;
}
