//! Loaded game


// Imports
use crate::EditScreen;
use anyhow::Context;
use dcb::CardTable;
use dcb_util::StrContainsCaseInsensitive;
use eframe::egui;
use std::{
	cell::Cell,
	fs,
	io::{self, Read, Seek},
	path::{Path, PathBuf},
};

/// Loaded game
pub struct LoadedGame {
	/// File path
	file_path: PathBuf,

	/// Card table
	pub card_table: CardTable,

	/// Hash of `card_table` within disk
	saved_card_table_hash: Cell<u64>,
}

impl LoadedGame {
	/// Card table offset
	pub const CARD_TABLE_OFFSET: u64 = 0x216d000;
	/// Card table size
	pub const CARD_TABLE_SIZE: u64 = 0x14958;

	/// Loads a game from a path
	pub fn load(file_path: PathBuf) -> Result<Self, anyhow::Error> {
		// Open the file
		let file = fs::File::open(&file_path).context("Unable to open file")?;
		let mut file = dcb_cdrom_xa::CdRomCursor::new(file);

		// Seek to the card file position and limit our reading to the file size
		// TODO: Use `dcb_io::GameFile`
		file.seek(io::SeekFrom::Start(Self::CARD_TABLE_OFFSET))
			.context("Unable to seek to card table")?;
		let mut file = file.take(Self::CARD_TABLE_SIZE);

		// Then parse it
		let card_table = CardTable::deserialize(&mut file).context("Unable to parse table")?;
		let saved_card_table_hash = dcb_util::hash_of(&card_table);

		Ok(Self {
			file_path,
			card_table,
			saved_card_table_hash: Cell::new(saved_card_table_hash),
		})
	}

	/// Saves this game
	pub fn save(&self) -> Result<(), anyhow::Error> {
		self.save_as(&self.file_path)
	}

	/// Saves this game to `path`
	pub fn save_as(&self, path: &Path) -> Result<(), anyhow::Error> {
		// If we haven't been modified, return
		if !self.modified() {
			return Ok(());
		}

		// Else open the file
		let file = fs::File::with_options()
			.write(true)
			.open(path)
			.context("Unable to open file")?;
		let mut file = dcb_cdrom_xa::CdRomCursor::new(file);

		// Seek to the card file position and limit our writing to the file size
		file.seek(io::SeekFrom::Start(Self::CARD_TABLE_OFFSET))
			.context("Unable to seek to card table")?;
		let mut file = dcb_util::WriteTake::new(file, Self::CARD_TABLE_SIZE);

		// Then serialize it
		self.card_table
			.serialize(&mut file)
			.context("Unable to serialize table")?;

		// And update our hash
		self.saved_card_table_hash.set(dcb_util::hash_of(&self.card_table));

		Ok(())
	}

	/// Saves a backup of the card table to file
	pub fn save_backup(&self, path: &Path) -> Result<(), anyhow::Error> {
		let file = fs::File::create(&path).context("Unable to create backup file")?;
		serde_yaml::to_writer(file, &self.card_table).context("Unable to write backup to file")
	}

	/// Returns if the card table has been modified from disk
	pub fn modified(&self) -> bool {
		dcb_util::hash_of(&self.card_table) != self.saved_card_table_hash.get()
	}

	/// Displays the card selection menu
	pub fn display_card_selection(
		&self, card_search: &str, ui: &mut egui::Ui, open_edit_screens: &mut Vec<EditScreen>,
	) {
		let names = self
			.card_table
			.cards
			.iter()
			.map(|card| card.name().as_str())
			.enumerate()
			.map(|(idx, name)| (idx, format!("{idx}. {name}")))
			.filter(|(_, name)| name.contains_case_insensitive(card_search));

		egui::ScrollArea::auto_sized().show(ui, |ui| {
			for (idx, name) in names {
				// If clicked, open/close a new screen
				let screen_idx = open_edit_screens.iter().position(|screen| screen.card_idx() == idx);
				if ui.selectable_label(screen_idx.is_some(), name).clicked() {
					match screen_idx {
						Some(screen_idx) => {
							open_edit_screens.remove(screen_idx);
						},
						None => open_edit_screens.push(EditScreen::new(idx)),
					}
				}
			}
		});
	}
}
