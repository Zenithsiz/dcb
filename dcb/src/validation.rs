//! Error and warning validation for structures

/// Structures that are validatable to be written to bytes.
///
/// This works in tandem with the [`Bytes`](dcb_bytes::Bytes) interface to allow
/// applications which take user input to validate input before serializing it.
///
/// Although this information exists by calling [`Bytes::to_bytes`](dcb_bytes::Bytes::to_bytes),
/// this interface provides two main advantages:
///
/// 1. It is faster than serializing the data, as it doesn't need to write the raw bytes and
///    can focus on simply parsing possible errors.
/// 2. It provides warnings alongside the errors. These are also provided via `log::warn`, but
///    these cannot be sent to the user easily.
// TODO: Move to `dcb-bytes`.
pub trait Validatable {
	/// Error type for this validation
	type Error;

	/// Warning type for this validation
	type Warning;

	/// Validates this structure
	fn validate(&self) -> Validation<Self::Error, Self::Warning>;
}

/// A validation
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Validation<Error, Warning> {
	/// All warnings
	warnings: Vec<Warning>,

	/// All errors
	errors: Vec<Error>,
}

impl<Error, Warning> Default for Validation<Error, Warning> {
	fn default() -> Self {
		Self {
			warnings: vec![],
			errors:   vec![],
		}
	}
}

impl<Error, Warning> Validation<Error, Warning> {
	/// Creates an empty validation
	#[must_use]
	pub fn new() -> Self {
		Self::default()
	}

	/// Emits a warning
	pub fn emit_warning(&mut self, warning: Warning) {
		self.warnings.push(warning);
	}

	/// Emits an error
	pub fn emit_error(&mut self, error: Error) {
		self.errors.push(error);
	}

	/// Returns all warnings
	#[must_use]
	pub fn warnings(&self) -> &[Warning] {
		&self.warnings
	}

	/// Returns all errors
	#[must_use]
	pub fn errors(&self) -> &[Error] {
		&self.errors
	}

	/// Returns if this validation was successful
	///
	/// A validation is considered successful if no errors occurred.
	#[must_use]
	pub fn successful(&self) -> bool {
		self.errors.is_empty()
	}
}
