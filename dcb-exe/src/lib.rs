//! `dcb` executable decompilation and recompilation.
//!
//! This crate aims at being able to decompile the original binary into
//! `mips` assembly, which may then be modified and compiled back into the binary.
//!
//! The decompiled assembly will have annotations for each function and data, both
//! manual, loaded from a `resources/known_*.yaml` and heuristically found.

// Features
#![feature(
	//unsafe_block_in_unsafe_fn,
	format_args_capture,
	never_type,
	or_patterns,
	associated_type_bounds,
	bindings_after_at
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
// We're fine with panicking when entering an unexpected state or on unfinished code.
// TODO: Remove `clippy::todo` once everything is finished.
#![allow(
	clippy::panic,
	clippy::unreachable,
	clippy::expect_used,
	clippy::todo,
	clippy::panic_in_result_fn,
	clippy::unwrap_in_result
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
#![allow(clippy::integer_arithmetic)]
// We want to benefit from match ergonomics where possible
#![allow(clippy::pattern_type_mismatch)]
// We only use wildcards when we only care about certain variants
#![allow(clippy::wildcard_enum_match_arm, clippy::match_wildcard_for_single_variants)]
// We're fine with shadowing, as long as it's related
#![allow(clippy::shadow_reuse)]

// Modules
pub mod exe;

// Exports
pub use exe::{Exe, Header, Pos};
