//! Storage for game data info.

// Traits
//--------------------------------------------------------------------------------------------------
	/// Trait that stores information about a data structure of the game.
	pub trait Bytes : Sized
	{
		/// The buffer size this type needs to work with.
		const BUF_BYTE_SIZE : usize;
	}
//--------------------------------------------------------------------------------------------------
