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

/// A string alphabet
///
/// This type serves to create marker types for strings that may only
/// contain a subset of characters, or must have them in a certain order.
///
/// This is accomplished by the [`validate`](Alphabet::validate) method,
/// which simply checks if a byte slice is valid for this alphabet.
pub trait Alphabet {
	/// Error type
	type Error;

	/// Validates `bytes` for a string of this alphabet and returns
	/// it, possibly without it's terminator.
	fn validate(bytes: &[u8]) -> Result<&[u8], Self::Error>;
}

/// Implements the [`Alphabet`] trait from a list of valid characters
/// and a possible terminator
pub trait OnlyValidCharsAlphabet {
	/// All valid characters
	fn valid_chars() -> &'static [u8];

	/// Terminator for the string.
	fn terminator() -> u8;
}

impl<A: OnlyValidCharsAlphabet> Alphabet for A {
	type Error = InvalidCharError;

	fn validate(bytes: &[u8]) -> Result<&[u8], Self::Error> {
		// Go through all bytes and validate them until end of
		// string or terminator.
		let terminator = Self::terminator();
		for (pos, &byte) in bytes.iter().enumerate() {
			// If we found the terminator, terminate
			if byte == terminator {
				return Ok(&bytes[..pos]);
			}

			// Else make sure it contains this byte
			if !Self::valid_chars().contains(&byte) {
				return Err(InvalidCharError { byte, pos });
			}
		}

		// If we got, there was no terminator, which is still a valid string.
		Ok(bytes)
	}
}

/// A-character alphabet
///
/// The list of valid characters are `A..Z`, `0..9`, `_`, `!`, `"`, `%`, `'`, `(`, `)`, `*`, `+`,
/// `+`, `,`, `-`, `.`, `/`, `:`, `;`, `<`, `=`, `>` and `?`.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct AlphabetA;

impl OnlyValidCharsAlphabet for AlphabetA {
	fn valid_chars() -> &'static [u8] {
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

/// D-character alphabet
///
/// The list of valid characters are `A..Z`, `0..9` and `_`
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct AlphabetD;

impl OnlyValidCharsAlphabet for AlphabetD {
	fn valid_chars() -> &'static [u8] {
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
///
/// The file alphabet dictates the format for file names,
/// which must follow `<name>.<extension>;<version>`, where
/// `<name>` and `<extension>` are D-character strings,
/// and `<version>` only contains numeric decimal characters.
///
/// There are 3 exceptions to this, which are the root directory
/// name, current directory name and parent directory name, which
/// are, "\0", "" and "\x01", respectively.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct FileAlphabet;

impl Alphabet for FileAlphabet {
	type Error = ValidateFileAlphabetError;

	fn validate(bytes: &[u8]) -> Result<&[u8], Self::Error> {
		// Special cases for the root, `.` and `..`, respectively
		// TODO: Remove exceptions from this string and make directories store the
		//       current and parent separately.
		if let [b'\0'] | [] | [b'\x01'] = bytes {
			return Ok(bytes);
		}

		// Separate into `<name>.<extension>;<version>`
		let (name, extension, version) = {
			// Separate into `<name>.<rest>` and ignore the `.` in `<rest>`
			let dot_idx = bytes.iter().position(|&b| b == b'.').ok_or(ValidateFileAlphabetError::MissingExtension)?;
			let (name, rest) = bytes.split_at(dot_idx);
			let rest = &rest[1..];

			// Then split at `<extension>;<version>` and ignore the `;`
			let version_idx = rest.iter().position(|&b| b == b';').ok_or(ValidateFileAlphabetError::MissingVersion)?;
			let (extension, version) = rest.split_at(version_idx);
			let version = &version[1..];

			(name, extension, version)
		};

		// Validate all separately
		AlphabetD::validate(name).map_err(ValidateFileAlphabetError::InvalidNameChar)?;
		AlphabetD::validate(extension).map_err(ValidateFileAlphabetError::InvalidExtensionChar)?;
		match version.iter().all(|ch| (b'0'..=b'9').contains(ch)) {
			true => Ok(bytes),
			false => Err(ValidateFileAlphabetError::InvalidVersion),
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
