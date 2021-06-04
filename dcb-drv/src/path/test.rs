//! Tests

// Imports
use super::*;
#[allow(clippy::enum_glob_use)] // It's a limited scope
use Component::*;

/// Creates an ascii string from `path`
fn ascii(path: &str) -> &AsciiStr {
	AsciiStr::from_ascii(path).expect("Unable to create path")
}

/// Asserts components of `path` are `cmpts`
fn assert_components_eq(path: &Path, cmpts: &[Component]) {
	assert_eq!(path.components().collect::<Vec<_>>(), cmpts);
}

#[test]
fn simple() {
	self::assert_components_eq(Path::new(ascii("A\\B\\C")), &[
		Normal(ascii("A")),
		Normal(ascii("B")),
		Normal(ascii("C")),
	]);
}

#[test]
fn root() {
	self::assert_components_eq(Path::new(ascii("\\A")), &[Root, Normal(ascii("A"))]);
}

#[test]
fn cur() {
	self::assert_components_eq(Path::new(ascii(".\\A\\.")), &[CurDir, Normal(ascii("A")), CurDir]);
}

#[test]
fn parent() {
	self::assert_components_eq(Path::new(ascii("..\\A\\..")), &[
		ParentDir,
		Normal(ascii("A")),
		ParentDir,
	]);
}

#[test]
fn leading() {
	self::assert_components_eq(Path::new(ascii("\\\\\\\\A")), &[Root, Normal(ascii("A"))]);
}

#[test]
fn trailing() {
	self::assert_components_eq(Path::new(ascii("A\\\\\\\\")), &[Normal(ascii("A"))]);
}

#[test]
fn extra_separators() {
	self::assert_components_eq(Path::new(ascii("A\\\\\\\\B")), &[
		Normal(ascii("A")),
		Normal(ascii("B")),
	]);
}
