//! Data extractor
//!
//! # Details
//! Extracts data from the game file to several other files, that can be
//! edited and then used by `patcher` to modify the game file.
//!
//! # Syntax
//! The executable may be called as `./extractor <game file> {-o <output directory>}`
//!
//! Use the command `./extractor --help` for more information.
//!
//! # Data extracted
//! Currently only the following is extracted:
//! - Card table
//! - Deck table

// Features
#![feature(box_syntax, backtrace, panic_info_message, unsafe_block_in_unsafe_fn, array_value_iter)]
// Lints
#![warn(clippy::restriction, clippy::pedantic, clippy::nursery)]
// Instead of `unwrap`, we must use `expect` and provide a reason
#![forbid(clippy::unwrap_used)]
// We must use `unsafe` in unsafe `fn`s and specify if the guarantee is
// made by the caller or by us.
#![forbid(unsafe_op_in_unsafe_fn)]
// We'll disable the ones we don't need
#![allow(clippy::blanket_clippy_restriction_lints)]
// Necessary items may be inlined using `LTO`, so we don't need to mark them as inline
#![allow(clippy::missing_inline_in_public_items)]
// We prefer tail returns where possible, as they help with code readability in most cases.
#![allow(clippy::implicit_return)]
// We're fine with shadowing, as long as the variable is used for the same purpose.
// Hence why `clippy::shadow_unrelated` isn't allowed.
#![allow(clippy::shadow_reuse, clippy::shadow_same)]
// We panic when we know it won't happen, or if it does happen, then a panic is the best option
#![allow(clippy::panic, clippy::expect_used, clippy::unreachable, clippy::todo)]
// We use `expect` even in functions that return a `Result` / `Option` if there is a logic error
#![allow(clippy::unwrap_in_result)]
// We find it more important to be able to copy paste literals such as `0xabcd1234` than
// being able to read them, which does not provide many benefits
#![allow(clippy::unreadable_literal, clippy::unseparated_literal_suffix)]
// We separate implementations per their functionality usually, such as constructors, getters, setters, and others.
#![allow(clippy::multiple_inherent_impl)]
// Many operations we need to repeat, and to keep symmetry
#![allow(clippy::identity_op)]
// We only introduce items before their first usage, which sometimes is half-way through the code.
// We make sure that we only use the item after introduced, however.
#![allow(clippy::items_after_statements)]
// Useful for when they either change a lot with new variants / data,
// or for symmetry purposes
#![allow(clippy::match_same_arms)]
// In this library we have very grain-level error types, each function
// will have it's own error type ideally, so any errors are explicit
// by the type, without needing a section for them
#![allow(clippy::missing_errors_doc)]
// Although we generally try to avoid this, this can happen due to our module organization.
// In the future, this lint should be removed globally and only enabled for modules which
// actually require the use of it.
#![allow(clippy::module_inception, clippy::module_name_repetitions)]
// We use integer arithmetic and operations with the correct intent
#![allow(clippy::integer_arithmetic, clippy::integer_division)]
// We prefer using match ergonomic where possible
#![allow(clippy::pattern_type_mismatch)]
// Sometimes the blocks make it easier to invert their order
#![allow(clippy::if_not_else)]
// This lint triggers when using `assert`s and `todo`s, which is unsuitable for this project
#![allow(clippy::panic_in_result_fn)]

// Modules
mod cli;
#[path = "../logger.rs"]
mod logger;

// Imports
use anyhow::Context;
//use dcb_io::GameFile;
use dcb_iso9660::{fs::Entry, CdRom, Filesystem};


#[allow(clippy::print_stdout, clippy::use_debug)]
fn main() -> Result<(), anyhow::Error> {
	// Initialize the logger and set the panic handler
	logger::init();

	// Get all data from cli
	let cli::CliData { game_file_path, .. } = cli::CliData::new();

	// Open the game file
	let input_file = std::fs::File::open(&game_file_path).context("Unable to open input file")?;
	let mut cdrom = CdRom::new(input_file);
	let filesystem: Filesystem = Filesystem::new(&mut cdrom).context("Unable to read filesystem")?;

	// Get all entries and search for `a_drv`.
	let entries = filesystem
		.root_dir()
		.read_entries(&mut cdrom)
		.context("Unable to read all entries in root")?;
	let a_drv = Entry::search_entries(&entries, "A.DRV;1").context("Unable to get `A.DRV`")?;

	println!("{:?}", a_drv);


	/*
	// Read the file system
	let filesystem = Filesystem::new(&mut game_file).context("Unable to read filesystem")?;

	println!("{:#?}", filesystem);
	*/

	/*
	// Get the cards table
	let cards_table = CardTable::deserialize(&mut game_file).context("Unable to deserialize cards table from game file")?;
	let cards_table_yaml = serde_yaml::to_string(&cards_table).context("Unable to serialize cards table to yaml")?;
	log::info!("Extracted {} cards", cards_table.card_count());

	// Get the decks table
	let decks_table = DeckTable::deserialize(&mut game_file).context("Unable to deserialize decks table from game file")?;
	let decks_table_yaml = serde_yaml::to_string(&decks_table).context("Unable to serialize decks table to yaml")?;

	// And output everything to the files
	std::fs::write(&output_dir.join("cards.yaml"), cards_table_yaml).context("Unable to write cards table to file")?;
	std::fs::write(&output_dir.join("decks.yaml"), decks_table_yaml).context("Unable to write decks table to file")?;
	*/

	Ok(())
}
