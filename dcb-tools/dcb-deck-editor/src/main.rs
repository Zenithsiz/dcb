//! Deck editor

// Features
#![feature(array_map, with_options, format_args_capture, once_cell, never_type)]

// Modules
pub mod ascii_text_buffer;

// Exports
pub use ascii_text_buffer::AsciiTextBuffer;

// Imports
use anyhow::Context;
use dcb::{CardTable, Deck, DeckTable};
use dcb_util::StrContainsCaseInsensitive;
use eframe::{egui, epi, NativeOptions};
use native_dialog::{FileDialog, MessageDialog, MessageType};
use ref_cast::RefCast;
use std::{
	fs,
	io::{self, Read, Seek},
	ops::Range,
	path::{Path, PathBuf},
};

fn main() {
	// Initialize the logger
	simplelog::TermLogger::init(
		log::LevelFilter::Debug,
		simplelog::Config::default(),
		simplelog::TerminalMode::Stderr,
		simplelog::ColorChoice::Auto,
	)
	.expect("Unable to initialize logger");

	// Crate the app and run it
	let app = DeckEditor::default();
	eframe::run_native(Box::new(app), NativeOptions::default());
}

pub struct DeckEditor {
	/// File path
	file_path: Option<PathBuf>,

	/// Loaded game
	loaded_game: Option<LoadedGame>,

	/// Deck search
	deck_search: String,

	/// All selected edit screens
	open_edit_screens: Vec<EditScreen>,
}

impl DeckEditor {
	/// Card table offset
	pub const CARD_TABLE_OFFSET: u64 = 0x216d000;
	/// Card table size
	pub const CARD_TABLE_SIZE: u64 = 0x14958;
	/// Deck table offset
	pub const DECK_TABLE_OFFSET: u64 = 0x21a6800;
	/// Deck table size
	pub const DECK_TABLE_SIZE: u64 = 0x445a;

	/// Parses the card table from file
	pub fn parse_card_table(file_path: &Path) -> Result<CardTable, anyhow::Error> {
		// Open the file
		let file = fs::File::open(file_path).context("Unable to open file")?;
		let mut file = dcb_cdrom_xa::CdRomCursor::new(file);

		// Seek to the card file position and limit our reading to the file size
		file.seek(io::SeekFrom::Start(Self::CARD_TABLE_OFFSET))
			.context("Unable to seek to card table")?;
		let mut file = file.take(Self::CARD_TABLE_SIZE);

		// Then parse it
		let card_table = CardTable::deserialize(&mut file).context("Unable to parse table")?;

		Ok(card_table)
	}

	/// Parses the deck table from file
	pub fn parse_deck_table(file_path: &Path) -> Result<DeckTable, anyhow::Error> {
		// Open the file
		let file = fs::File::open(file_path).context("Unable to open file")?;
		let mut file = dcb_cdrom_xa::CdRomCursor::new(file);

		// Seek to the deck file position and limit our reading to the file size
		file.seek(io::SeekFrom::Start(Self::DECK_TABLE_OFFSET))
			.context("Unable to seek to deck table")?;
		let mut file = file.take(Self::DECK_TABLE_SIZE);

		// Then parse it
		let deck_table = DeckTable::deserialize(&mut file).context("Unable to parse table")?;

		Ok(deck_table)
	}

	/// Saves the deck table to file
	pub fn save_deck_table(file_path: &Path, deck_table: &DeckTable) -> Result<(), anyhow::Error> {
		// Open the file
		let file = fs::File::with_options()
			.write(true)
			.open(file_path)
			.context("Unable to open file")?;
		let mut file = dcb_cdrom_xa::CdRomCursor::new(file);

		// Seek to the deck file position and limit our writing to the file size
		file.seek(io::SeekFrom::Start(Self::DECK_TABLE_OFFSET))
			.context("Unable to seek to deck table")?;
		let mut file = dcb_util::WriteTake::new(file, Self::DECK_TABLE_SIZE);

		// Then serialize it
		deck_table.serialize(&mut file).context("Unable to serialize table")?;

		Ok(())
	}

	/// Returns the digimon's indexes
	pub fn digimon_idxs(card_table: &CardTable) -> Range<usize> {
		0..card_table.digimons.len()
	}

	/// Returns the item's indexes
	pub fn item_idxs(card_table: &CardTable) -> Range<usize> {
		card_table.digimons.len()..(card_table.digimons.len() + card_table.items.len())
	}

	/// Returns the digivolve's indexes
	pub fn digivolve_idxs(card_table: &CardTable) -> Range<usize> {
		(card_table.digimons.len() + card_table.items.len())..
			(card_table.digimons.len() + card_table.items.len() + card_table.digivolves.len())
	}

	/// Returns a card given it's index
	pub fn get_card_from_idx(card_table: &mut CardTable, idx: usize) -> Card {
		let digimons_len = card_table.digimons.len();
		let items_len = card_table.items.len();

		if Self::digimon_idxs(card_table).contains(&idx) {
			Card::Digimon(&mut card_table.digimons[idx])
		} else if Self::item_idxs(card_table).contains(&idx) {
			Card::Item(&mut card_table.items[idx - digimons_len])
		} else if Self::digivolve_idxs(card_table).contains(&idx) {
			Card::Digivolve(&mut card_table.digivolves[idx - digimons_len - items_len])
		} else {
			panic!("Invalid card index");
		}
	}
}

impl Default for DeckEditor {
	fn default() -> Self {
		Self {
			file_path:         None,
			loaded_game:       None,
			deck_search:       String::new(),
			open_edit_screens: vec![],
		}
	}
}

impl epi::App for DeckEditor {
	fn update(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) {
		let Self {
			file_path,
			loaded_game,
			deck_search,
			open_edit_screens,
		} = self;

		// Top panel
		egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
			egui::menu::bar(ui, |ui| {
				egui::menu::menu(ui, "File", |ui| {
					// On open, ask the user and open the file
					if ui.button("Open").clicked() {
						let cur_dir_path = std::env::current_dir().expect("Unable to get current directory path");
						*file_path = FileDialog::new()
							.set_location(&cur_dir_path)
							.add_filter("Game file", &["bin"])
							.show_open_single_file()
							.expect("Unable to ask user for file");

						// Then load the card table if we got a file
						if let Some(file_path) = file_path {
							match (
								Self::parse_card_table(file_path).context("Unable to load card table"),
								Self::parse_deck_table(file_path).context("Unable to load deck table"),
							) {
								(Ok(card_table), Ok(deck_table)) => {
									*loaded_game = Some(LoadedGame { card_table, deck_table });
								},
								(Err(err), _) => MessageDialog::new()
									.set_text(&format!("Unable to open file: {:?}", err))
									.set_type(MessageType::Error)
									.show_alert()
									.expect("Unable to alert user"),
								(_, Err(err)) => MessageDialog::new()
									.set_text(&format!("Unable to open file: {:?}", err))
									.set_type(MessageType::Error)
									.show_alert()
									.expect("Unable to alert user"),
							}
						}
					}

					// On save, if we have a file, save it to there, else tell error
					if ui.button("Save").clicked() {
						match (&file_path, &mut *loaded_game) {
							(Some(file_path), Some(loaded_game)) => {
								match Self::save_deck_table(file_path, &loaded_game.deck_table) {
									Ok(()) => MessageDialog::new()
										.set_text("Successfully saved!")
										.set_type(MessageType::Info)
										.show_alert()
										.expect("Unable to alert user"),
									Err(err) => MessageDialog::new()
										.set_text(&format!("Unable to save file: {:?}", err))
										.set_type(MessageType::Error)
										.show_alert()
										.expect("Unable to alert user"),
								}
							},
							_ => MessageDialog::new()
								.set_text("You must first open a file to save")
								.set_type(MessageType::Warning)
								.show_alert()
								.expect("Unable to alert user"),
						}
					}

					if ui.button("Quit").clicked() {
						frame.quit();
					}
				});

				/*
				egui::menu::menu(ui, "Edit", |ui| {
					if loaded_game.is_some() && ui.button("Swap").clicked() {
						todo!();
					}
				});
				*/
			});
		});

		egui::SidePanel::left("side_panel").show(ctx, |ui| {
			ui.heading("Deck list");

			ui.vertical(|ui| {
				ui.label("Search");
				ui.text_edit_singleline(deck_search);
			});

			// If we have a loaded game, display all decks
			if let Some(loaded_game) = &loaded_game {
				let names = loaded_game
					.deck_table
					.decks
					.iter()
					.map(|deck| deck.name)
					.enumerate()
					.map(|(idx, name)| (idx, format!("{idx}. {name}")))
					.filter(|(_, name)| name.contains_case_insensitive(deck_search));

				egui::ScrollArea::auto_sized().show(ui, |ui| {
					for (idx, name) in names {
						// If clicked, open/close a new screen
						let screen_idx = open_edit_screens.iter().position(|screen| screen.deck_idx == idx);
						if ui.selectable_label(screen_idx.is_some(), name).clicked() {
							match screen_idx {
								Some(screen_idx) => {
									open_edit_screens.remove(screen_idx);
								},
								None => open_edit_screens.push(EditScreen { deck_idx: idx }),
							}
						}
					}
				});
			}
		});

		// For every screen, display it
		if let Some(loaded_game) = loaded_game {
			egui::CentralPanel::default().show(ctx, |ui| {
				let screens_len = open_edit_screens.len();
				for screen in open_edit_screens {
					let deck = &mut loaded_game.deck_table.decks[screen.deck_idx];
					let card_table = &mut loaded_game.card_table;

					let total_available_width = ui.available_width();
					let default_width = total_available_width / (screens_len as f32);
					egui::SidePanel::left((screen as *const _, "panel", default_width.to_bits()))
						.default_width(default_width)
						.show(ctx, |ui| {
							// Header for the card
							ui.vertical(|ui| {
								ui.heading(deck.name.as_str());
								ui.separator();
							});

							egui::ScrollArea::auto_sized().show(ui, |ui| {
								self::render_deck(ui, deck, card_table);
							});
						});
				}
			});
		}
	}

	fn on_exit(&mut self) {
		todo!();
	}

	fn name(&self) -> &str {
		"Dcb deck editor"
	}
}

/// An edit screen
pub struct EditScreen {
	/// Currently selected deck
	deck_idx: usize,
}

/// Loaded game
pub struct LoadedGame {
	/// Card table
	card_table: CardTable,

	/// Deck table
	deck_table: DeckTable,
}

/// Digimon, Item or digivolve
pub enum Card<'a> {
	Digimon(&'a mut dcb::Digimon),
	Item(&'a mut dcb::Item),
	Digivolve(&'a mut dcb::Digivolve),
}

impl<'a> Card<'a> {
	/// Returns the name of this card
	pub fn name(&self) -> &str {
		match self {
			Card::Digimon(digimon) => digimon.name.as_str(),
			Card::Item(item) => item.name.as_str(),
			Card::Digivolve(digivolve) => digivolve.name.as_str(),
		}
	}
}

/// Renders a deck
fn render_deck(ui: &mut egui::Ui, deck: &mut Deck, card_table: &mut CardTable) {
	// Name
	ui.horizontal(|ui| {
		ui.label("Name");
		ui.text_edit_singleline(AsciiTextBuffer::ref_cast_mut(&mut deck.name));
	});

	// Owner
	ui.horizontal(|ui| {
		ui.label("Owner");
		ui.text_edit_singleline(AsciiTextBuffer::ref_cast_mut(&mut deck.owner));
	});

	ui.group(|ui| {
		ui.label("Cards");
		for card_id in &mut deck.cards {
			ui.horizontal(|ui| {
				let card = DeckEditor::get_card_from_idx(card_table, usize::from(card_id.0));

				ui.add(egui::Slider::new(&mut card_id.0, 0..=300).clamp_to_range(true));
				ui.label(card.name());
			});
		}
	});
}
