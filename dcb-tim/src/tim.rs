//! Tim

// Modules
pub mod error;

// Exports
pub use error::DeserializeError;

// Imports
use crate::{Clut, Img};

/// `tim` file
#[derive(PartialEq, Clone, Debug)]
pub struct Tim {
	/// Clut
	pub clut: Option<Clut>,

	/// Image
	pub img: Img,
}
