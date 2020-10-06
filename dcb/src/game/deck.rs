//! Deck

// Imports
use crate::generate_enum_property_mod;

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

/// A [`City`] that may exist or not with a [`Bytes`] implementation
#[repr(transparent)]
#[derive(ref_cast::RefCast)]
#[derive(derive_more::From, derive_more::Into)]
#[derive(dcb_bytes_derive::ProxySentinel)]
#[proxy_sentinel(value = 0, wrapper_type = "City")]
pub struct MaybeCity(Option<City>);

/// An [`ArmorEvo`] that may exist or not with a [`Bytes`] implementation
#[repr(transparent)]
#[derive(ref_cast::RefCast)]
#[derive(derive_more::From, derive_more::Into)]
#[derive(dcb_bytes_derive::ProxySentinel)]
#[proxy_sentinel(value = 0, wrapper_type = "ArmorEvo")]
pub struct MaybeArmorEvo(Option<ArmorEvo>);

/// A [`Music`] that may exist or not with a [`Bytes`] implementation
#[repr(transparent)]
#[derive(ref_cast::RefCast)]
#[derive(derive_more::From, derive_more::Into)]
#[derive(dcb_bytes_derive::ProxySentinel)]
#[proxy_sentinel(value = 0, wrapper_type = "Music")]
pub struct MaybeMusic(Option<Music>);

// Modules
pub mod deck;
pub mod table;

// Exports
pub use armor_evo::ArmorEvo;
pub use city::City;
pub use deck::Deck;
pub use music::Music;
pub use table::Table;
