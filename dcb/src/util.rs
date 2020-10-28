//! Utility macros and functions
//!
//! This modules is used for miscellaneous macros, functions that have
//! not been moved to a more permanent location.
//!
//! All items in this module will eventually be deprecated and moved
//! somewhere else, but this change might take some time.

// Modules
pub mod array_split;
pub mod null_ascii_string;
#[macro_use]
pub mod impl_bytes;
pub mod merge_iter;
pub mod signed_hex;

// Exports
pub use array_split::{array_split, array_split_mut};
pub use signed_hex::SignedHex;

/// Returns the absolute different between `a` and `b`, `a - b` as a `i64`.
///
/// If the result would not fit into a `i64`, a panic occurs.
#[allow(clippy::as_conversions)] // We check every operation
#[allow(clippy::panic)] // Rust panics on failed arithmetic operations by default
pub const fn abs_diff(a: u64, b: u64) -> i64 {
	let diff = if a > b { a - b } else { b - a };

	if diff > i64::MAX as u64 {
		panic!("Overflow when computing signed distance between `u64`");
	}

	#[allow(clippy::cast_possible_wrap)] // We've verified, `diff` is less than `i64::MAX`
	if a > b {
		diff as i64
	} else {
		-(diff as i64)
	}
}

/// Adds a `i64` to a `u64`, performing `a + b`.
///
/// This function panics if the result would overflow or underflow
#[allow(clippy::as_conversions)] // We check every operation
#[allow(clippy::panic)] // Rust panics on failed arithmetic operations by default
#[allow(clippy::cast_sign_loss)] // We've verify it's positive
pub const fn signed_offset(a: u64, b: i64) -> u64 {
	// If `b` is positive, check for overflows. Else check for underflows
	if b > 0 {
		// Note: Cast is safe, as a positive `i64` fits into a `u64`.
		match a.checked_add(b as u64) {
			Some(res) => res,
			None => panic!("Overflow evaluating `u64 + i64`"),
		}
	} else {
		// Note: On `i64::MIN`, `-b` would overflow
		if b == i64::MIN || a < (-b) as u64 {
			panic!("Underflow evaluating `u64 + i64`");
		} else {
			a - ((-b) as u64)
		}
	}
}
