//! Errors

// Imports
use super::fs;

/// Error for [`Bytes::new`](super::Bytes::new)
#[derive(Debug, thiserror::Error)]
pub enum NewError {
	/// Unable to read filesystem
	#[error("Unable to read Iso9660 filesystem")]
	NewIso9660FileSystem(#[source] dcb_iso9660::fs::NewError),

	/// Unable to read filesystem root
	#[error("Unable to read Iso9660 filesystem root")]
	Iso9660FilesystemRootReadEntries(#[source] dcb_iso9660::fs::entry::ReadEntriesError),

	/// Unable to find filesystem file
	#[error("Unable to find Iso9660 filesystem file 'A.DRV'")]
	Iso9660FilesystemFindFileA,

	/// Unable to read filesystem file
	#[error("Unable to read Iso9660 filesystem file 'A.DRV'")]
	Iso9660FilesystemReadFileA(#[source] dcb_iso9660::fs::entry::ReadError),

	/// Unable to read parse filesystem
	#[error("Unable to read filesystem file 'A.DRV'")]
	ParseFilesystemA(#[source] fs::FromBytesError),

	/// Unable to find filesystem file
	#[error("Unable to find Iso9660 filesystem file 'B.DRV'")]
	Iso9660FilesystemFindFileB,

	/// Unable to read filesystem file
	#[error("Unable to read Iso9660 filesystem file 'B.DRV'")]
	Iso9660FilesystemReadFileB(#[source] dcb_iso9660::fs::entry::ReadError),

	/// Unable to read parse filesystem
	#[error("Unable to read filesystem file 'B.DRV'")]
	ParseFilesystemB(#[source] fs::FromBytesError),

	/// Unable to find filesystem file
	#[error("Unable to find Iso9660 filesystem file 'C.DRV'")]
	Iso9660FilesystemFindFileC,

	/// Unable to read filesystem file
	#[error("Unable to read Iso9660 filesystem file 'C.DRV'")]
	Iso9660FilesystemReadFileC(#[source] dcb_iso9660::fs::entry::ReadError),

	/// Unable to read parse filesystem
	#[error("Unable to read filesystem file 'C.DRV'")]
	ParseFilesystemC(#[source] fs::FromBytesError),

	/// Unable to find filesystem file
	#[error("Unable to find Iso9660 filesystem file 'E.DRV'")]
	Iso9660FilesystemFindFileE,

	/// Unable to read filesystem file
	#[error("Unable to read Iso9660 filesystem file 'E.DRV'")]
	Iso9660FilesystemReadFileE(#[source] dcb_iso9660::fs::entry::ReadError),

	/// Unable to read parse filesystem
	#[error("Unable to read filesystem file 'E.DRV'")]
	ParseFilesystemE(#[source] fs::FromBytesError),

	/// Unable to find filesystem file
	#[error("Unable to find Iso9660 filesystem file 'F.DRV'")]
	Iso9660FilesystemFindFileF,

	/// Unable to read filesystem file
	#[error("Unable to read Iso9660 filesystem file 'F.DRV'")]
	Iso9660FilesystemReadFileF(#[source] dcb_iso9660::fs::entry::ReadError),

	/// Unable to read parse filesystem
	#[error("Unable to read filesystem file 'F.DRV'")]
	ParseFilesystemF(#[source] fs::FromBytesError),

	/// Unable to find filesystem file
	#[error("Unable to find Iso9660 filesystem file 'G.DRV'")]
	Iso9660FilesystemFindFileG,

	/// Unable to read filesystem file
	#[error("Unable to read Iso9660 filesystem file 'G.DRV'")]
	Iso9660FilesystemReadFileG(#[source] dcb_iso9660::fs::entry::ReadError),

	/// Unable to read parse filesystem
	#[error("Unable to read filesystem file 'G.DRV'")]
	ParseFilesystemG(#[source] fs::FromBytesError),

	/// Unable to find filesystem file
	#[error("Unable to find Iso9660 filesystem file 'P.DRV'")]
	Iso9660FilesystemFindFileP,

	/// Unable to read filesystem file
	#[error("Unable to read Iso9660 filesystem file 'P.DRV'")]
	Iso9660FilesystemReadFileP(#[source] dcb_iso9660::fs::entry::ReadError),

	/// Unable to read parse filesystem
	#[error("Unable to read filesystem file 'G.DRV'")]
	ParseFilesystemP(#[source] fs::FromBytesError),
}
