//! Ascii text buffer

use dcb_util::{ascii_str_arr::AsciiChar, AsciiStrArr};

/// An ascii text buffer
#[derive(PartialEq, Default, Clone, Debug, derive_more::Display)]
#[derive(ref_cast::RefCast)]
#[repr(transparent)]
pub struct AsciiTextBuffer<const N: usize>(pub AsciiStrArr<N>);


// Truncates any extra characters and ignores non-ascii
impl<const N: usize> From<String> for AsciiTextBuffer<N> {
	fn from(s: String) -> Self {
		let mut buffer = Self::default();
		for ch in s.chars() {
			match AsciiChar::from_ascii(ch) {
				Ok(ch) => match buffer.0.push(ch) {
					Ok(_) => continue,
					Err(_) => break,
				},
				Err(_) => continue,
			}
		}
		buffer
	}
}

impl<const N: usize> From<AsciiTextBuffer<N>> for String {
	fn from(buffer: AsciiTextBuffer<N>) -> Self {
		buffer.as_ref().to_owned()
	}
}

impl<const N: usize> AsRef<str> for AsciiTextBuffer<N> {
	fn as_ref(&self) -> &str {
		self.0.as_str()
	}
}

// Note: In ascii, the character index is the same
//       as the byte index.
impl<const N: usize> eframe::egui::widgets::TextBuffer for AsciiTextBuffer<N> {
	fn insert_text(&mut self, text: &str, ch_idx: usize) -> usize {
		let mut chars_inserted = 0;
		for ch in text.chars() {
			match AsciiChar::from_ascii(ch) {
				Ok(ch) => match self.0.insert(ch_idx + chars_inserted, ch) {
					Ok(_) => chars_inserted += 1,
					Err(_) => break,
				},
				Err(_) => continue,
			}
		}
		chars_inserted
	}

	fn delete_text_range(&mut self, ch_range: std::ops::Range<usize>) {
		self.0.drain_range(ch_range);
	}
}
