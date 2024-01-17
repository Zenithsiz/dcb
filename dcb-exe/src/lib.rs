//! `dcb` executable decompilation and recompilation.
//!
//! This crate aims at being able to decompile the original binary into
//! `mips` assembly, which may then be modified and compiled back into the binary.
//!
//! The decompiled assembly will have annotations for each function and data, both
//! manual, loaded from a `resources/known_*.yaml` and heuristically found.

// Features
#![feature(
	format_args_capture,
	never_type,
	array_chunks,
	const_btree_new,
	unwrap_infallible,
	type_alias_impl_trait,
	assert_matches,
	extend_one,
	exclusive_range_pattern,
	label_break_value,
	impl_trait_in_assoc_type
)]

// Modules
pub mod data;
pub mod func;
pub mod header;
pub mod inst;
pub mod pos;
pub mod reader;

// Exports
pub use data::{Data, DataTable, DataType};
pub use func::{Func, FuncTable};
pub use header::Header;
pub use pos::Pos;
pub use reader::ExeReader;
