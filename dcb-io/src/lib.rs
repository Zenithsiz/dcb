//! `dcb` io
//!
//! This crate is responsible for interactions with the game file itself,
//! including the custom filesystem used by the game with the `.DRV` files.

// Features
#![feature()]

// Modules
pub mod game_file;
pub mod tim;

// Exports
pub use game_file::GameFile;
