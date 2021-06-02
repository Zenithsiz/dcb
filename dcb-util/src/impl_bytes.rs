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

					// Extra fields for `Bytes::deserialize_bytes`.
					$(
						$deserialize_bytes_value:literal => $deserialize_bytes_body:tt,
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

				/// Error type for [`::dcb_bytes::Bytes::deserialize_bytes`]
				#[derive(PartialEq, Eq, Clone, Copy, ::std::fmt::Debug, ::thiserror::Error)]
				pub enum DeserializeBytesError {

					/// Unknown value
					#[error($error_unknown_value_display, byte)]
					UnknownValue {
						byte: u8,
					}
				}

				impl ::dcb_bytes::Bytes for $enum_name
				{
					type ByteArray = u8;

					type DeserializeError = DeserializeBytesError;
					fn deserialize_bytes(byte: &Self::ByteArray) -> Result<Self, Self::DeserializeError>
					{
						match byte {
							$(
								$enum_variant_value =>
								Ok( <$enum_name>::$enum_variant_name ),
							)+

							$(
								$deserialize_bytes_value => {
									Ok( { $deserialize_bytes_body } )
								}
							)*

							&byte => Err( Self::DeserializeError::UnknownValue{ byte } ),
						}
					}

					type SerializeError = !;
					#[allow(unreachable_code, unused_variables)] // For when there are multiple values
					fn serialize_bytes(&self, byte: &mut Self::ByteArray) -> Result<(), Self::SerializeError>
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
