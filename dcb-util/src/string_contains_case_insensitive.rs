//! String contains with case insensitivity

/// Helper trait for `contains_case_insensitive`
pub trait StrContainsCaseInsensitive {
	/// Checks if string `pattern` is contained in `haystack` without
	/// checking for case
	fn contains_case_insensitive(&self, pattern: &str) -> bool;
}

impl StrContainsCaseInsensitive for str {
	fn contains_case_insensitive(mut self: &Self, pattern: &str) -> bool {
		loop {
			match self.get(..pattern.len()) {
				Some(s) => match s.eq_ignore_ascii_case(pattern) {
					true => return true,
					false => self = &self[1..],
				},
				None => return false,
			}
		}
	}
}
