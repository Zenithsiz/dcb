//! Panic handlers for this application

// Std
use std::{
	backtrace::{Backtrace, BacktraceStatus},
	error::Error,
};
// Error backtrace
use err_backtrace::ErrBacktraceExt;

/// Panic handler based on logging to the current initialization
pub fn log_handler(info: &std::panic::PanicInfo) {
	// Log that this thread has panicked
	log::error!("Thread \"{}\" panicked", std::thread::current().name().unwrap_or("[Unknown]"));

	// Log any message that came with the panic
	log::info!("Panic message: {}", info.message().unwrap_or(&format_args!("None")));

	// Print an error backtrace if we found any
	if let Some(err) = info.payload().downcast_ref::<Box<dyn Error + Send + Sync>>() {
		log::info!("Error backtrace:\n{}", err.err_backtrace());
	}

	// And print a backtrace of where this panic occured.
	let backtrace = Backtrace::force_capture();
	if backtrace.status() == BacktraceStatus::Captured {
		log::info!("Backtrace:\n{}", backtrace);
	} else {
		log::info!("Unable to get backtrace: {}", backtrace);
	}
}
