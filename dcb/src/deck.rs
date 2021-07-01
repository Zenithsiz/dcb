//! Deck


/// Defines and implements a property enum
// TODO: Make better documentation
// TODO: Turn into a `macro` once they work
// TODO: Remove
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
					#[must_use]
					pub const fn as_str(self) -> &'static str {
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

generate_enum_property_mod! {
	pub mod city {
		/// A deck city
		enum City
		{
			Starter      ("Starter"       ) => 32,
			Fire         ("Fire"          ) => 33,
			Jungle       ("Jungle"        ) => 34,
			Ice          ("Ice"           ) => 35,
			Junk         ("Junk"          ) => 36,
			Dark         ("Dark"          ) => 37,
			Pyramid      ("Pyramid"       ) => 38,
			Desert       ("Desert"        ) => 39,
			Cloud        ("Cloud"         ) => 40,
			Road         ("Road"          ) => 41,
			WisemanTower ("Wiseman Tower" ) => 42,
			InfinityTower("Infinity Tower") => 43,

			_ => "Unknown byte {:#x} for a city"
		}
	}

	pub mod armor_evo {
		/// An armor evolution
		enum ArmorEvo
		{
			First ("First" ) => 1,
			Second("Second") => 2,
			Third ("Third" ) => 3,

			_ => "Unknown byte {:#x} for an armor evolution"
		}
	}

	pub mod music {
		/// Music
		enum Music
		{
			BattleProtag ("Battle Protag" ) => 46,
			BattleWorm   ("Battle Worm"   ) => 47,
			BattleBasic  ("Battle Basic"  ) => 143,
			BattleVillain("Battle Villain") => 144,

			PolygonProtag ("Polygon Protag" ) => 37,
			PolygonWorm   ("Polygon Worm"   ) => 44,
			PolygonBasic  ("Polygon Basic"  ) => 147,
			PolygonVillain("Polygon Villain") => 148,

			_ => "Unknown byte {:#x} for a music"
		}
	}
}

/// A possible [`City`]
#[repr(transparent)]
#[derive(ref_cast::RefCast)]
#[derive(derive_more::From, derive_more::Into)]
#[derive(dcb_bytes_derive::ProxySentinel)]
#[proxy_sentinel(value = 0, wrapper_type = "City")]
pub struct MaybeCity(Option<City>);

/// A possible [`ArmorEvo`]
#[repr(transparent)]
#[derive(ref_cast::RefCast)]
#[derive(derive_more::From, derive_more::Into)]
#[derive(dcb_bytes_derive::ProxySentinel)]
#[proxy_sentinel(value = 0, wrapper_type = "ArmorEvo")]
pub struct MaybeArmorEvo(Option<ArmorEvo>);

/// A possible [`Music`]
#[repr(transparent)]
#[derive(ref_cast::RefCast)]
#[derive(derive_more::From, derive_more::Into)]
#[derive(dcb_bytes_derive::ProxySentinel)]
#[proxy_sentinel(value = 0, wrapper_type = "Music")]
pub struct MaybeMusic(Option<Music>);

// Modules
#[allow(clippy::module_inception)] // TODO: Fix
pub mod deck;
pub mod table;

// Exports
pub use armor_evo::ArmorEvo;
pub use city::City;
pub use deck::Deck;
pub use music::Music;
pub use table::Table;
