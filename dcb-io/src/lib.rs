//! `dcb` io
//!
//! This crate is responsible for interactions with the game file itself,
//! including the custom filesystem used by the game with the `.DRV` files.

// Features
#![feature(
	stmt_expr_attributes,
	format_args_capture,
	exclusive_range_pattern,
	never_type,
	unwrap_infallible,
	array_methods,
	array_chunks,
	iter_map_while,
	external_doc
)]
// Lints
#![warn(clippy::restriction, clippy::pedantic, clippy::nursery)]
// We'll disable the ones we don't need
#![allow(clippy::blanket_clippy_restriction_lints)]
// No unsafe allowed in this crate
#![forbid(unsafe_code)]
// Must use `expect` instead of `unwrap`
#![forbid(clippy::unwrap_used)]
// We don't need to mark every public function `inline`
#![allow(clippy::missing_inline_in_public_items)]
// We prefer literals to be copy-paste-able rather than readable
#![allow(clippy::unreadable_literal)]
// We prefer suffixes to be glued to the literal
#![allow(clippy::unseparated_literal_suffix)]
// We're fine with panicking when entering an unexpected state
#![allow(
	clippy::panic,
	clippy::unreachable,
	clippy::expect_used,
	clippy::panic_in_result_fn,
	clippy::unwrap_in_result,
	clippy::indexing_slicing,
	clippy::todo
)]
// We prefer tail calls
#![allow(clippy::implicit_return)]
// We use multiple implementations to separate logic
#![allow(clippy::multiple_inherent_impl)]
// We use granular error types, usually one for each function, which document the
// errors that might happen, as opposed to documenting them in the function
#![allow(clippy::missing_errors_doc)]
// Due to our module organization, we end up with data types inheriting their module's name
#![allow(clippy::module_name_repetitions)]
// We need arithmetic for this crate
#![allow(clippy::integer_arithmetic, clippy::integer_division)]
// We want to benefit from match ergonomics where possible
#![allow(clippy::pattern_type_mismatch)]
// We only use wildcards when we only care about certain variants
#![allow(clippy::wildcard_enum_match_arm, clippy::match_wildcard_for_single_variants)]
// We're fine with shadowing, as long as it's related
#![allow(clippy::shadow_reuse, clippy::shadow_same)]
// Matching on booleans can look better than `if / else`
#![allow(clippy::match_bool)]
// If the `else` isn't needed, we don't put it
#![allow(clippy::else_if_without_else)]
// We're fine with non-exhaustive structs / enums, we aren't committing to them yet.
#![allow(clippy::exhaustive_structs, clippy::exhaustive_enums)]
// There are too many false positives with these lints
#![allow(clippy::use_self)]
// `Header` and `Reader` are common names
#![allow(clippy::similar_names)]
// We only use `# Panics` where a panic might be caused by a mis-use of the user, not assertions
#![allow(clippy::missing_panics_doc)]

// Modules
pub mod game_file;
pub mod tim;

// Exports
pub use game_file::GameFile;
