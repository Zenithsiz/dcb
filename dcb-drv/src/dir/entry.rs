#![doc(include = "entry.md")]

// Modules
pub mod reader;
pub mod writer;

// Exports
pub use reader::{DirEntryReader, DirEntryReaderKind};
pub use writer::{DirEntryWriter, DirEntryWriterKind};
