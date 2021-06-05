//! Alerts

use std::{borrow::Cow, fmt};

// Imports
use native_dialog::{MessageDialog, MessageType};

/// Alerts a message
fn alert(ty: MessageType, msg: fmt::Arguments) {
	// Get the string to display without allocating if possible
	let msg = msg.as_str().map_or_else(|| Cow::Owned(msg.to_string()), Cow::Borrowed);

	MessageDialog::new()
		.set_text(&*msg)
		.set_type(ty)
		.show_alert()
		.expect("Unable to alert user");
}

/// Confirms a message
fn confirm(ty: MessageType, msg: fmt::Arguments) -> bool {
	// Get the string to display without allocating if possible
	let msg = msg.as_str().map_or_else(|| Cow::Owned(msg.to_string()), Cow::Borrowed);

	MessageDialog::new()
		.set_text(&*msg)
		.set_type(ty)
		.show_confirm()
		.expect("Unable to confirm user")
}

/// Alerts an error
pub fn error(msg: fmt::Arguments) {
	self::alert(MessageType::Error, msg);
}

/// Alerts an error with interpolation
pub macro error($($args:tt)*) {
	$crate::alert::error(::std::format_args!($($args)*))
}

/// Alerts a warning
pub fn warn(msg: fmt::Arguments) {
	self::alert(MessageType::Warning, msg);
}

/// Alerts a warning with interpolation
pub macro warn($($args:tt)*) {
	$crate::alert::warn(::std::format_args!($($args)*))
}

/// Alerts info
pub fn info(msg: fmt::Arguments) {
	self::alert(MessageType::Info, msg);
}

/// Alerts info with interpolation
pub macro info($($args:tt)*) {
	$crate::alert::info(::std::format_args!($($args)*))
}

/// Alerts and requests a confirmation for a warning
#[must_use]
pub fn warn_confirm(msg: fmt::Arguments) -> bool {
	self::confirm(MessageType::Warning, msg)
}

/// Alerts and requests a confirmation for a warning with interpolation
pub macro warn_confirm($($args:tt)*) {
	$crate::alert::warn_confirm(::std::format_args!($($args)*))
}

/// Alerts and requests a confirmation for info
#[must_use]
pub fn info_confirm(msg: fmt::Arguments) -> bool {
	self::confirm(MessageType::Info, msg)
}

/// Alerts and requests a confirmation for a info with interpolation
pub macro info_confirm($($args:tt)*) {
	$crate::alert::info_confirm(::std::format_args!($($args)*))
}
