//! Addressing modes of the game file
//!
//! The game file, as explained in `GameFile`, is separated
//! into real addresses, which correspond to actual file
//! offsets, and data addresses, which correspond to offsets
//! inside the data section of each sector.

// Modules
pub mod data;
pub mod real;

// Exports
pub use data::Data;
pub use real::Real;
