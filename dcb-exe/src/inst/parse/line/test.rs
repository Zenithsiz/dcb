//! Tests

// Imports
use super::*;

#[test]
fn test_parse_literal() {
	// Oks
	assert_matches!(self::parse_literal("0"), Ok((0, "")));
	assert_matches!(self::parse_literal("123"), Ok((123, "")));
	assert_matches!(self::parse_literal("123abc"), Ok((123, "abc")));
	assert_matches!(self::parse_literal("+1"), Ok((1, "")));
	assert_matches!(self::parse_literal("-1"), Ok((-1, "")));
	assert_matches!(self::parse_literal("0x100"), Ok((0x100, "")));
	assert_matches!(self::parse_literal("0b100"), Ok((0b100, "")));
	assert_matches!(self::parse_literal("0o100"), Ok((0o100, "")));
	assert_matches!(self::parse_literal("-0x100"), Ok((-0x100, "")));
	assert_matches!(self::parse_literal("-0b100"), Ok((-0b100, "")));
	assert_matches!(self::parse_literal("-0o100"), Ok((-0o100, "")));
	assert_matches!(self::parse_literal("0x123abc"), Ok((0x123abc, "")));
	assert_matches!(self::parse_literal("0b123abc"), Ok((0b1, "23abc")));
	assert_matches!(self::parse_literal("0o123abc"), Ok((0o123, "abc")));

	// Errors
	assert_matches!(self::parse_literal(""), Err(ParseLiteralError::Parse(_)));
	assert_matches!(self::parse_literal("abc"), Err(ParseLiteralError::Parse(_)));
	assert_matches!(self::parse_literal("0xg"), Err(ParseLiteralError::Parse(_)));
	assert_matches!(self::parse_literal("0b2"), Err(ParseLiteralError::Parse(_)));
	assert_matches!(self::parse_literal("0o8"), Err(ParseLiteralError::Parse(_)));
}

#[test]
fn test_parse_string() {
	// Oks
	assert_eq!(
		self::parse_string(r#""Hello,\n World""#),
		Ok(("Hello,\n World".to_owned(), ""))
	);
}

#[test]
#[should_panic]
fn parse_string_panic() {
	match self::parse_string("a") {
		Ok(_) | Err(_) => (),
	}
}
