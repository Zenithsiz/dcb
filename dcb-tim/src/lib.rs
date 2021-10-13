#![doc = include_str!("lib.md")]
// Features
#![feature(never_type, unwrap_infallible, array_chunks)]

// Modules
pub mod bpp;
pub mod clut;
pub mod color;
pub mod header;
pub mod img;
pub mod tim;
pub mod tis;

// Exports
pub use bpp::BitsPerPixel;
pub use clut::Clut;
pub use color::Color;
pub use header::Header;
pub use img::Img;
pub use tim::Tim;
pub use tis::Tis;
