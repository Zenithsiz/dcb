//! Digimon property

/// A digimon's property
#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(strum::IntoStaticStr, strum::Display, strum::EnumIter)]
#[derive(dcb_bytes_derive::Discriminant)]
pub enum DigimonProperty {
	/// Own speciality
	#[strum(serialize = "Own speciality")]
	OwnSpeciality     = 1,

	/// Opponent speciality
	#[strum(serialize = "Opponent speciality")]
	OpnSpeciality     = 2,

	/// Own HP
	#[strum(serialize = "Own HP")]
	OwnHP             = 3,

	/// Opponent HP
	#[strum(serialize = "Opponent HP")]
	OpnHP             = 4,

	/// Own circle attack
	#[strum(serialize = "Own circle attack")]
	OwnCircleAttack   = 5,

	/// Opponent circle attack
	#[strum(serialize = "Opponent circle attack")]
	OpnCircleAttack   = 6,

	/// Own triangle attack
	#[strum(serialize = "Own triangle attack")]
	OwnTriangleAttack = 7,

	/// Opponent triangle attack
	#[strum(serialize = "Opponent triangle attack")]
	OpnTriangleAttack = 8,

	/// Own cross attack
	#[strum(serialize = "Own cross attack")]
	OwnCrossAttack    = 9,

	/// Opponent cross attack
	#[strum(serialize = "Opponent cross attack")]
	OpnCrossAttack    = 10,

	/// Own attack
	#[strum(serialize = "Own attack")]
	OwnAttack         = 11,

	/// Opponent attack
	#[strum(serialize = "Opponent attack")]
	OpnAttack         = 12,

	/// Own level
	#[strum(serialize = "Own level")]
	OwnLevel          = 13,

	/// Opponent level
	#[strum(serialize = "Opponent level")]
	OpnLevel          = 14,

	/// Own attack type
	#[strum(serialize = "Own attack type")]
	OwnAttackType     = 17,

	/// Opponent attack type
	#[strum(serialize = "Opponent attack type")]
	OpnAttackType     = 18,

	/// Attack order
	#[strum(serialize = "Attack order")]
	AttackOrder       = 20,

	/// Cards in own hand
	#[strum(serialize = "Cards in own hand")]
	CardsInOwnHand    = 21,

	/// Cards in opponent hand
	#[strum(serialize = "Cards in opponent hand")]
	CardsInOpnHand    = 22,

	/// Cards in own dp slot
	#[strum(serialize = "Cards in own dp slot")]
	CardsInOwnDpSlot  = 23,

	/// Cards in opponent dp slot
	#[strum(serialize = "Cards in opponent dp slot")]
	CardsInOpnDpSlot  = 24,

	/// Cards in own offline deck
	#[strum(serialize = "Cards in own offline deck")]
	CardsInOwnOffDeck = 25,

	/// Temp slot
	#[strum(serialize = "Temp slot")]
	TempSlot          = 26,

	/// Cards in own online deck
	#[strum(serialize = "Cards in own online deck")]
	CardsInOwnOnDeck  = 27,

	/// Cards in opponent online deck
	#[strum(serialize = "Cards in opponent online deck")]
	CardsInOpnOnDeck  = 28,
}

impl DigimonProperty {
	/// Returns a string representing this
	#[must_use]
	pub fn as_str(self) -> &'static str {
		self.into()
	}
}
