//! Logger

// Imports
use log::LevelFilter;
use simplelog::{CombinedLogger, Config, SharedLogger, TermLogger, TerminalMode, WriteLogger};

/// The type of logger required to pass to `CombinedLogger::init`
type BoxedLogger = Box<dyn SharedLogger>;

/// Initializes the global logger
pub fn init() {
	// All loggers to try and initialize
	let loggers = [
		Some(TermLogger::new(
			LevelFilter::Info,
			Config::default(),
			TerminalMode::Stderr,
		))
		.map(|logger| BoxedLogger::from(logger)),
		std::fs::File::create("latest.log")
			.ok()
			.map(|file| WriteLogger::new(LevelFilter::Debug, Config::default(), file))
			.map(|logger| BoxedLogger::from(logger)),
	];

	// Filter all logger that actually work and initialize them
	if CombinedLogger::init(std::array::IntoIter::new(loggers).flatten().collect()).is_err() {
		log::warn!("Logger was already initialized");
	}
}
