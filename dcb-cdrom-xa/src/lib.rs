#![doc = include_str!("lib.md")]
// Features
#![feature(never_type, unwrap_infallible, exclusive_range_pattern, assert_matches)]

// Modules
pub mod cursor;
pub mod reader;
pub mod sector;
pub mod writer;

// Exports
pub use cursor::CdRomCursor;
pub use reader::CdRomReader;
pub use sector::Sector;
pub use writer::CdRomWriter;
