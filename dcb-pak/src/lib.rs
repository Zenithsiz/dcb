//! `.PAK` files

// Features
#![feature(never_type, unwrap_infallible, array_chunks)]

// Modules
pub mod header;
pub mod reader;

// Exports
pub use header::Header;
pub use reader::PakFileReader;
