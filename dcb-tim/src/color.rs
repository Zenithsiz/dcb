//! Clut color

// Imports
use int_conv::Split;

/// Clut color
#[derive(PartialEq, Clone, Copy, Debug)]
pub struct Color {
	/// Red
	pub r: u8,

	/// Green
	pub g: u8,

	/// Blue
	pub b: u8,

	/// Stp
	pub stp: bool,
}

impl Color {
	/// Parses a color from a `u16`
	#[bitmatch::bitmatch]
	#[allow(clippy::as_conversions, clippy::cast_possible_truncation)] // We want to truncate
	#[must_use]
	pub const fn from_16bit(value: u16) -> Self {
		// Get rgb
		#[bitmatch]
		let "a_bbbbb_ggggg_rrrrr" = value;

		// Scale them up from `0..31` to `0..255`
		let r = (r << 0x3u16) as u8;
		let g = (g << 0x3u16) as u8;
		let b = (b << 0x3u16) as u8;

		Self { r, g, b, stp: a == 1 }
	}

	/// Parses two colors from a `[u16; 3]`
	#[bitmatch::bitmatch]
	#[allow(clippy::as_conversions, clippy::cast_possible_truncation)] // We want to truncate
	#[must_use]
	pub fn from_24bit(value: [u16; 3]) -> [Self; 2] {
		// Get rgb
		let [(r1, g1), (b1, r2), (g2, b2)] = value.map(Split::lo_hi);

		[
			Self {
				r:   r1,
				g:   g1,
				b:   b1,
				stp: false,
			},
			Self {
				r:   r2,
				g:   g2,
				b:   b2,
				stp: false,
			},
		]
	}

	/// Converts this color to a `[u8; 4]` rgba
	#[must_use]
	pub const fn to_rgba(self) -> [u8; 4] {
		[self.r, self.g, self.b, if self.stp { 255 } else { 0 }]
	}
}
