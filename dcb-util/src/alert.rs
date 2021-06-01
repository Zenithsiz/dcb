//! Alerts

// TODO: Maybe don't always format in the macros if it's just a string literal?

// Imports
use native_dialog::{MessageDialog, MessageType};

/// Alerts an error
pub fn error(msg: &str) {
	MessageDialog::new()
		.set_text(msg)
		.set_type(MessageType::Error)
		.show_alert()
		.expect("Unable to alert user");
}

/// Alerts an error with interpolation
pub macro error($($args:tt)*) {
	$crate::alert::error(&::std::format!($($args)*))
}

/// Alerts a warning
pub fn warn(msg: &str) {
	MessageDialog::new()
		.set_text(msg)
		.set_type(MessageType::Warning)
		.show_alert()
		.expect("Unable to alert user");
}

/// Alerts a warning with interpolation
pub macro warn($($args:tt)*) {
	$crate::alert::warn(&::std::format!($($args)*))
}

/// Alerts info
pub fn info(msg: &str) {
	MessageDialog::new()
		.set_text(msg)
		.set_type(MessageType::Info)
		.show_alert()
		.expect("Unable to alert user");
}

/// Alerts info with interpolation
pub macro info($($args:tt)*) {
	$crate::alert::info(&::std::format!($($args)*))
}

/// Alerts and requests a confirmation for a warning
#[must_use]
pub fn warn_confirm(msg: &str) -> bool {
	MessageDialog::new()
		.set_text(msg)
		.set_type(MessageType::Info)
		.show_confirm()
		.expect("Unable to alert user")
}

/// Alerts and requests a confirmation for a warning with interpolation
pub macro warn_confirm($($args:tt)*) {
	$crate::alert::warn_confirm(&::std::format!($($args)*))
}
