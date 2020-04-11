//! Input / Output
//! 
//! The Io module takes care of interacting with the game file itself, such
//! as ensuring that only the data sections in the game file are written to.
//! As well as making convertions between coordinates in data to real file
//! coordinates. (For more details, visit the [`address`] module)

// Modules
pub mod game_file;
pub mod address;

// Exports
pub use game_file::GameFile;
pub use address::Data as DataAddress;
