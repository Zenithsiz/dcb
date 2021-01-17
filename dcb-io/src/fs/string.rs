//! Filesystem strings

/// Modules
pub mod arr;
pub mod error;
pub mod owned;
pub mod slice;

// Exports
pub use arr::StrArrAlphabet;
pub use error::{InvalidCharError, ValidateFileAlphabetError};
pub use owned::StringAlphabet;
pub use slice::StrAlphabet;


/// An alphabet for a string
pub trait Alphabet {
	/// Error type
	type Error;

	/// Returns if `bytes` are valid for this alphabet
	fn validate(bytes: &[u8]) -> Result<(), Self::Error>;
}


/// Implements the [`Alphabet`] trait from an alphabet
pub trait ImplFromAlphabet {
	/// The alphabet
	fn alphabet() -> &'static [u8];

	/// String terminator
	fn terminator() -> u8;
}

impl<A: ImplFromAlphabet> Alphabet for A {
	type Error = InvalidCharError;

	fn validate(bytes: &[u8]) -> Result<(), Self::Error> {
		// If any are invalid, return Err
		for (pos, &byte) in bytes.iter().enumerate() {
			// If we found the terminator, terminate
			// TODO: Maybe make sure everything after the `;` is valid too
			if byte == Self::terminator() {
				break;
			}

			// Else make sure it contains this byte
			if !Self::alphabet().contains(&byte) {
				return Err(InvalidCharError { byte, pos });
			}
		}

		Ok(())
	}
}

/// A-type alphabet
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct AlphabetA;

impl ImplFromAlphabet for AlphabetA {
	fn alphabet() -> &'static [u8] {
		&[
			b'A', b'B', b'C', b'D', b'E', b'F', b'G', b'H', b'I', b'J', b'K', b'L', b'M', b'N', b'O', b'P', b'Q', b'R', b'S', b'T', b'U', b'V', b'W',
			b'X', b'Y', b'Z', b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'_', b'!', b'"', b'%', b'&', b'\'', b'(', b')', b'*',
			b'+', b',', b'-', b'.', b'/', b':', b';', b'<', b'=', b'>', b'?',
		]
	}

	fn terminator() -> u8 {
		b' '
	}
}

/// D-type alphabet
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct AlphabetD;

impl ImplFromAlphabet for AlphabetD {
	fn alphabet() -> &'static [u8] {
		&[
			b'A', b'B', b'C', b'D', b'E', b'F', b'G', b'H', b'I', b'J', b'K', b'L', b'M', b'N', b'O', b'P', b'Q', b'R', b'S', b'T', b'U', b'V', b'W',
			b'X', b'Y', b'Z', b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'_',
		]
	}

	fn terminator() -> u8 {
		b' '
	}
}

/// File alphabet
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct FileAlphabet;

impl Alphabet for FileAlphabet {
	type Error = ValidateFileAlphabetError;

	fn validate(bytes: &[u8]) -> Result<(), Self::Error> {
		// Separate into `<name>.<extension>;<version>`
		let dot_idx = bytes.iter().position(|&b| b == b'.').ok_or(ValidateFileAlphabetError::MissingExtension)?;
		let version_idx = bytes.iter().position(|&b| b == b';').ok_or(ValidateFileAlphabetError::MissingVersion)?;
		let (name, bytes) = bytes.split_at(dot_idx);
		let (extension, version) = bytes.split_at(version_idx);

		// Validate all separately
		AlphabetD::validate(name).map_err(ValidateFileAlphabetError::InvalidNameChar)?;
		AlphabetD::validate(extension).map_err(ValidateFileAlphabetError::InvalidExtensionChar)?;
		match version {
			[b'0'..=b'9'] => Ok(()),
			_ => Err(ValidateFileAlphabetError::InvalidVersion),
		}
	}
}

/// A-type string array
pub type StrArrA<const N: usize> = StrArrAlphabet<AlphabetA, N>;

/// A-type string
pub type StringA = StringAlphabet<AlphabetA>;

/// A-type string slice
pub type StrA = StrAlphabet<AlphabetA>;

/// D-type string array
pub type StrArrD<const N: usize> = StrArrAlphabet<AlphabetD, N>;

/// D-type string
pub type StringD = StringAlphabet<AlphabetD>;

/// D-type string slice
pub type StrD = StrAlphabet<AlphabetD>;

/// File string array
pub type FileStrArr<const N: usize> = StrArrAlphabet<FileAlphabet, N>;

/// File string
pub type FileString = StringAlphabet<FileAlphabet>;

/// File string slice
pub type FileStr = StrAlphabet<FileAlphabet>;
