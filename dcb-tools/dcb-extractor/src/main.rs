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
#![feature(
	box_syntax,
	backtrace,
	panic_info_message,
	unsafe_block_in_unsafe_fn,
	array_value_iter,
	format_args_capture
)]
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
mod logger;

// Imports
use anyhow::Context;
use dcb_cdrom_xa::CdRom;
use dcb_io::{
	drv::{
		dir::{entry::DirEntryKind, DirEntry},
		Dir,
	},
	pak::PakEntry,
	tim::TimFile,
	DrvFs, GameFile, PakFile,
};
use std::{
	io::{self, SeekFrom},
	path::Path,
};


fn main() -> Result<(), anyhow::Error> {
	// Initialize the logger and set the panic handler
	logger::init();

	// Get all data from cli
	let cli::CliData { game_file_path, output_dir } = cli::CliData::new();

	// Open the game file
	let input_file = std::fs::File::open(&game_file_path).context("Unable to open input file")?;
	let mut cdrom = CdRom::new(input_file);
	let mut game_file = GameFile::new(&mut cdrom).context("Unable to read filesystem")?;


	log::info!("A.DRV:");
	self::print_drv_fs(&mut game_file, "A.DRV;1", &output_dir.join("A.DRV")).context("Unable to print `A.DRV`")?;
	log::info!("B.DRV:");
	self::print_drv_fs(&mut game_file, "B.DRV;1", &output_dir.join("B.DRV")).context("Unable to print `B.DRV`")?;
	log::info!("C.DRV:");
	self::print_drv_fs(&mut game_file, "C.DRV;1", &output_dir.join("C.DRV")).context("Unable to print `C.DRV`")?;
	log::info!("E.DRV:");
	self::print_drv_fs(&mut game_file, "E.DRV;1", &output_dir.join("E.DRV")).context("Unable to print `E.DRV`")?;
	log::info!("F.DRV:");
	self::print_drv_fs(&mut game_file, "F.DRV;1", &output_dir.join("F.DRV")).context("Unable to print `F.DRV`")?;
	log::info!("G.DRV:");
	self::print_drv_fs(&mut game_file, "G.DRV;1", &output_dir.join("G.DRV")).context("Unable to print `G.DRV`")?;
	log::info!("P.DRV:");
	self::print_drv_fs(&mut game_file, "P.DRV;1", &output_dir.join("P.DRV")).context("Unable to print `P.DRV`")?;

	Ok(())
}

/// Prints a drv filesystem
fn print_drv_fs<R: io::Read + io::Seek>(game_file: &mut GameFile<R>, name: &str, output_dir: &Path) -> Result<(), anyhow::Error> {
	let mut a_drv = game_file.read_drv(name).context("Unable to read file")?;
	let a_drv_fs = DrvFs::from_reader(&mut a_drv).context("Unable to parse filesystem")?;
	self::print_dir_tree(&mut a_drv, a_drv_fs.root(), output_dir).context("Unable to print")
}

/// Prints a directory tree
fn print_dir_tree<R: io::Read + io::Seek>(drv_fs: &mut R, dir: &Dir, path: &Path) -> Result<(), anyhow::Error> {
	print_dir_tree_with_depth(drv_fs, dir, path, 1)
}

/// Prints a directory tree
fn print_dir_tree_with_depth<R: io::Read + io::Seek>(mut drv_fs: &mut R, dir: &Dir, path: &Path, depth: usize) -> Result<(), anyhow::Error> {
	// Create path
	//self::_try_create_folder(path)?;

	let tabs = "\t".repeat(depth);
	for DirEntry {
		name,
		date,
		sector_pos,
		kind,
	} in dir.entries()
	{
		match kind {
			DirEntryKind::File { extension, size } => {
				let path = path.join(format!("{}.{}", name, extension));

				// If it's a `.PAK`, unpack it
				if extension.as_str() == "PAK" {
					drv_fs
						.seek(SeekFrom::Start(u64::from(*sector_pos) * 2048))
						.with_context(|| format!("Unable to seek to pak file {}", path.display()))?;
					let pak_file: PakFile =
						PakFile::deserialize(&mut drv_fs).with_context(|| format!("Unable to parse pak file {}", path.display()))?;

					log::info!("{}{}{} ({} bytes)", date, tabs, path.display(), size);
					//self::_try_create_folder(&path)?;

					let tabs = "\t".repeat(depth + 1);
					for (idx, entry) in pak_file.entries.iter().enumerate() {
						let extension = match &entry {
							PakEntry::Unknown0(_) => "UN0",
							PakEntry::Unknown1(_) => "UN1",
							PakEntry::GameScript(_) => "MSD",
							PakEntry::Animation2d(_) => "A2D",
							PakEntry::FileSubHeader(_) => "SHD",
							PakEntry::FileContents(data) => match TimFile::deserialize(std::io::Cursor::new(&data)) {
								Ok(_) => "TIM",
								Err(_) if data.starts_with(b"Tp") => "TIS",
								Err(_) => "BIN",
							},
							PakEntry::AudioSeq(_) => "SEQ",
							PakEntry::AudioVh(_) => "VH",
							PakEntry::AudioVb(_) => "VB",
						};
						let path = path.join(format!("{idx}.{extension}"));
						log::info!("{}{}{}", date, tabs, path.display());
						//log::info!("{}\tWriting file {}", tabs, path.display());

						//std::fs::write(&path, data).with_context(|| format!("Unable to write file {}", path.display()))?;
					}
				}
				// Else just write it.
				else {
					log::info!("{}{}{} ({} bytes)", date, tabs, path.display(), size);
					//std::fs::write(&path, contents).with_context(|| format!("Unable to write file {}", path.display()))?;
				}
			},
			DirEntryKind::Dir => {
				let path = path.join(name.as_str());
				log::info!("{}{}{}", date, tabs, path.display());

				//self::_try_create_folder(&path)?;
				drv_fs
					.seek(SeekFrom::Start(u64::from(*sector_pos) * 2048))
					.with_context(|| format!("Unable to seek to directory {}", path.display()))?;
				let dir = Dir::from_reader(&mut drv_fs).with_context(|| format!("Unable to parse directory {}", path.display()))?;
				self::print_dir_tree_with_depth(drv_fs, &dir, &path, depth + 1)
					.with_context(|| format!("Unable to write directory {}", path.display()))?;
			},
		}
	}

	Ok(())
}

/// Attempts to create a folder. Returns `Ok` if it already exists.
#[allow(clippy::create_dir)] // We only want to create a single level
fn _try_create_folder(path: impl AsRef<std::path::Path>) -> Result<(), anyhow::Error> {
	match std::fs::create_dir(&path) {
		// If it already exists, ignore
		Ok(_) => Ok(()),
		Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => Ok(()),
		Err(err) => Err(err).with_context(|| format!("Unable to create directory {}", path.as_ref().display())),
	}
}
