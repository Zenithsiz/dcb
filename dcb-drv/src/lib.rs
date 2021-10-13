#![doc = include_str!("lib.md")]
// Features
#![feature(
	seek_stream_len,
	try_blocks,
	associated_type_bounds,
	never_type,
	unwrap_infallible,
	format_args_capture,
	str_internals,
	destructuring_assignment
)]

// Modules
pub mod dir;
pub mod entry;
pub mod path;
pub mod ptr;
pub mod swap;
pub mod writer;

// Exports
pub use entry::{DirEntry, DirEntryKind};
pub use path::{Path, PathBuf};
pub use ptr::{DirEntryPtr, DirPtr, FilePtr};
pub use swap::swap_files;
pub use writer::{DirEntryWriter, DirEntryWriterKind, DirWriter, DirWriterLister};
