//! `dcb` is a library for interacting with the game file of `Digimon Digital Card Battle`.
//!
//! # Modules
//! `dcb` is split across 2 main modules, [`io`] and [`game`].
//!
//! ## Io
//! The Io module is responsible for interacting with the game file. In the future it may be responsible
//! for also interacting with the game extracted database, once work on that is complete.
//!
//! ## Game
//! The game module is responsible for representing in-game structures such as cards, sprites, text, and
//! others. The trait has various interfaces to be able to deserialize these structures from both the game
//! file, database or even other sources, depending on the structure.
//!
//! # Example
//!
//! The following is an example of how to use the `dcb` library.
//! This example extracts the card table and prints it to screen
//!
//! ```no_run
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! #   use std::fs::File;
//! let mut game_file = dcb::GameFile::from_reader( File::open("Digimon Digital Card Battle.bin")? )?;
//! let card_table = dcb::CardTable::deserialize(&mut game_file)?;
//! println!("Card table: {:?}", card_table);
//! #   Ok(())
//! # }
//! ```

// Features
#![feature(
	seek_convenience,
	never_type,
	bool_to_option,
	decl_macro,
	stmt_expr_attributes,
	unwrap_infallible,
	external_doc,
	format_args_capture,
	const_fn,
	const_panic,
	min_const_generics,
	exclusive_range_pattern,
	unsafe_block_in_unsafe_fn,
	maybe_uninit_uninit_array,
	maybe_uninit_slice,
	array_map,
	const_mut_refs,
	core_intrinsics,
	const_assume,
	bindings_after_at,
	array_value_iter,
	or_patterns,
	once_cell,
	box_syntax,
	str_split_once
)]
// Lints
#![warn(clippy::restriction, clippy::pedantic, clippy::nursery)]
// Instead of `unwrap`, we must use `expect` and provide a reason
#![forbid(clippy::unwrap_used)]
// We must use `unsafe` in unsafe `fn`s and specify if the guarantee is
// made by the caller or by us.
#![forbid(unsafe_op_in_unsafe_fn)]
// We'll disable the ones we don't need
#![allow(clippy::blanket_clippy_restriction_lints)]
// Necessary items may be inlined using `LTO`, so we don't need to mark them as inline
#![allow(clippy::missing_inline_in_public_items)]
// We prefer tail returns where possible, as they help with code readability in most cases.
#![allow(clippy::implicit_return)]
// We're fine with shadowing, as long as the variable is used for the same purpose.
// Hence why `clippy::shadow_unrelated` isn't allowed.
#![allow(clippy::shadow_reuse, clippy::shadow_same)]
// We panic when we know it won't happen, or if it does happen, then a panic is the best option
#![allow(clippy::panic, clippy::expect_used, clippy::unreachable, clippy::todo)]
// We use `expect` even in functions that return a `Result` / `Option` if there is a logic error
#![allow(clippy::unwrap_in_result)]
// We find it more important to be able to copy paste literals such as `0xabcd1234` than
// being able to read them, which does not provide many benefits
#![allow(clippy::unreadable_literal, clippy::unseparated_literal_suffix)]
// We separate implementations per their functionality usually, such as constructors, getters, setters, and others.
#![allow(clippy::multiple_inherent_impl)]
// Many operations we need to repeat, and to keep symmetry
#![allow(clippy::identity_op)]
// We only introduce items before their first usage, which sometimes is half-way through the code.
// We make sure that we only use the item after introduced, however.
#![allow(clippy::items_after_statements)]
// Useful for when they either change a lot with new variants / data,
// or for symmetry purposes
#![allow(clippy::match_same_arms)]
// In this library we have very grain-level error types, each function
// will have it's own error type ideally, so any errors are explicit
// by the type, without needing a section for them
#![allow(clippy::missing_errors_doc)]
// Although we generally try to avoid this, this can happen due to our module organization.
// In the future, this lint should be removed globally and only enabled for modules which
// actually require the use of it.
#![allow(clippy::module_inception, clippy::module_name_repetitions)]
// We use integer arithmetic and operations with the correct intent
#![allow(clippy::integer_arithmetic, clippy::integer_division)]
// We prefer using match ergonomic where possible
#![allow(clippy::pattern_type_mismatch)]
// Sometimes the blocks make it easier to invert their order
#![allow(clippy::if_not_else)]
// This lint triggers when using `assert`s and `todo`s, which is unsuitable for this project
#![allow(clippy::panic_in_result_fn)]
// A `match Option / Result / Bool` can sometimes look cleaner than a `if let / else`
#![allow(clippy::single_match_else, clippy::match_bool)]
// We're usually fine with missing future variants
#![allow(clippy::wildcard_enum_match_arm)]


// Modules
pub mod array_split;
pub mod ascii_str_arr;
pub mod null_ascii_string;
#[macro_use]
pub mod impl_bytes;
pub mod discarding_sorted_merge_iter;
pub mod signed_hex;

// Exports
//pub use array_split::{array_split, array_split_mut};
pub use ascii_str_arr::AsciiStrArr;
pub use discarding_sorted_merge_iter::DiscardingSortedMergeIter;
pub use signed_hex::SignedHex;

/// Returns the absolute different between `a` and `b`, `a - b` as a `i64`.
///
/// If the result would not fit into a `i64`, a panic occurs.
#[allow(clippy::as_conversions)] // We check every operation
#[allow(clippy::panic)] // Rust panics on failed arithmetic operations by default
#[must_use]
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
#[must_use]
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
