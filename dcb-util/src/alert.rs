//! Alerts

// Imports
use native_dialog::{MessageDialog, MessageType};

/// Alerts an error to the user
pub fn error(msg: &str) {
	MessageDialog::new()
		.set_text(msg)
		.set_type(MessageType::Error)
		.show_alert()
		.expect("Unable to alert user");
}

/// Alerts a warning to the user
pub fn warn(msg: &str) {
	MessageDialog::new()
		.set_text(msg)
		.set_type(MessageType::Warning)
		.show_alert()
		.expect("Unable to alert user");
}

/// Alerts info to the user
pub fn info(msg: &str) {
	MessageDialog::new()
		.set_text(msg)
		.set_type(MessageType::Info)
		.show_alert()
		.expect("Unable to alert user");
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
