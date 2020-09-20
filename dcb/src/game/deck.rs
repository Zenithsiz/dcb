//! Deck

// Imports
use crate::{generate_enum_property_mod, util::impl_bytes::generate_enum_property_option};

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

generate_enum_property_option!(
	pub struct MaybeCity    (City    ) => 0,
	pub struct MaybeArmorEvo(ArmorEvo) => 0,
	pub struct MaybeMusic   (Music   ) => 0,
);

// Modules
pub mod deck;
pub mod table;

// Exports
pub use armor_evo::ArmorEvo;
pub use city::City;
pub use deck::Deck;
pub use music::Music;
pub use table::Table;
