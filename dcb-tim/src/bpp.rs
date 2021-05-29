//! Bits per pixel

/// Bits per pixel
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum BitsPerPixel {
	/// 4-bit indexed
	Index4Bit,

	/// 8-bit indexed
	Index8Bit,

	/// 16-bit color
	Color16Bit,

	/// 24-bit color
	Color24Bit,
}

impl BitsPerPixel {
	/// Returns if this bpp is indexed
	#[must_use]
	pub const fn is_indexed(self) -> bool {
		matches!(self, Self::Index4Bit | Self::Index8Bit)
	}

	/// Scales the image size as per the bpp
	#[must_use]
	pub fn scale_size(self, size: [u16; 2]) -> [u16; 2] {
		let [width, height] = size;
		let width = match self {
			Self::Index4Bit => width * 4,
			Self::Index8Bit => width * 2,
			Self::Color16Bit => width,
			Self::Color24Bit => todo!(),
		};
		[width, height]
	}
}
