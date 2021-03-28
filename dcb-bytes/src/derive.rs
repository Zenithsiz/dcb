//! Derives for [`Bytes`](super::Bytes)

/// Derives `Bytes` by splitting the input bytes and parsing them with `BYTEORDER`
#[macro_export]
macro_rules! derive_bytes_split {
	($T:ty, $($field:ident : $U:ty as $BYTEORDER:ident),* $(,)?) => {
		const _: () = {
			use $crate::{
				ByteOrderExt,
				ByteArray,
				arrayref,
			};

			#[allow(clippy::ptr_offset_with_cast)] // `arrayref` does it
			impl $crate::Bytes for $T {
				type ByteArray = [u8; {0 $( + <<$U as ByteOrderExt<$crate::byteorder::$BYTEORDER>>::ByteArray as ByteArray>::SIZE )*}];

				type FromError = !;
				type ToError   = !;

				fn from_bytes(bytes: &Self::ByteArray) -> Result<Self, Self::FromError> {
					let ( $($field,)* ) = arrayref::array_refs![
						bytes,
						$( <<$U as ByteOrderExt<$crate::byteorder::$BYTEORDER>>::ByteArray as ByteArray>::SIZE ),*
					];

					Ok(Self {
						$(
							$field: <$U as ByteOrderExt::<$crate::byteorder::$BYTEORDER>>::read( $field ).into(),
						)*
					})
				}

				fn to_bytes(&self, bytes: &mut Self::ByteArray) -> Result<(), Self::ToError> {
					let ( $($field,)* ) = arrayref::mut_array_refs![
						bytes,
						$( <<$U as ByteOrderExt<$crate::byteorder::$BYTEORDER>>::ByteArray as ByteArray>::SIZE ),*
					];

					$(
						<$U as ByteOrderExt::<$crate::byteorder::$BYTEORDER>>::write(&self.$field.into(), $field);
					)*

					Ok(())
				}
			}
		};
	};
}
