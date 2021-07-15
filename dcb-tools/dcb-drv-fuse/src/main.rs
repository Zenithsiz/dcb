//! `dcb-drv` as a `FUSE` filesystem

// Features
#![feature(format_args_capture, try_blocks)]

// Modules
mod cli;
mod fs;

// Imports
use anyhow::Context;
use fuser::{MountOption, Session};

fn main() -> Result<(), anyhow::Error> {
	// Initialize the logger
	simplelog::TermLogger::init(
		log::LevelFilter::Info,
		simplelog::Config::default(),
		simplelog::TerminalMode::Stderr,
	)
	.expect("Unable to initialize logger");

	// Get CLI
	let args = cli::CliData::new();

	// Open filesystem
	let fs = fs::DrvFs::new(&args.input_file).context("Unable to open filesystem")?;

	// Then start the session
	let options = vec![MountOption::RO, MountOption::FSName("drv".to_owned())];
	let session = Session::new(fs, &args.mount_point, &options).context("Unable to create session")?;
	let session = session.spawn().context("Unable to spawn session")?;

	// Wait for input
	std::io::stdin()
		.read_line(&mut String::new())
		.context("Unable to wait for input")?;

	// And join
	session.join();

	Ok(())
}
