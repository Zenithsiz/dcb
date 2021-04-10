//! Errors

/// Error for [`GameFile::new`](super::GameFile::new)
#[derive(Debug, thiserror::Error)]
pub enum NewError {
	/// Unable to parse filesystem
	#[error("Unable to parse filesystem")]
	ParseFilesystem(#[source] dcb_iso9660::NewError),
}

/// Error for [`GameFile::read_drv`](super::GameFile::read_drv)
#[derive(Debug, thiserror::Error)]
pub enum ReadDrvError {
	/// Unable to read filesystem root
	#[error("Unable to read filesystem root")]
	ReadRoot(#[source] dcb_iso9660::entry::ReadDirError),

	/// Unable to find file
	#[error("Unable to find file")]
	FindFile,

	/// Unable to read file
	#[error("Unable to read file")]
	ReadFile(#[source] dcb_iso9660::entry::ReadFileError),
}
