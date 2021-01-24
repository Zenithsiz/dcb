//! A `.PAK` entry

// Modules
pub mod animation2d;
pub mod model3d_set;

// Exports
pub use animation2d::Animation2d;
pub use model3d_set::Model3dSet;

// Imports
use super::Header;

/// A `.PAK` entry
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct PakEntry {
	/// Header
	header: Header,

	/// Position
	pos: u64,
}

impl PakEntry {
	/// Creates a pak entry from it's header and position
	#[must_use]
	pub const fn new(header: Header, pos: u64) -> Self {
		Self { header, pos }
	}

	/// Returns this entry's header
	#[must_use]
	pub const fn header(&self) -> &Header {
		&self.header
	}

	/// Returns this entry's position
	#[must_use]
	pub const fn pos(&self) -> u64 {
		self.pos
	}
}
