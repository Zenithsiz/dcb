//! Card editor

// Features
#![feature(array_map, with_options, format_args_capture, once_cell, never_type)]

// Modules
pub mod ascii_text_buffer;

// Exports
pub use ascii_text_buffer::AsciiTextBuffer;

// Imports
use anyhow::Context;
use dcb::{
	card::property::{
		ArrowColor, AttackType, CardType, CrossMoveEffect, DigimonProperty, DigivolveEffect, Effect, EffectCondition,
		EffectConditionOperation, EffectOperation, Level, Move, PlayerType, Slot, Speciality,
	},
	CardTable,
};
use dcb_bytes::Validate;
use eframe::{egui, epi, NativeOptions};
use either::Either;
use native_dialog::{FileDialog, MessageDialog, MessageType};
use ref_cast::RefCast;
use std::{
	collections::hash_map::DefaultHasher,
	fs,
	hash::{Hash, Hasher},
	io::{self, Read, Seek},
	lazy::SyncLazy,
	ops::Range,
	path::{Path, PathBuf},
	sync::Mutex,
	time::{Duration, SystemTime},
};
use strum::IntoEnumIterator;

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
	let app = CardEditor::default();
	eframe::run_native(Box::new(app), NativeOptions::default());
}

pub struct CardEditor {
	/// File path
	file_path: Option<PathBuf>,

	/// Loaded game
	loaded_game: Option<LoadedGame>,

	/// Card search
	card_search: String,

	/// All selected edit screens
	open_edit_screens: Vec<EditScreen>,

	/// swap screen
	swap_screen: Option<SwapScreen>,
}

impl CardEditor {
	/// Card table offset
	pub const CARD_TABLE_OFFSET: u64 = 0x216d000;
	/// Card table size
	pub const CARD_TABLE_SIZE: u64 = 0x14958;

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

	/// Saves the card table to file
	pub fn save_card_table(file_path: &Path, card_table: &CardTable) -> Result<(), anyhow::Error> {
		// Open the file
		let file = fs::File::with_options()
			.write(true)
			.open(file_path)
			.context("Unable to open file")?;
		let mut file = dcb_cdrom_xa::CdRomCursor::new(file);

		// Seek to the card file position and limit our writing to the file size
		file.seek(io::SeekFrom::Start(Self::CARD_TABLE_OFFSET))
			.context("Unable to seek to card table")?;
		let mut file = dcb_util::WriteTake::new(file, Self::CARD_TABLE_SIZE);

		// Then serialize it
		card_table.serialize(&mut file).context("Unable to serialize table")?;

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

	/// Swaps two cards in the card table
	pub fn swap_cards(card_table: &mut CardTable, lhs_idx: usize, rhs_idx: usize) {
		let digimon_idxs = Self::digimon_idxs(card_table);
		let item_idxs = Self::item_idxs(card_table);
		let digivolve_idxs = Self::digivolve_idxs(card_table);
		let digimons_len = card_table.digimons.len();
		let items_len = card_table.items.len();


		if digimon_idxs.contains(&lhs_idx) && digimon_idxs.contains(&rhs_idx) {
			card_table.digimons.swap(lhs_idx, rhs_idx);
		} else if item_idxs.contains(&lhs_idx) && item_idxs.contains(&rhs_idx) {
			card_table.items.swap(lhs_idx - digimons_len, rhs_idx - digimons_len);
		} else if digivolve_idxs.contains(&lhs_idx) && digivolve_idxs.contains(&rhs_idx) {
			card_table
				.digivolves
				.swap(lhs_idx - digimons_len - items_len, rhs_idx - digimons_len - items_len);
		} else {
			panic!("Invalid indexes {} & {}", lhs_idx, rhs_idx);
		}
	}
}

impl Default for CardEditor {
	fn default() -> Self {
		Self {
			file_path:         None,
			loaded_game:       None,
			card_search:       String::new(),
			open_edit_screens: vec![],
			swap_screen:       None,
		}
	}
}

impl epi::App for CardEditor {
	fn update(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) {
		let Self {
			file_path,
			loaded_game,
			card_search,
			open_edit_screens,
			swap_screen,
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
							match Self::parse_card_table(file_path) {
								Ok(card_table) => {
									let hash = self::hash_of(&card_table);
									*loaded_game = Some(LoadedGame {
										card_table,
										saved_card_table_hash: hash,
									});
								},
								Err(err) => MessageDialog::new()
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
								match Self::save_card_table(file_path, &loaded_game.card_table) {
									// After saving, update our hash
									Ok(()) => {
										loaded_game.saved_card_table_hash = self::hash_of(&loaded_game.card_table);
										MessageDialog::new()
											.set_text("Successfully saved!")
											.set_type(MessageType::Info)
											.show_alert()
											.expect("Unable to alert user");
									},
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

				egui::menu::menu(ui, "Edit", |ui| {
					if loaded_game.is_some() && ui.button("Swap").clicked() {
						*swap_screen = Some(SwapScreen {
							card_type: CardType::Digimon,
							lhs_idx:   0,
							rhs_idx:   0,
						});
					}
				});
			});
		});

		// Draw swap screen
		if let (Some(screen), Some(loaded_game)) = (swap_screen.as_mut(), loaded_game.as_mut()) {
			let mut close = false;
			egui::Window::new("Swap screen").show(ctx, |ui| {
				ui.horizontal(|ui| {
					ui.label("Card type");
					self::render_card_type(ui, &mut screen.card_type);
				});

				let range = match screen.card_type {
					CardType::Digimon => Self::digimon_idxs(&loaded_game.card_table),
					CardType::Item => Self::item_idxs(&loaded_game.card_table),
					CardType::Digivolve => Self::digivolve_idxs(&loaded_game.card_table),
				};
				screen.lhs_idx = screen.lhs_idx.clamp(range.start, range.end - 1);
				screen.rhs_idx = screen.rhs_idx.clamp(range.start, range.end - 1);
				let range = range.start..=(range.end - 1);

				ui.horizontal(|ui| {
					ui.label("Left");
					ui.add(egui::Slider::new(&mut screen.lhs_idx, range.clone()));
				});
				ui.horizontal(|ui| {
					ui.label("Right");
					ui.add(egui::Slider::new(&mut screen.rhs_idx, range));
				});
				if ui.button("Swap").clicked() {
					Self::swap_cards(&mut loaded_game.card_table, screen.lhs_idx, screen.rhs_idx);
					close = true;
				}
			});

			if close {
				*swap_screen = None;
			}
		}

		egui::SidePanel::left("side_panel").show(ctx, |ui| {
			ui.heading("Card list");

			ui.vertical(|ui| {
				ui.label("Search");
				ui.text_edit_singleline(card_search);
			});

			// If we have a loaded game, display all cards
			if let Some(loaded_game) = &loaded_game {
				let names = loaded_game
					.card_table
					.digimons
					.iter()
					.map(|digimon| digimon.name.as_str())
					.chain(loaded_game.card_table.items.iter().map(|item| item.name.as_str()))
					.chain(
						loaded_game
							.card_table
							.digivolves
							.iter()
							.map(|digivolve| digivolve.name.as_str()),
					)
					.enumerate()
					.map(|(idx, name)| (idx, format!("{idx}. {name}")))
					.filter(|(_, name)| self::contains_case_insensitive(name, card_search));

				egui::ScrollArea::auto_sized().show(ui, |ui| {
					for (idx, name) in names {
						// If clicked, open/close a new screen
						let screen_idx = open_edit_screens.iter().position(|screen| screen.card_idx == idx);
						if ui.selectable_label(screen_idx.is_some(), name).clicked() {
							match screen_idx {
								Some(screen_idx) => {
									open_edit_screens.remove(screen_idx);
								},
								None => open_edit_screens.push(EditScreen { card_idx: idx }),
							}
						}
					}
				});
			}
		});

		// For every screen, display it
		egui::CentralPanel::default().show(ctx, |ui| {
			let screens_len = open_edit_screens.len();
			for screen in open_edit_screens {
				let card = Self::get_card_from_idx(
					&mut loaded_game
						.as_mut()
						.expect("Had a selected card without a card table")
						.card_table,
					screen.card_idx,
				);

				let total_available_width = ui.available_width();
				let default_width = total_available_width / (screens_len as f32);
				egui::SidePanel::left((screen as *const _, "panel", default_width.to_bits()))
					.default_width(default_width)
					.show(ctx, |ui| {
						// Header for the card
						ui.vertical(|ui| {
							ui.heading(card.name());
							ui.label(match card {
								Card::Digimon(_) => "Digimon",
								Card::Item(_) => "Item",
								Card::Digivolve(_) => "Digivolve",
							});
							ui.separator();
						});

						egui::ScrollArea::auto_sized().show(ui, |ui| {
							self::render_card(ui, card);
						});
					});
			}
		});
	}

	fn on_exit(&mut self) {
		// Ask user if they want to save before leaving if they had any changes
		let wants_to_save = match (&self.file_path, &self.loaded_game) {
			(Some(_), Some(loaded_game))
				if self::hash_of(&loaded_game.card_table) != loaded_game.saved_card_table_hash =>
			{
				MessageDialog::new()
					.set_text("Do you want to save?")
					.set_type(MessageType::Warning)
					.show_confirm()
					.expect("Unable to ask user for confirmation")
			},

			// If we have no file or card table wasn't loaded, user won't want to save
			_ => false,
		};

		if wants_to_save {
			let file_path = self.file_path.as_ref().expect("No file path was set");
			let card_table = &self.loaded_game.as_ref().expect("No card table").card_table;

			match Self::save_card_table(file_path, card_table) {
				Ok(()) => MessageDialog::new()
					.set_text("Successfully saved!")
					.set_type(MessageType::Info)
					.show_alert()
					.expect("Unable to alert user"),
				// If unable to save, save the state to disk just in case changes are lost
				// TODO: Be able to load these backup files up
				Err(err) => {
					// Create backup
					let time = SystemTime::now()
						.duration_since(SystemTime::UNIX_EPOCH)
						.as_ref()
						.map_or(u64::MAX, Duration::as_secs);
					let file = fs::File::create(&format!("cards-{time}.bak")).expect("Unable to create backup file");
					serde_yaml::to_writer(
						file,
						&self.loaded_game.as_ref().map(|loaded_game| &loaded_game.card_table),
					)
					.expect("Unable to write back up to file");

					MessageDialog::new()
						.set_text(&format!("Unable to save file: {:?}\n\nBackup made in file.", err))
						.set_type(MessageType::Error)
						.show_alert()
						.expect("Unable to alert user");
				},
			}
		}
	}

	fn name(&self) -> &str {
		"Dcb card editor"
	}
}

/// An edit screen
pub struct EditScreen {
	/// Currently selected card
	card_idx: usize,
}

/// A swap screen
pub struct SwapScreen {
	/// Card type
	card_type: CardType,

	/// Left idx
	lhs_idx: usize,

	/// Right idx
	rhs_idx: usize,
}

/// Loaded game
pub struct LoadedGame {
	/// Card table
	card_table: CardTable,

	/// Original hash
	saved_card_table_hash: u64,
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


/// Renders a card
fn render_card(ui: &mut egui::Ui, card: Card) {
	match card {
		Card::Digimon(digimon) => self::render_digimon_card(ui, digimon),
		Card::Item(item) => self::render_item_card(ui, item),
		Card::Digivolve(digivolve) => self::render_digivolve_card(ui, digivolve),
	}

	// Add some space at bottom for cut-off stuff at the bottom
	ui.add_space(400.0);
}

/// Renders a digimon card
fn render_digimon_card(ui: &mut egui::Ui, digimon: &mut dcb::Digimon) {
	// Name
	ui.horizontal(|ui| {
		ui.label("Name");
		ui.text_edit_singleline(AsciiTextBuffer::ref_cast_mut(&mut digimon.name));
	});

	// Speciality
	ui.horizontal(|ui| {
		ui.label("Speciality");
		self::render_speciality(ui, &mut digimon.speciality);
	});

	ui.horizontal(|ui| {
		ui.label("Level");
		self::render_level(ui, &mut digimon.level);
	});

	ui.horizontal(|ui| {
		ui.label("HP");
		ui.add(egui::Slider::new(&mut digimon.hp, 0..=5000));
	});
	ui.horizontal(|ui| {
		ui.label("DP");
		ui.add(egui::Slider::new(&mut digimon.dp_cost, 0..=100));
	});
	ui.horizontal(|ui| {
		ui.label("+P");
		ui.add(egui::Slider::new(&mut digimon.dp_give, 0..=100));
	});

	// Moves
	ui.group(|ui| {
		ui.heading("Moves");

		for (name, mv) in std::array::IntoIter::new([
			("Circle", &mut digimon.move_circle),
			("Triangle", &mut digimon.move_triangle),
			("Cross", &mut digimon.move_cross),
		]) {
			self::render_move(ui, name, mv);
		}
	});

	// Cross move effect
	ui.group(|ui| {
		ui.label("Cross move effect");
		self::render_cross_move_effect_opt(ui, &mut digimon.cross_move_effect);
	});

	ui.group(|ui| {
		ui.label("Effect description");
		for line in &mut digimon.effect_description {
			ui.text_edit_singleline(AsciiTextBuffer::ref_cast_mut(line));
		}
	});

	ui.group(|ui| {
		ui.label("Effect arrow color");
		self::render_arrow_color_opt(ui, &mut digimon.effect_arrow_color);
	});

	ui.group(|ui| {
		ui.label("Effect conditions");
		for cond in &mut digimon.effect_conditions {
			self::render_effect_condition_opt(ui, cond);
		}
	});

	ui.group(|ui| {
		ui.label("Effects");
		for effect in &mut digimon.effects {
			self::render_effect_opt(ui, effect);
		}
	});
}

/// Renders an item card
fn render_item_card(ui: &mut egui::Ui, item: &mut dcb::Item) {
	// Name
	ui.horizontal(|ui| {
		ui.label("Name");
		ui.text_edit_singleline(AsciiTextBuffer::ref_cast_mut(&mut item.name))
			.changed();
	});

	ui.group(|ui| {
		ui.label("Effect description");
		for line in &mut item.effect_description {
			ui.text_edit_singleline(AsciiTextBuffer::ref_cast_mut(line));
		}
	});

	ui.group(|ui| {
		ui.label("Effect arrow color");
		self::render_arrow_color_opt(ui, &mut item.effect_arrow_color);
	});

	ui.group(|ui| {
		ui.label("Effect conditions");
		for cond in &mut item.effect_conditions {
			self::render_effect_condition_opt(ui, cond);
		}
	});

	ui.group(|ui| {
		ui.label("Effects");
		for effect in &mut item.effects {
			self::render_effect_opt(ui, effect);
		}
	});
}

/// Renders a digivolve card
fn render_digivolve_card(ui: &mut egui::Ui, digivolve: &mut dcb::Digivolve) {
	// Name
	ui.horizontal(|ui| {
		ui.label("Name");
		ui.text_edit_singleline(AsciiTextBuffer::ref_cast_mut(&mut digivolve.name))
			.changed();
	});

	ui.group(|ui| {
		ui.label("Effect description");
		for line in &mut digivolve.effect_description {
			ui.text_edit_singleline(AsciiTextBuffer::ref_cast_mut(line));
		}
	});

	ui.label("Effect");
	self::render_digivolve_effect(ui, &mut digivolve.effect);
}

/// Displays an optional cross move effect
fn render_cross_move_effect_opt(ui: &mut egui::Ui, cross_move_effect: &mut Option<CrossMoveEffect>) {
	ui.horizontal(|ui| {
		// Show the effect generically
		egui::ComboBox::from_id_source("cross_move_effect")
			.selected_text(cross_move_effect.map_or("None", CrossMoveEffect::as_str))
			.show_ui(ui, |ui| {
				const ATTACK_TO_ZERO_DEFAULT: CrossMoveEffect = CrossMoveEffect::AttackToZero(AttackType::Circle);
				const COUNTER_DEFAULT: CrossMoveEffect = CrossMoveEffect::Counter(AttackType::Circle);
				const TRIPLE_AGAINST_DEFAULT: CrossMoveEffect = CrossMoveEffect::TripleAgainst(Speciality::Darkness);

				let is_attack_to_zero = cross_move_effect.map_or(false, CrossMoveEffect::is_attack_to_zero);
				let is_counter = cross_move_effect.map_or(false, CrossMoveEffect::is_counter);
				let is_triple_against = cross_move_effect.map_or(false, CrossMoveEffect::is_triple_against);

				ui.selectable_value(
					cross_move_effect,
					Some(CrossMoveEffect::AttackFirst),
					CrossMoveEffect::AttackFirst.as_str(),
				);

				if ui
					.selectable_label(is_attack_to_zero, ATTACK_TO_ZERO_DEFAULT.as_str())
					.clicked() && !is_attack_to_zero
				{
					*cross_move_effect = Some(ATTACK_TO_ZERO_DEFAULT);
				}

				if ui.selectable_label(is_counter, COUNTER_DEFAULT.as_str()).clicked() && !is_counter {
					*cross_move_effect = Some(COUNTER_DEFAULT);
				}

				ui.selectable_value(
					cross_move_effect,
					Some(CrossMoveEffect::Crash),
					CrossMoveEffect::Crash.as_str(),
				);
				ui.selectable_value(
					cross_move_effect,
					Some(CrossMoveEffect::EatUpHP),
					CrossMoveEffect::EatUpHP.as_str(),
				);
				ui.selectable_value(
					cross_move_effect,
					Some(CrossMoveEffect::Jamming),
					CrossMoveEffect::Jamming.as_str(),
				);

				if ui
					.selectable_label(is_triple_against, TRIPLE_AGAINST_DEFAULT.as_str())
					.clicked() && !is_triple_against
				{
					*cross_move_effect = Some(TRIPLE_AGAINST_DEFAULT);
				}
			});

		// Then display extra arguments
		match cross_move_effect {
			Some(CrossMoveEffect::AttackToZero(attack_type)) | Some(CrossMoveEffect::Counter(attack_type)) => {
				self::render_attack_type(ui, attack_type)
			},
			Some(CrossMoveEffect::TripleAgainst(speciality)) => self::render_speciality(ui, speciality),
			_ => (),
		};
	});
}

/// Displays an attack type
fn render_attack_type(ui: &mut egui::Ui, cur_attack_type: &mut AttackType) {
	egui::ComboBox::from_id_source(cur_attack_type as *const _)
		.selected_text(cur_attack_type.as_str())
		.show_ui(ui, |ui| {
			for attack_type in AttackType::iter() {
				ui.selectable_value(cur_attack_type, attack_type, attack_type.as_str());
			}
		});
}

/// Displays a speciality
fn render_speciality(ui: &mut egui::Ui, cur_speciality: &mut Speciality) {
	egui::ComboBox::from_id_source(cur_speciality as *const _)
		.selected_text(cur_speciality.as_str())
		.show_ui(ui, |ui| {
			for speciality in Speciality::iter() {
				ui.selectable_value(cur_speciality, speciality, speciality.as_str());
			}
		});
}

/// Displays a level
fn render_level(ui: &mut egui::Ui, cur_level: &mut Level) {
	egui::ComboBox::from_id_source(cur_level as *const _)
		.selected_text(cur_level.as_str())
		.show_ui(ui, |ui| {
			for level in Level::iter() {
				ui.selectable_value(cur_level, level, level.as_str());
			}
		});
}

/// Displays an optional arrow color
fn render_arrow_color_opt(ui: &mut egui::Ui, cur_color: &mut Option<ArrowColor>) {
	let to_str = |color: Option<ArrowColor>| color.map_or("None", ArrowColor::as_str);
	egui::ComboBox::from_id_source(cur_color as *const _)
		.selected_text(to_str(*cur_color))
		.show_ui(ui, |ui| {
			for color in ArrowColor::iter().map(Some).chain(std::iter::once(None)) {
				ui.selectable_value(cur_color, color, to_str(color));
			}
		});
}

/// Displays an effect condition operation
fn render_effect_condition_operation(ui: &mut egui::Ui, cur_op: &mut EffectConditionOperation) {
	egui::ComboBox::from_id_source(cur_op as *const _)
		.selected_text(cur_op.as_str())
		.show_ui(ui, |ui| {
			for op in EffectConditionOperation::iter() {
				ui.selectable_value(cur_op, op, op.as_str());
			}
		});
}

/// Displays an effect operation
fn render_effect_operation(ui: &mut egui::Ui, cur_op: &mut EffectOperation) {
	egui::ComboBox::from_id_source(cur_op as *const _)
		.selected_text(cur_op.as_str())
		.show_ui(ui, |ui| {
			for op in EffectOperation::iter() {
				ui.selectable_value(cur_op, op, op.as_str());
			}
		});
}

/// Displays a player type
fn render_player_type(ui: &mut egui::Ui, cur_player: &mut PlayerType) {
	egui::ComboBox::from_id_source(cur_player as *const _)
		.selected_text(cur_player.as_str())
		.show_ui(ui, |ui| {
			for player in PlayerType::iter() {
				ui.selectable_value(cur_player, player, player.as_str());
			}
		});
}

/// Displays a card type
fn render_card_type(ui: &mut egui::Ui, cur_card: &mut CardType) {
	egui::ComboBox::from_id_source(cur_card as *const _)
		.selected_text(cur_card.as_str())
		.show_ui(ui, |ui| {
			for card in CardType::iter() {
				ui.selectable_value(cur_card, card, card.as_str());
			}
		});
}

/// Displays a slot
fn render_slot(ui: &mut egui::Ui, cur_slot: &mut Slot) {
	egui::ComboBox::from_id_source(cur_slot as *const _)
		.selected_text(cur_slot.as_str())
		.show_ui(ui, |ui| {
			for slot in Slot::iter() {
				ui.selectable_value(cur_slot, slot, slot.as_str());
			}
		});
}

/// Displays a digivolve effect
fn render_digivolve_effect(ui: &mut egui::Ui, cur_effect: &mut DigivolveEffect) {
	egui::ComboBox::from_id_source(cur_effect as *const _)
		.selected_text(cur_effect.as_str())
		.show_ui(ui, |ui| {
			for effect in DigivolveEffect::iter() {
				ui.selectable_value(cur_effect, effect, effect.as_str());
			}
		});
}

/// Displays a digimon property
fn render_digimon_property(ui: &mut egui::Ui, cur_property: &mut DigimonProperty) {
	// Note: Only one search menu is up at a time, so this is fine.
	static SEARCH: SyncLazy<Mutex<String>> = SyncLazy::new(Mutex::default);
	let mut search = SEARCH.lock().expect("Poisoned");

	let response = egui::ComboBox::from_id_source(cur_property as *const _)
		.selected_text(cur_property.as_str())
		.show_ui(ui, |ui| {
			ui.label("Search");
			ui.text_edit_singleline(&mut *search);
			ui.separator();

			let properties = DigimonProperty::iter()
				.map(|property| (property, property.as_str()))
				.filter(|(_, name)| self::contains_case_insensitive(name, &*search));

			for (property, name) in properties {
				ui.selectable_value(cur_property, property, name);
			}
		});

	// If we no longer have focus, reset the search
	if response.clicked_elsewhere() {
		search.clear();
	}
}

/// Displays an optional digimon property
fn render_digimon_property_opt(ui: &mut egui::Ui, cur_property: &mut Option<DigimonProperty>) {
	// Note: Only one search menu is up at a time, so this is fine.
	static SEARCH: SyncLazy<Mutex<String>> = SyncLazy::new(Mutex::default);
	let mut search = SEARCH.lock().expect("Poisoned");

	const TO_STR: fn(Option<DigimonProperty>) -> &'static str = |color| color.map_or("None", DigimonProperty::as_str);
	let response = egui::ComboBox::from_id_source(cur_property as *const _)
		.selected_text(TO_STR(*cur_property))
		.show_ui(ui, |ui| {
			ui.label("Search");
			ui.text_edit_singleline(&mut *search);
			ui.separator();

			let properties = DigimonProperty::iter()
				.map(Some)
				.chain(std::iter::once(None))
				.map(|property| (property, TO_STR(property)))
				.filter(|(_, name)| self::contains_case_insensitive(name, &*search));

			for (property, name) in properties {
				ui.selectable_value(cur_property, property, name);
			}
		});

	// If we no longer have focus, reset the search
	if response.clicked_elsewhere() {
		search.clear();
	}
}

/// Displays a move
fn render_move(ui: &mut egui::Ui, name: &str, mv: &mut Move) {
	// Validate the move so we can display warnings
	let mut warn_power_multiple_of_10 = false;
	mv.validate(|event: Either<_, !>| match event.unwrap_left() {
		dcb::card::property::moves::ValidationWarning::PowerMultiple10 => warn_power_multiple_of_10 = true,
	});

	ui.group(|ui| {
		ui.vertical(|ui| {
			ui.heading(name);
			ui.horizontal(|ui| {
				ui.label("Name");
				ui.text_edit_singleline(AsciiTextBuffer::ref_cast_mut(&mut mv.name));
			});
			ui.horizontal(|ui| {
				ui.label("Power");
				ui.add(egui::Slider::new(&mut mv.power, 0..=2000));
				if warn_power_multiple_of_10 {
					ui.label("Warning: Power should be a multiple of 10");
				}
			});
		});
	});
}

/// Displays an optional effect condition
fn render_effect_condition_opt(ui: &mut egui::Ui, cur_cond: &mut Option<EffectCondition>) {
	ui.group(|ui| match cur_cond {
		Some(cond) => {
			// Calculate what the condition is doing to display to the user
			// TODO: Improve this once the effects are actually figured out properly
			let explanation = {
				let op = cond.operation.operator_str();
				let property_cmp = cond.property_cmp.as_str();
				let arg_property = cond.arg_property.map_or("0", DigimonProperty::as_str);
				let arg_num = cond.arg_num;
				match cond.operation.targets_property() {
					true => format!("{property_cmp} {op} {arg_property}"),
					false => format!("{property_cmp} {op} {arg_num}"),
				}
			};
			ui.heading(explanation);

			ui.checkbox(&mut cond.misfire, "Misfire");

			ui.label("Property cmp");
			self::render_digimon_property(ui, &mut cond.property_cmp);

			ui.label("Arg property");
			self::render_digimon_property_opt(ui, &mut cond.arg_property);

			ui.horizontal(|ui| {
				ui.label("Number property");
				ui.add(egui::Slider::new(&mut cond.arg_num, 0..=10));
			});

			ui.label("Operation");
			self::render_effect_condition_operation(ui, &mut cond.operation);

			ui.separator();
			if ui.button("Remove").clicked() {
				*cur_cond = None;
			}
		},
		None => {
			ui.label("None");
			if ui.button("Add").clicked() {
				*cur_cond = Some(EffectCondition {
					misfire:      false,
					property_cmp: DigimonProperty::OwnSpeciality,
					arg_property: None,
					arg_num:      0,
					operation:    EffectConditionOperation::DifferentFromNumber,
				});
			}
		},
	});
}

/// Displays an optional effect
fn render_effect_opt(ui: &mut egui::Ui, effect: &mut Option<Effect>) {
	ui.group(|ui| {
		match effect {
			Some(Effect::ChangeProperty {
				property,
				a,
				b,
				c,
				x,
				y,
				op,
			}) => {
				// Calculate what the condition is doing to display to the user
				// `<property> = ( <A> + <Y> ) + ( <C> <op> ( <B> + <X> ) )`
				let explanation = match (&*property, &*a, &*y, &*c, &*op, &*b, &*x) {
					// `property += <c> <op> <x>`
					(property, Some(a), 0, Some(c), op, None, x) if a == property => {
						let op = op.operator_str();
						format!("{property} += {c} {op} {x}")
					},

					// `property <op> = <b>`
					(property, None, 0, Some(c), op, Some(b), 0) if property == c => {
						let op = op.operator_str();
						format!("{property} {op}= {b}")
					},

					// `property <op> = <x>`
					(property, None, 0, Some(c), op, None, x) if c == property => {
						let op = op.operator_str();
						format!("{property} {op}= {x}")
					},

					// `property = <c>`
					(property, None, 0, Some(c), EffectOperation::Addition, None, 0) => {
						format!("{property} = {c}")
					},

					// `property = <x | y>`
					(property, None, 0, None, EffectOperation::Addition, None, num) |
					(property, None, num, None, EffectOperation::Addition, None, 0) => {
						format!("{property} = {num}")
					},

					// Else just fully format it
					_ => {
						let a = a.map_or("0", DigimonProperty::as_str);
						let b = b.map_or("0", DigimonProperty::as_str);
						let c = c.map_or("0", DigimonProperty::as_str);
						let op = op.operator_str();

						format!("{property} = ({a} + {y}) + {c} {op} ({b} + {x})")
					},
				};
				ui.heading(explanation);

				ui.vertical(|ui| {
					ui.label("Property");
					self::render_digimon_property(ui, property);

					ui.label("A");
					self::render_digimon_property_opt(ui, a);
					ui.label("B");
					self::render_digimon_property_opt(ui, b);
					ui.label("C");
					self::render_digimon_property_opt(ui, c);

					ui.horizontal(|ui| {
						ui.label("X");
						ui.add(egui::Slider::new(x, 0..=500));
					});
					ui.horizontal(|ui| {
						ui.label("Y");
						ui.add(egui::Slider::new(y, 0..=500));
					});

					ui.label("Operation");
					self::render_effect_operation(ui, op);
				});
			},
			Some(Effect::UseAttack { player, attack }) => {
				ui.heading("Use attack");

				ui.label("Player");
				self::render_player_type(ui, player);

				ui.label("Attack");
				self::render_attack_type(ui, attack);
			},
			Some(Effect::MoveCards {
				player,
				source,
				destination,
				count,
			}) => {
				ui.heading("Move cards");

				ui.label("Player");
				self::render_player_type(ui, player);

				ui.label("Source");
				self::render_slot(ui, source);

				ui.label("Destination");
				self::render_slot(ui, destination);

				ui.horizontal(|ui| {
					ui.label("Count");
					ui.add(egui::Slider::new(count, 0..=40));
				});
			},
			Some(Effect::ShuffleOnlineDeck { player }) => {
				ui.heading("Shuffle online deck");

				ui.label("Player");
				self::render_player_type(ui, player);
			},
			Some(Effect::VoidOpponentSupportEffect) => {
				ui.heading("Void opponent support effect");
			},
			Some(Effect::VoidOpponentSupportOptionEffect) => {
				ui.heading("Void opponent support option effect");
			},
			Some(Effect::PickPartnerCard) => {
				ui.heading("Pick partner card");
			},
			Some(Effect::CycleOpponentAttackType) => {
				ui.heading("Cycle opponent attack type");
			},
			Some(Effect::KoDigimonRevives { health }) => {
				ui.heading("Ko'd digimon revives");

				ui.horizontal(|ui| {
					ui.label("Health");
					ui.add(egui::Slider::new(health, 0..=2000));
				});
			},
			Some(Effect::DrawCards { player, count }) => {
				ui.heading("Draw cards");

				ui.label("Player");
				self::render_player_type(ui, player);

				ui.horizontal(|ui| {
					ui.label("Count");
					ui.add(egui::Slider::new(count, 0..=40));
				});
			},
			Some(Effect::OwnAttackBecomesEatUpHP) => {
				ui.heading("Own attack becomes eat up hp");
			},
			Some(Effect::AttackFirst { player }) => {
				ui.heading("Attack first");

				ui.label("Player");
				self::render_player_type(ui, player);
			},
			None => {
				ui.label("None");
			},
		}

		ui.separator();
		ui.label("Change");
		egui::ComboBox::from_id_source(effect as *const _)
			.selected_text(effect.map_or("None", Effect::as_str))
			.show_ui(ui, |ui| {
				const CHANGE_PROPERTY_DEFAULT: Effect = Effect::ChangeProperty {
					property: DigimonProperty::OwnSpeciality,
					a:        None,
					b:        None,
					c:        None,
					x:        0,
					y:        0,
					op:       EffectOperation::Addition,
				};
				const USE_ATTACK_DEFAULT: Effect = Effect::UseAttack {
					player: PlayerType::Player,
					attack: AttackType::Circle,
				};
				const MOVE_CARDS_DEFAULT: Effect = Effect::MoveCards {
					player:      PlayerType::Player,
					source:      Slot::Hand,
					destination: Slot::Offline,
					count:       0,
				};
				const SHUFFLE_ONLINE_DECK_DEFAULT: Effect = Effect::ShuffleOnlineDeck {
					player: PlayerType::Player,
				};
				const KO_DIGIMON_REVIVES_DEFAULT: Effect = Effect::KoDigimonRevives { health: 0 };
				const DRAW_CARDS_DEFAULT: Effect = Effect::DrawCards {
					player: PlayerType::Player,
					count:  0,
				};
				const ATTACK_FIRST_DEFAULT: Effect = Effect::AttackFirst {
					player: PlayerType::Player,
				};

				let is_change_property = effect.map_or(false, Effect::is_change_property);
				let is_use_attack = effect.map_or(false, Effect::is_use_attack);
				let is_move_cards = effect.map_or(false, Effect::is_move_cards);
				let is_shuffle_online_deck = effect.map_or(false, Effect::is_shuffle_online_deck);
				let is_ko_digimon_revives = effect.map_or(false, Effect::is_ko_digimon_revives);
				let is_draw_cards = effect.map_or(false, Effect::is_draw_cards);
				let is_attack_first = effect.map_or(false, Effect::is_attack_first);

				if ui
					.selectable_label(is_change_property, CHANGE_PROPERTY_DEFAULT.as_str())
					.clicked() && !is_change_property
				{
					*effect = Some(CHANGE_PROPERTY_DEFAULT);
				}
				if ui
					.selectable_label(is_use_attack, USE_ATTACK_DEFAULT.as_str())
					.clicked() && !is_use_attack
				{
					*effect = Some(USE_ATTACK_DEFAULT);
				}
				if ui
					.selectable_label(is_move_cards, MOVE_CARDS_DEFAULT.as_str())
					.clicked() && !is_move_cards
				{
					*effect = Some(MOVE_CARDS_DEFAULT);
				}
				if ui
					.selectable_label(is_shuffle_online_deck, SHUFFLE_ONLINE_DECK_DEFAULT.as_str())
					.clicked() && !is_shuffle_online_deck
				{
					*effect = Some(SHUFFLE_ONLINE_DECK_DEFAULT);
				}

				ui.selectable_value(
					effect,
					Some(Effect::VoidOpponentSupportEffect),
					Effect::VoidOpponentSupportEffect.as_str(),
				);
				ui.selectable_value(
					effect,
					Some(Effect::VoidOpponentSupportOptionEffect),
					Effect::VoidOpponentSupportOptionEffect.as_str(),
				);
				ui.selectable_value(effect, Some(Effect::PickPartnerCard), Effect::PickPartnerCard.as_str());
				ui.selectable_value(
					effect,
					Some(Effect::CycleOpponentAttackType),
					Effect::CycleOpponentAttackType.as_str(),
				);

				if ui
					.selectable_label(is_ko_digimon_revives, KO_DIGIMON_REVIVES_DEFAULT.as_str())
					.clicked() && !is_ko_digimon_revives
				{
					*effect = Some(KO_DIGIMON_REVIVES_DEFAULT);
				}
				if ui
					.selectable_label(is_draw_cards, DRAW_CARDS_DEFAULT.as_str())
					.clicked() && !is_draw_cards
				{
					*effect = Some(DRAW_CARDS_DEFAULT);
				}

				ui.selectable_value(
					effect,
					Some(Effect::OwnAttackBecomesEatUpHP),
					Effect::OwnAttackBecomesEatUpHP.as_str(),
				);

				if ui
					.selectable_label(is_attack_first, ATTACK_FIRST_DEFAULT.as_str())
					.clicked() && !is_attack_first
				{
					*effect = Some(ATTACK_FIRST_DEFAULT);
				}
			});
	});
}

/// Calculates the hash of any single value
pub fn hash_of<T: Hash>(value: &T) -> u64 {
	let mut state = DefaultHasher::new();
	value.hash(&mut state);
	state.finish()
}

/// Checks if string `pattern` is contained in `haystack` without
/// checking for case
pub fn contains_case_insensitive(mut haystack: &str, pattern: &str) -> bool {
	loop {
		match haystack.get(..pattern.len()) {
			Some(s) => match s.eq_ignore_ascii_case(pattern) {
				true => return true,
				false => haystack = &haystack[1..],
			},
			None => return false,
		}
	}
}
