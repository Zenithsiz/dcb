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

// Exports
pub use array_split::{array_split, array_split_mut};
