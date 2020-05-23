//! Error and warning validation for [`Bytes`](crate::Bytes) structures

// Std
use std::borrow::Cow;


/// Validation for [`bytes::validate`]
#[derive(Debug, PartialEq, Clone)]
pub struct Validation<'a> {
	/// If the validation was successful.
	///
	/// If this is `false`, it is strongly encouraged for `warnings` or
	/// `errors` to have something to explain why it wasn't successful.
	success: bool,

	/// All warnings emitted.
	///
	/// Warnings must not be fatal. `self.to_bytes()` must succeed if only
	/// warnings are emitted.
	warnings: Vec<Cow<'a, str>>,

	/// All errors emitted.
	///
	/// Errors are fatal by default, `self.to_bytes()` should fail if any errors
	/// are emitted.
	errors: Vec<Cow<'a, str>>,
}

impl<'a> Default for Validation<'a> {
	fn default() -> Self {
		Self::new()
	}
}

// Constructors
impl<'a> Validation<'a> {
	/// Create an empty successful validation, with no warnings or errors
	#[must_use]
	pub const fn new() -> Self {
		Self {
			success:  true,
			warnings: vec![],
			errors:   vec![],
		}
	}
}

// Adders
impl<'a> Validation<'a> {
	/// Adds a new warning to this validation.
	pub fn add_warning(&mut self, warning: impl Into<Cow<'a, str>>) {
		self.warnings.push(warning.into());
	}

	/// Adds a new error to this validation.
	///
	/// This also turns the validation unsuccessful.
	pub fn add_error(&mut self, error: impl Into<Cow<'a, str>>) {
		self.errors.push(error.into());
		self.success = false;
	}
}

// Getters
impl<'a> Validation<'a> {
	/// Returns if this validation was successful
	#[must_use]
	pub const fn successful(&self) -> bool {
		self.success
	}

	/// Returns all warnings
	#[must_use]
	pub fn warnings(&self) -> &[impl AsRef<str> + 'a] {
		&self.warnings
	}

	/// Returns all errors
	#[must_use]
	pub fn errors(&self) -> &[impl AsRef<str> + 'a] {
		&self.errors
	}
}
