//! Loaded game


// Imports
use crate::{Card, EditScreen};
use anyhow::Context;
use dcb::CardTable;
use dcb_util::StrContainsCaseInsensitive;
use eframe::egui;
use std::{
	cell::Cell,
	fs,
	io::{self, Read, Seek},
	ops::Range,
	path::{Path, PathBuf},
};

/// Loaded game
pub struct LoadedGame {
	/// File path
	file_path: PathBuf,

	/// Card table
	card_table: CardTable,

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

	/// Returns the digimon's indexes
	pub fn digimon_idxs(&self) -> Range<usize> {
		0..self.card_table.digimons.len()
	}

	/// Returns the item's indexes
	pub fn item_idxs(&self) -> Range<usize> {
		self.card_table.digimons.len()..(self.card_table.digimons.len() + self.card_table.items.len())
	}

	/// Returns the digivolve's indexes
	pub fn digivolve_idxs(&self) -> Range<usize> {
		(self.card_table.digimons.len() + self.card_table.items.len())..
			(self.card_table.digimons.len() + self.card_table.items.len() + self.card_table.digivolves.len())
	}

	/// Returns a card given it's index
	pub fn get_card_from_idx(&mut self, idx: usize) -> Card {
		let digimons_len = self.card_table.digimons.len();
		let items_len = self.card_table.items.len();

		if self.digimon_idxs().contains(&idx) {
			Card::Digimon(&mut self.card_table.digimons[idx])
		} else if self.item_idxs().contains(&idx) {
			Card::Item(&mut self.card_table.items[idx - digimons_len])
		} else if self.digivolve_idxs().contains(&idx) {
			Card::Digivolve(&mut self.card_table.digivolves[idx - digimons_len - items_len])
		} else {
			panic!("Invalid card index");
		}
	}

	/// Swaps two cards in the card table
	pub fn swap_cards(&mut self, lhs_idx: usize, rhs_idx: usize) {
		let digimon_idxs = self.digimon_idxs();
		let item_idxs = self.item_idxs();
		let digivolve_idxs = self.digivolve_idxs();
		let digimons_len = self.card_table.digimons.len();
		let items_len = self.card_table.items.len();


		if digimon_idxs.contains(&lhs_idx) && digimon_idxs.contains(&rhs_idx) {
			self.card_table.digimons.swap(lhs_idx, rhs_idx);
		} else if item_idxs.contains(&lhs_idx) && item_idxs.contains(&rhs_idx) {
			self.card_table
				.items
				.swap(lhs_idx - digimons_len, rhs_idx - digimons_len);
		} else if digivolve_idxs.contains(&lhs_idx) && digivolve_idxs.contains(&rhs_idx) {
			self.card_table
				.digivolves
				.swap(lhs_idx - digimons_len - items_len, rhs_idx - digimons_len - items_len);
		} else {
			panic!("Invalid indexes {} & {}", lhs_idx, rhs_idx);
		}
	}

	/// Displays the card selection menu
	pub fn display_card_selection(
		&self, card_search: &str, ui: &mut egui::Ui, open_edit_screens: &mut Vec<EditScreen>,
	) {
		let names = self
			.card_table
			.digimons
			.iter()
			.map(|digimon| digimon.name.as_str())
			.chain(self.card_table.items.iter().map(|item| item.name.as_str()))
			.chain(
				self.card_table
					.digivolves
					.iter()
					.map(|digivolve| digivolve.name.as_str()),
			)
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

	/// Get a reference to the loaded game's card table.
	pub fn card_table(&self) -> &CardTable {
		&self.card_table
	}
}
