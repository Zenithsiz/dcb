//! Loaded game


// Imports
use crate::EditScreen;
use anyhow::Context;
use dcb::CardTable;
use dcb_cdrom_xa::CdRomCursor;
use eframe::{egui, epi::TextureAllocator};
use std::{
	convert::TryInto,
	fs,
	io::{Seek, Write},
	path::Path,
};
use zutil::StrContainsCaseInsensitive;

/// Loaded game
pub struct LoadedGame {
	/// File
	file: CdRomCursor<fs::File>,

	/// Card table
	pub card_table: CardTable,

	/// Hash of `card_table` within disk
	file_card_table_hash: u64,

	/// Card search
	card_search: String,
}

impl LoadedGame {
	/// Loads a game from a path
	pub fn load(file_path: &Path) -> Result<Self, anyhow::Error> {
		// Open the file
		let file = fs::File::options()
			.read(true)
			.write(true)
			.open(&file_path)
			.context("Unable to open game file")?;
		let mut file = dcb_cdrom_xa::CdRomCursor::new(file);

		// Open the card table file and parse it
		let mut game_file = dcb_io::GameFile::new(&mut file);
		let mut table_file = CardTable::open(&mut game_file).context("Unable to open table file")?;

		// Then parse it
		let card_table = CardTable::deserialize(&mut table_file).context("Unable to parse table")?;
		let file_card_table_hash = zutil::hash_of(&card_table);

		Ok(Self {
			card_table,
			file,
			file_card_table_hash,
			card_search: String::new(),
		})
	}

	/// Saves this game
	pub fn save(&mut self) -> Result<(), anyhow::Error> {
		// Serialize the card table to a temporary vector
		let mut bytes = vec![];
		self.card_table
			.serialize(&mut bytes)
			.context("Unable to serialize table")?;

		// Open the card table file
		let mut game_file = dcb_io::GameFile::new(&mut self.file);
		let mut table_file = CardTable::open(&mut game_file).context("Unable to open table file")?;

		// If it's larger than the file, return Err
		let bytes_len: u64 = bytes.len().try_into().expect("Card table was larger than `u64`");
		let file_len = table_file.stream_len().context("Unable to get file size")?;
		anyhow::ensure!(
			bytes_len <= file_len,
			"Card table is too big ({} / {})",
			size_format::SizeFormatterSI::new(bytes_len),
			size_format::SizeFormatterSI::new(file_len),
		);

		// Then write it
		table_file
			.write_all(&bytes)
			.context("Unable to write card table to file")?;

		// And update our hash
		self.file_card_table_hash = zutil::hash_of(&self.card_table);

		Ok(())
	}

	/// Saves a backup of the card table to file
	pub fn save_backup(&self, path: &Path) -> Result<(), anyhow::Error> {
		let file = fs::File::create(&path).context("Unable to create backup file")?;
		serde_yaml::to_writer(file, &self.card_table).context("Unable to write backup to file")
	}

	/// Returns if the card table has been modified from disk
	pub fn modified(&self) -> bool {
		zutil::hash_of(&self.card_table) != self.file_card_table_hash
	}

	/// Displays the card selection menu
	pub fn display_card_selection(
		&mut self, ui: &mut egui::Ui, open_edit_screens: &mut Vec<EditScreen>, tex_allocator: &mut dyn TextureAllocator,
	) {
		ui.heading("Card list");

		ui.vertical(|ui| {
			ui.label("Search");
			ui.text_edit_singleline(&mut self.card_search);
		});
		ui.separator();

		let card_search = &self.card_search;
		let names = self
			.card_table
			.cards
			.iter()
			.map(|card| card.name().as_str())
			.enumerate()
			.map(|(idx, name)| (idx, format!("{idx}. {name}")))
			.filter(|(_, name)| name.contains_case_insensitive(card_search));
		let mut game_file = dcb_io::GameFile::new(&mut self.file);

		egui::ScrollArea::auto_sized().show(ui, |ui| {
			for (idx, name) in names {
				// If clicked, open/close a new screen
				let screen_idx = open_edit_screens.iter().position(|screen| screen.card_idx() == idx);
				if ui.selectable_label(screen_idx.is_some(), name).clicked() {
					match screen_idx {
						Some(screen_idx) => {
							open_edit_screens.remove(screen_idx);
						},
						None => open_edit_screens.push(EditScreen::new(idx, &mut game_file, tex_allocator)),
					}
				}
			}
		});
	}
}
