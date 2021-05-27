//! Boxing + Mapping

// Imports
use crate::ResultFamily;

/// Boxes a variant of `Result` and maps it
pub trait MapBoxResult: ResultFamily {
	/// Boxes the `Ok` variant
	fn box_map<T, F: FnOnce(Box<Self::Ok>) -> T>(self, f: F) -> Result<T, Self::Err> {
		self.into().map(Box::new).map(f)
	}

	/// Maps and boxes the `Err` variant
	fn box_map_err<E, F: FnOnce(Box<Self::Err>) -> E>(self, f: F) -> Result<Self::Ok, E> {
		self.into().map_err(Box::new).map_err(f)
	}
}

impl<T, E> MapBoxResult for Result<T, E> {}
