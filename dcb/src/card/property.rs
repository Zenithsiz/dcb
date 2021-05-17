//! Card properties

// Complex
pub mod arrow_color;
pub mod attack_type;
pub mod card_type;
pub mod cross_move_effect;
pub mod digimon_property;
pub mod digivolve_effect;
pub mod effect;
pub mod effect_condition;
pub mod effect_condition_operation;
pub mod effect_operation;
pub mod level;
pub mod moves; // Note: Can't be `move`, as it's a keyword
pub mod player_type;
pub mod slot;
pub mod speciality;

// Exports
pub use arrow_color::ArrowColor;
pub use attack_type::AttackType;
pub use card_type::CardType;
pub use cross_move_effect::CrossMoveEffect;
pub use digimon_property::DigimonProperty;
pub use digivolve_effect::DigivolveEffect;
pub use effect::{Effect, MaybeEffect};
pub use effect_condition::{EffectCondition, MaybeEffectCondition};
pub use effect_condition_operation::EffectConditionOperation;
pub use effect_operation::EffectOperation;
pub use level::Level;
pub use moves::Move;
pub use player_type::PlayerType;
pub use slot::Slot;
pub use speciality::Speciality;

/// A possible [`ArrowColor`]
#[repr(transparent)]
#[derive(ref_cast::RefCast)]
#[derive(derive_more::From, derive_more::Into)]
#[derive(dcb_bytes_derive::ProxySentinel)]
#[proxy_sentinel(value = 0, wrapper_type = "ArrowColor")]
pub struct MaybeArrowColor(Option<ArrowColor>);

/// A possible [`CrossMoveEffect`]
#[repr(transparent)]
#[derive(ref_cast::RefCast)]
#[derive(derive_more::From, derive_more::Into)]
#[derive(dcb_bytes_derive::ProxySentinel)]
#[proxy_sentinel(value = 0, wrapper_type = "CrossMoveEffect")]
pub struct MaybeCrossMoveEffect(Option<CrossMoveEffect>);

/// A possible [`DigimonProperty`]
#[repr(transparent)]
#[derive(ref_cast::RefCast)]
#[derive(derive_more::From, derive_more::Into)]
#[derive(dcb_bytes_derive::ProxySentinel)]
#[proxy_sentinel(value = 0, wrapper_type = "DigimonProperty")]
pub struct MaybeDigimonProperty(Option<DigimonProperty>);
