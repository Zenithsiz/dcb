//! Next type from bytes

/// Parses some types from bytes
pub trait NextFromBytes {
	/// Parses the next `u8` from bytes
	fn next_u8(&self) -> Option<u8>;

	/// Parses the next `u16` from bytes
	fn next_u16(&self) -> Option<u16>;

	/// Parses the next `u32` from bytes
	fn next_u32(&self) -> Option<u32>;
}

impl NextFromBytes for [u8] {
	fn next_u8(&self) -> Option<u8> {
		match *self {
			[a, ..] => Some(a),
			_ => None,
		}
	}

	fn next_u16(&self) -> Option<u16> {
		match *self {
			[a, b, ..] => Some(u16::from_ne_bytes([a, b])),
			_ => None,
		}
	}

	fn next_u32(&self) -> Option<u32> {
		match *self {
			[a, b, c, d, ..] => Some(u32::from_ne_bytes([a, b, c, d])),
			_ => None,
		}
	}
}
