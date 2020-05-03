//! Logger initialization

// Log
use log::LevelFilter;
use simplelog::{CombinedLogger, Config, SharedLogger, TermLogger, TerminalMode, WriteLogger};

// Error
use err_ext::ResultExt;

/// The type of logger required to pass to `CombinedLogger::init`
type BoxedLogger = Box<dyn SharedLogger>;

/// Initializes the global logger
pub fn init() {
	// All loggers to try and initialize
	let loggers: Vec<Option<BoxedLogger>> = vec![
		TermLogger::new(LevelFilter::Warn, Config::default(), TerminalMode::Mixed).map(|logger| BoxedLogger::from(logger)),
		std::fs::File::create("latest.log")
			.ok()
			.map(|file| WriteLogger::new(LevelFilter::Trace, Config::default(), file))
			.map(|logger| BoxedLogger::from(logger)),
	];

	// Filter all logger that actually work and initialize them
	CombinedLogger::init(loggers.into_iter().filter_map(std::convert::identity).collect())
		.ignore_with_err(|_| log::warn!("Logger was already initialized at the start of the program"));
}
