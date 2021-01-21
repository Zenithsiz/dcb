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
#[path = "../logger.rs"]
mod logger;

// Imports
use anyhow::Context;
use dcb_io::{
	drv::{dir::DirEntry, Dir},
	pak::{
		entry::{animation2d::Frame, Animation2d},
		PakEntry,
	},
	GameFile, PakFile,
};
use dcb_cdrom_xa::CdRom;
use std::path::PathBuf;


fn main() -> Result<(), anyhow::Error> {
	// Initialize the logger and set the panic handler
	logger::init();

	// Get all data from cli
	let cli::CliData { game_file_path, output_dir } = cli::CliData::new();

	// Open the game file
	let input_file = std::fs::File::open(&game_file_path).context("Unable to open input file")?;
	let mut cdrom = CdRom::new(input_file);
	let game_file = GameFile::new(&mut cdrom).context("Unable to read filesystem")?;

	log::info!("A.DRV:");
	self::write_dir(game_file.a_drv.root(), &output_dir.join("A.DRV")).context("Unable to write `A.DRV`")?;
	log::info!("B.DRV:");
	self::write_dir(game_file.b_drv.root(), &output_dir.join("B.DRV")).context("Unable to write `B.DRV`")?;
	log::info!("C.DRV:");
	self::write_dir(game_file.c_drv.root(), &output_dir.join("C.DRV")).context("Unable to write `C.DRV`")?;
	log::info!("E.DRV:");
	self::write_dir(game_file.e_drv.root(), &output_dir.join("E.DRV")).context("Unable to write `E.DRV`")?;
	log::info!("F.DRV:");
	self::write_dir(game_file.f_drv.root(), &output_dir.join("F.DRV")).context("Unable to write `F.DRV`")?;
	log::info!("G.DRV:");
	self::write_dir(game_file.g_drv.root(), &output_dir.join("G.DRV")).context("Unable to write `G.DRV`")?;
	log::info!("P.DRV:");
	self::write_dir(game_file.p_drv.root(), &output_dir.join("P.DRV")).context("Unable to write `P.DRV`")?;

	Ok(())
}

/// Prints a directory tree
#[allow(clippy::print_stdout, clippy::print_stderr)]
fn write_dir(dir: &Dir, path: &PathBuf) -> Result<(), anyhow::Error> {
	//write_dir_with_depth(dir, path, 1)

	for entry in dir.entries() {
		match entry {
			DirEntry::File {
				name, extension, contents, ..
			} => {
				let path = path.join(format!("{}.{}", name, extension));

				// If it's a `.PAK`, unpack it
				if extension.as_str() == "PAK" {
					let pak_file: PakFile =
						PakFile::deserialize(std::io::Cursor::new(contents)).with_context(|| format!("Unable to parse file {}", path.display()))?;

					for (idx, entry) in pak_file.entries.iter().enumerate() {
						if let PakEntry::Animation2d(animation2d) = &entry {
							let Animation2d {
								file_name,
								unknown0,
								unknown1,
								unknown2,
								unknown3,
								unknown4,
								unknown5,
								unknown6,
								frames,
							} = animation2d;
							#[rustfmt::skip]
							println!(
								"{file_name}[{idx}]\t{unknown0:#x}\t{unknown1:#x}\t{unknown2:#x}\t{unknown3:#x}\t{unknown4:#x}\t{unknown5:#x}\t{unknown6:#x}\t"
							);
							for frame in frames {
								let Frame {
									unknown0,
									x0,
									x1,
									y0,
									y1,
									width,
									height,
									unknown1,
									duration,
									unknown2,
									unknown3,
								} = frame;
								#[rustfmt::skip]
								eprint!(
									"{file_name}[{idx}]\t{unknown0:#x}\t{x0:#x}\t{x1:#x}\t{y0:#x}\t{y1:#x}\t{width:#x}\t{height:#x}\t{unknown1:#x}\t{duration:#x}\t{unknown2:#x}"
								);
								let digits = [
									(unknown3 & 0xF000) >> 0xc,
									(unknown3 & 0x0F00) >> 0x8,
									(unknown3 & 0x00F0) >> 0x4,
									(unknown3 & 0x000F) >> 0x0,
								];
								for digit in &digits {
									eprint!("\t{digit:#x}");
								}
								eprintln!();
							}
							//log::info!("{}[{}]: {:?}", path.display(), idx, animation2d);
						};
					}
				}
			},
			DirEntry::Dir { name, dir, .. } => {
				let path = path.join(name.as_str());
				self::write_dir(dir, &path).with_context(|| format!("Unable to write directory {}", path.display()))?;
			},
		}
	}

	Ok(())
}

/// Prints a directory tree
fn _write_dir_with_depth(dir: &Dir, path: &PathBuf, depth: usize) -> Result<(), anyhow::Error> {
	// Create path
	self::_try_create_folder(path)?;

	let tabs = "\t".repeat(depth);
	for entry in dir.entries() {
		match entry {
			DirEntry::File {
				name, extension, contents, ..
			} => {
				let path = path.join(format!("{}.{}", name, extension));

				// If it's a `.PAK`, unpack it
				if extension.as_str() == "PAK" {
					let _pak_file: PakFile =
						PakFile::deserialize(std::io::Cursor::new(contents)).with_context(|| format!("Unable to parse file {}", path.display()))?;

					log::info!("{}Creating directory {}", tabs, path.display());
					self::_try_create_folder(&path)?;
				/*
				for (idx, entry) in pak_file.entries.iter().enumerate() {
					let (extension, data) = match &entry {
						PakEntry::Unknown0(data) => ("UN0", data),
						PakEntry::Unknown1(data) => ("UN1", data),
						PakEntry::GameScript(data) => ("MSD", data),
						PakEntry::Animation2D(_data) => ("A2D", todo!()),
						PakEntry::FileSubHeader(data) => ("SHD", data),
						PakEntry::FileContents(data) => (
							match TimFile::deserialize(std::io::Cursor::new(&data)) {
								Ok(_) => "TIM",
								Err(_) if data.starts_with(b"Tp") => "TIS",
								Err(_) => "BIN",
							},
							data,
						),
						PakEntry::AudioSeq(data) => ("SEQ", data),
						PakEntry::AudioVh(data) => ("VH", data),
						PakEntry::AudioVb(data) => ("VB", data),
					};
					let path = path.join(format!("{idx}.{extension}"));
					log::info!("{}\tWriting file {}", tabs, path.display());

					std::fs::write(&path, data).with_context(|| format!("Unable to write file {}", path.display()))?;
				}
				*/
				}
				// Else just write it.
				else {
					log::info!("{}Writing file {}", tabs, path.display());
					std::fs::write(&path, contents).with_context(|| format!("Unable to write file {}", path.display()))?;
				}
			},
			DirEntry::Dir { name, dir, .. } => {
				let path = path.join(name.as_str());
				log::info!("{}Creating directory {}", tabs, path.display());

				self::_try_create_folder(&path)?;
				self::_write_dir_with_depth(dir, &path, depth + 1).with_context(|| format!("Unable to write directory {}", path.display()))?;
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
