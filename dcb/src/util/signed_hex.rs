//! Signed hexadecimal formatting
// TODO: Improve this module overall.

// Imports
use int_conv::Extended;
use std::fmt::{self, Formatter, LowerHex};

/// A signed numeric type that uses signed hexadecimal formatting.
pub struct SignedHex<T>(pub T);

// All references implement it for their underlying type.
impl<'a, T: LowerHex> LowerHex for SignedHex<&'a T> {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		<T as LowerHex>::fmt(self.0, f)
	}
}

/// Macro to help implement [`SignedHex`]
macro impl_signed_hex($($T:ty => $TBigger:ty),* $(,)?) {
	$(
		impl LowerHex for SignedHex<$T> {
			fn fmt(&self, f: &mut Formatter) -> fmt::Result {
				let sign = match (self.0 < 0, f.sign_plus()) {
					(true, _) => "-",
					(false, true) => "+",
					_ => "",
				};
				f.write_str(sign)?;

				if f.sign_plus() {
					todo!("Signed hex does not support + flag yet");
				};

				// TODO: Remove `+` from the formatter flags when we do
				//       this to fully support the `+` flag.
				LowerHex::fmt(&self.0.extended::<$TBigger>().abs(), f)
			}
		}
	)*
}

impl_signed_hex! {
	i8  => i16,
	i16 => i32,
	i32 => i64,
	i64 => i128,
}
