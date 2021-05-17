//! `.Msd` extractor

// Features
#![feature(array_chunks, format_args_capture, bool_to_option, assert_matches)]

// Modules
mod cli;

// Imports
use anyhow::Context;
use byteorder::{ByteOrder, LittleEndian};
use cli::CliData;
use std::fs;


fn main() -> Result<(), anyhow::Error> {
	// Initialize the logger
	simplelog::TermLogger::init(
		log::LevelFilter::Info,
		simplelog::Config::default(),
		simplelog::TerminalMode::Stderr,
	)
	.expect("Unable to initialize logger");

	// Get all data from cli
	let cli_data = CliData::new();

	// Read the file
	let mut contents = fs::read(&cli_data.input_file).context("Unable to read file")?;

	// Skip header
	contents.drain(..0x10);

	let mut iter = contents.iter();
	loop {
		let pos = iter.as_slice().as_ptr() as usize - contents.as_slice().as_ptr() as usize;
		print!("{pos:#010x}: ");

		match parse_command(iter.by_ref().copied()) {
			Some(()) => continue,
			None => break,
		}
	}

	assert_eq!(iter.next(), None);

	Ok(())
}

/// Parses the next command
pub fn parse_command(mut iter: impl Iterator<Item = u8>) -> Option<()> {
	match [iter.next()?, iter.next()?, iter.next()?, iter.next()?] {
		[0x0a, 0x0, 0x4, 0x0] => println!("display_buffer"),
		[0x0a, 0x0, 0x5, 0x0] => println!("wait_input"),
		[0x0a, 0x0, 0x6, 0x0] => println!("new_screen"),
		[0x0a, 0x0, 0x1, 0x0] => println!("finish_menu"),
		[0x0a, 0x0, value, kind] => println!("unknown_0a {value:#x}, {kind:#x}"),

		// Set buffer
		[0x08, 0x0, kind, 0x0] => {
			match kind {
				0x4 => print!("set_text_buffer \""),
				_ => print!("set_buffer {kind:#x}, \""),
			}
			let len = LittleEndian::read_u16(&[iter.next()?, iter.next()?]);
			for n in 0..len {
				let is_last = n == len - 1;

				match (iter.next()?, is_last) {
					(0, true) => break,
					(0, false) => log::warn!("Found null in non-last position of string"),
					(ch, true) => log::warn!("Found non-null {ch:#x} in last position of string"),
					(ch, false) => print!("{}", ch as char),
				}
			}

			let pad_len = match (len + 2) % 4 {
				0 => 0,
				n => 4 - n,
			};
			for _ in 0..pad_len {
				iter.next()?;
			}

			println!("\"");
		},

		// Set brightness
		[0x0d, 0x0, kind, 0x0] => {
			skip_0(&mut iter)?;
			skip_0(&mut iter)?;
			let place = LittleEndian::read_u16(&[iter.next()?, iter.next()?]);
			skip_0(&mut iter)?;
			skip_0(&mut iter)?;
			let brightness = LittleEndian::read_u16(&[iter.next()?, iter.next()?]);
			skip_0(&mut iter)?;
			skip_0(&mut iter)?;
			let value = LittleEndian::read_u16(&[iter.next()?, iter.next()?]);

			match (kind, place, value) {
				(0x0, 0x0, 0xa) => println!("set_light_left_char {brightness:#x}"),
				(0x0, 0x1, 0xa) => println!("set_light_right_char {brightness:#x}"),
				_ => println!("set_light {kind:#x}, {place:#x}, {brightness:#x}, {value:#x}"),
			}
		},

		// Open menu
		[0x0b, 0x0, 0x0, 0x0] => {
			skip_0(&mut iter)?;
			skip_0(&mut iter)?;
			let value = LittleEndian::read_u16(&[iter.next()?, iter.next()?]);

			// value: 0x61 0x78

			println!("open_menu {value:#x}");
		},
		[0x0b, 0x0, 0x1, 0x0] => {
			skip_0(&mut iter)?;
			skip_0(&mut iter)?;
			let value = LittleEndian::read_u16(&[iter.next()?, iter.next()?]);

			println!("add_menu_option {value:#x}")
		},

		// Choice jump
		[0x09, 0x0, value0, kind] => {
			let value1 = LittleEndian::read_u32(&[iter.next()?, iter.next()?, iter.next()?, iter.next()?]);
			let value2 = LittleEndian::read_u32(&[iter.next()?, iter.next()?, iter.next()?, iter.next()?]);

			// value1: 0x3 0x5
			// kind: 0x0 0x1

			// value1: If 0x3, then buttons work normally
			// value1: If 0x1, then buttons work reverse
			// value1: If 0x5, they both choose "No"
			// value1: If 0x7, they both choose "Yes"

			// value2: If 0x0, they both choose "No"

			println!("menu_choice_offsets {value0:#x}, {kind:#x}, {value1:#x}, {value2:#x}")
		},

		// Jump?
		[0x05, 0x0, value, kind] => {
			let addr = LittleEndian::read_u32(&[iter.next()?, iter.next()?, iter.next()?, iter.next()?]);

			println!("jump {value:#x}, {kind:#x}, {addr:#010x}");
		},

		// Display scene?
		[0x0b, 0x0, value0, 0x0] => {
			skip_0(&mut iter)?;
			skip_0(&mut iter)?;
			let deck_id = LittleEndian::read_u16(&[iter.next()?, iter.next()?]);

			// value0: 0x2 0x3 0x4 0x6 0x7 0x8 0x9 0xa 0xc 0xd 0xe 0xf 0x10 0x11 0x12 0x13 0x14 0x15

			match (value0, deck_id) {
				(0x2, _) => println!("battle {deck_id:#x}"),

				(0xf, 0x81) => println!("battle1"),
				(0xe, 0x3c) => println!("battle2"),

				_ => println!("display_scene {value0:#x}, {deck_id:#x}"),
			}
		},

		// Set variable
		[0x07, 0x0, var, 0x0] => {
			let value0 = LittleEndian::read_u32(&[iter.next()?, iter.next()?, iter.next()?, iter.next()?]);
			let value1 = LittleEndian::read_u32(&[iter.next()?, iter.next()?, iter.next()?, iter.next()?]);

			println!("set_value {var:#x}, {value0:#x}, {value1:#x}");
		},

		// ??
		[0x07, 0x0, value0, kind] => {
			let value1 = LittleEndian::read_u32(&[iter.next()?, iter.next()?, iter.next()?, iter.next()?]);
			let value2 = LittleEndian::read_u32(&[iter.next()?, iter.next()?, iter.next()?, iter.next()?]);

			println!("unknown_07 {value0:#x}, {kind:#x}, {value1:#x}, {value2:#x}");
		},

		[a, b, c, d] => println!("--- {a:02x} {b:02x} {c:02x} {d:02x}"),
	}

	Some(())
}

fn skip_0(iter: &mut impl Iterator<Item = u8>) -> Option<()> {
	match iter.next()? {
		0 => (),
		x => log::warn!("Found non-zero value: {x:#x}"),
	};

	Some(())
}
