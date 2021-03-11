//! Signed hexadecimal formatting
// TODO: Improve this module overall.

// Imports
use int_conv::Extended;
use ref_cast::RefCast;
use std::fmt;

/// A signed numeric type that uses signed hexadecimal formatting.
#[derive(ref_cast::RefCast)]
#[repr(transparent)]
pub struct SignedHex<T>(pub T);

// All references implement it for their underlying type.
#[allow(clippy::use_self)] // We're using a generic version `SignedHex`, not `Self`
impl<'a, T> fmt::Display for SignedHex<&'a T>
where
	SignedHex<T>: fmt::Display,
{
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		<SignedHex<T> as fmt::Display>::fmt(SignedHex::<T>::ref_cast(self.0), f)
	}
}

/// Macro to help implement [`SignedHex`]
macro_rules! impl_signed_hex {
	($($T:ty => $TBigger:ty),* $(,)?) => {
		$(
		impl fmt::Display for SignedHex<$T> {
			#[allow(clippy::default_numeric_fallback)] // We want inference to take care of the `0` here
			fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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
				fmt::LowerHex::fmt(&self.0.extended::<$TBigger>().abs(), f)
			}
		}
	)*
}
}

impl_signed_hex! {
	i8  => i16,
	i16 => i32,
	i32 => i64,
	i64 => i128,
}
