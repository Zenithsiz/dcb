//! Error and warning validation for structures

/// Structures that are validatable to be written to bytes.
///
/// This works in tandem with the [`Bytes`](crate::Bytes) interface to allow
/// applications which take user input to validate input before serializing it.
///
/// Although this information exists by calling [`Bytes::to_bytes`](crate::Bytes::to_bytes),
/// this interface provides two main advantages:
///
/// 1. It is faster than serializing the data, as it doesn't need to write the raw bytes and
///    can focus on simply parsing possible errors.
/// 2. It provides warnings alongside the errors. These are also provided via `log::warn`, but
///    these cannot be sent to the user easily.
pub trait Validatable {
	/// Validation type
	type Output: Validation;

	/// Validates this structure
	fn validate(&self) -> Self::Output;
}

/// A validation type.
///
/// This is the output of structures which may be validated.
/// It is a trait to offer more flexibility to each structure to report
/// errors and warnings in it's preferred manner.
pub trait Validation: Clone {
	/// Warnings type
	type Warnings;

	/// Errors type
	type Errors;

	/// If this validation was successful.
	///
	/// A successful validation is one that, although may emit warnings, did not emit
	/// any errors. Conversely, this also indicates that calling [`to_bytes`] will _not_
	/// produce a `Err` value.
	fn successful(&self) -> bool {
		self.errors().is_none()
	}

	/// Returns any warnings
	fn warnings(&self) -> Option<Self::Warnings>;

	/// Returns any errors
	fn errors(&self) -> Option<Self::Errors>;
}
