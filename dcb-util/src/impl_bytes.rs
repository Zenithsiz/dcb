//! Helper macros to implement [`Bytes`](dcb_bytes::Bytes)

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

				impl $enum_name {
					/// All variants
					pub const ALL: &'static [Self] = &[
						$(
							<$enum_name>::$enum_variant_name,
						)*
					];

					/// Returns a string representing this
					pub fn as_str(self) -> &'static str {
						match self {
							$(
								<$enum_name>::$enum_variant_name => $enum_variant_rename,
							)+
						}
					}
				}

				// Extra definitions
				$( $extra_defs )*
			}
		)*
	}
}
