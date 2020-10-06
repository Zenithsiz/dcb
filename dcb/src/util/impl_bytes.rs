//! Helper macros to implement [`Bytes`](crate::Bytes)

/// Defines and implements a property enum
// TODO: Make better documentation
// TODO: Turn into a `macro` once they work
#[macro_export]
macro_rules! generate_enum_property_mod
{
	// Entry point
	(
		// The modules
		$(
			// Module
			$mod_vis:vis mod $mod_name:ident
			{
				// Enum attributes
				$( #[$enum_attr:meta] )*

				// Enum
				enum $enum_name:ident
				{
					// Enum variants
					$(
						// Attributes
						$( #[$enum_variant_attr:meta] )*

						// Variant
						// Note: Must have no data
						$enum_variant_name:ident

						// `Display` conversion name
						($enum_variant_rename:literal)

						=>

						// Variant value
						$enum_variant_value:literal,
					)+

					// Extra fields for `Bytes::from_bytes`.
					$(
						$from_bytes_value:literal => $from_bytes_body:tt,
					)*

					// Error
					_ => $error_unknown_value_display:literal

					$(,)?
				}

				// Any further definitions inside the module
				$( $extra_defs:tt )*
			}
		)*
	) =>
	{
		// Modules
		$(
			// The module
			$mod_vis mod $mod_name
			{
				// The property enum
				$( #[$enum_attr] )*
				#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
				#[derive(::serde::Serialize, ::serde::Deserialize)]
				#[derive(::derive_more::Display)]
				pub enum $enum_name
				{
					$(
						$( #[$enum_variant_attr] )*
						#[serde(rename = $enum_variant_rename)]
						#[display(fmt = $enum_variant_rename)]
						$enum_variant_name = $enum_variant_value,
					)+
				}

				/// Error type for [`::dcb_bytes::Bytes::from_bytes`]
				#[derive(PartialEq, Eq, Clone, Copy, ::std::fmt::Debug, ::thiserror::Error)]
				pub enum FromBytesError {

					/// Unknown value
					#[error($error_unknown_value_display, byte)]
					UnknownValue {
						byte: u8,
					}
				}

				impl ::dcb_bytes::Bytes for $enum_name
				{
					type ByteArray = u8;

					type FromError = FromBytesError;
					fn from_bytes(byte: &Self::ByteArray) -> Result<Self, Self::FromError>
					{
						match byte {
							$(
								$enum_variant_value =>
								Ok( <$enum_name>::$enum_variant_name ),
							)+

							$(
								$from_bytes_value => {
									Ok( { $from_bytes_body } )
								}
							)*

							&byte => Err( Self::FromError::UnknownValue{ byte } ),
						}
					}

					type ToError = !;
					#[allow(unreachable_code, unused_variables)] // For when there are multiple values
					fn to_bytes(&self, byte: &mut Self::ByteArray) -> Result<(), Self::ToError>
					{
						*byte = match self {
							$(
								<$enum_name>::$enum_variant_name => $enum_variant_value,
							)+
						};

						Ok(())
					}
				}

				// Extra definitions
				$( $extra_defs )*
			}
		)*
	}
}

/// Implements [`Bytes`](dcb_bytes::Bytes) for `Option<E>` where `E`
/// is the first argument of this macro and an enum.
///
/// This is done by supplying a sentinel value which is read/written as `None`.
pub macro generate_enum_property_option (
	$(
		#[derive(BytesProxySentinel)]
		#[bytes_proxy_sentinel(value = $sentinel_value:literal)]
		$struct_vis:vis struct $struct_name:ident ( $enum_vis:vis Option<$enum_name:ty> );
	)*
) {
	$(
		#[repr(transparent)]
		#[derive(::ref_cast::RefCast)]
		#[derive(::derive_more::From, ::derive_more::Into)]
		$struct_vis struct $struct_name( $enum_vis Option<$enum_name> );

		#[allow(clippy::diverging_sub_expression)] // Errors might be `!`
		impl ::dcb_bytes::Bytes for $struct_name {
			type ByteArray = <$enum_name as ::dcb_bytes::Bytes>::ByteArray;

			type FromError = <$enum_name as ::dcb_bytes::Bytes>::FromError;
			fn from_bytes(bytes: &Self::ByteArray) -> Result<Self, Self::FromError>
			{
				match bytes {
					$sentinel_value => Ok( Self(None) ),
					_               => Ok( Self(Some( ::dcb_bytes::Bytes::from_bytes(bytes)? )) ),
				}
			}

			type ToError = <$enum_name as ::dcb_bytes::Bytes>::ToError;
			fn to_bytes(&self, bytes: &mut Self::ByteArray) -> Result<(), Self::ToError>
			{
				match &self.0 {
					Some(value) => ::dcb_bytes::Bytes::to_bytes(value, bytes)?,
					None        => *bytes = $sentinel_value,
				}

				Ok(())
			}
		}
	)*
}
