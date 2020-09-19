//! Tests

// Imports
use std::mem::MaybeUninit;

use super::AsciiStrArr;
use ascii::{AsciiChar, AsciiStr};

#[test]
pub fn new() {
	const N: usize = 10;
	let mut ascii = AsciiStrArr::<N>::new();
	assert_eq!(ascii, AsciiStrArr::default());

	// SAFETY: We're initializing all elements
	let mut cur_len = 0;
	for c in unsafe { ascii.buffer_mut() } {
		*c = MaybeUninit::new(AsciiChar::Null);
	}

	loop {
		assert_eq!(ascii.len(), cur_len);
		assert_eq!(ascii.is_empty(), cur_len == 0);
		assert_eq!(ascii.as_ascii().len(), cur_len);
		assert_eq!(ascii.as_ascii_mut().len(), cur_len);
		assert_eq!(ascii.as_ascii_slice().len(), cur_len);
		assert_eq!(ascii.as_ascii_slice_mut().len(), cur_len);
		assert_eq!(ascii.as_bytes().len(), cur_len);
		assert_eq!(ascii.as_str().len(), cur_len);
		assert_eq!(ascii.get(0).copied(), if cur_len == 0 { None } else { Some(AsciiChar::Null) });
		assert_eq!(ascii.get_mut(0).copied(), if cur_len == 0 { None } else { Some(AsciiChar::Null) });
		assert_eq!(AsRef::<[AsciiChar]>::as_ref(&ascii).len(), cur_len);
		assert_eq!(AsMut::<[AsciiChar]>::as_mut(&mut ascii).len(), cur_len);
		assert_eq!(AsRef::<AsciiStr>::as_ref(&ascii).len(), cur_len);
		assert_eq!(AsMut::<AsciiStr>::as_mut(&mut ascii).len(), cur_len);
		assert_eq!(AsRef::<[u8]>::as_ref(&ascii).len(), cur_len);
		assert_eq!(AsRef::<str>::as_ref(&ascii).len(), cur_len);
		assert_eq!(ascii, ascii.clone());
		assert_eq!(format!("{}", ascii), format!("{}", ascii.as_ascii()));
		assert_eq!(format!("{:?}", ascii), format!("{:?}", ascii.as_ascii()));

		if cur_len < N {
			cur_len += 1;
			// SAFETY: All elements are initialized at the beginning and `cur_len <= N`
			unsafe {
				ascii.set_len(cur_len);
			}
		} else {
			break;
		}
	}
}
