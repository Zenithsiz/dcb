//! Card editor

// Features
#![feature(array_map, with_options)]

// Modules
pub mod edit_state;

// Exports
pub use edit_state::{CardEditState, DigimonEditState};

// Imports
use anyhow::Context;
use dcb::{
	card::property::{
		ArrowColor, AttackType, CrossMoveEffect, DigimonProperty, Effect, EffectConditionOperation, EffectOperation,
		Level, PlayerType, Slot, Speciality,
	},
	CardTable,
};
use eframe::{egui, epi};
use native_dialog::{FileDialog, MessageDialog, MessageType};
use std::{
	borrow::Cow,
	fs,
	io::{self, Read, Seek},
	path::{Path, PathBuf},
};

fn main() {
	// Crate the app and run it
	let app = CardEditor::default();
	eframe::run_native(Box::new(app));
}

pub struct CardEditor {
	/// File path
	file_path: Option<PathBuf>,

	/// Card table
	card_table: Option<CardTable>,

	/// Card search
	card_search: String,

	/// Currently selected card
	selected_card_idx: Option<usize>,

	/// Card edit state
	cur_card_edit_state: Option<CardEditState>,

	/// Card edit status
	cur_card_edit_status: Option<Cow<'static, str>>,
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

		// Then parse it
		card_table.serialize(&mut file).context("Unable to serialize table")?;

		Ok(())
	}

	/// Returns a card given it's index
	pub fn get_card_from_idx(card_table: &mut CardTable, idx: usize) -> Card {
		let digimons_len = card_table.digimons.len();
		let items_len = card_table.items.len();
		let digivolves_len = card_table.digivolves.len();

		if idx < digimons_len {
			Card::Digimon(&mut card_table.digimons[idx])
		} else if idx < digimons_len + items_len {
			Card::Item(&mut card_table.items[idx - digimons_len])
		} else if idx < digimons_len + items_len + digivolves_len {
			Card::Digivolve(&mut card_table.digivolves[idx - digimons_len - items_len])
		} else {
			panic!("Invalid card index");
		}
	}
}

impl Default for CardEditor {
	fn default() -> Self {
		Self {
			file_path:            None,
			card_table:           None,
			card_search:          String::new(),
			selected_card_idx:    None,
			cur_card_edit_state:  None,
			cur_card_edit_status: None,
		}
	}
}

impl epi::App for CardEditor {
	fn update(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) {
		let Self {
			file_path,
			card_table,
			card_search,
			selected_card_idx,
			cur_card_edit_state,
			cur_card_edit_status,
		} = self;

		// Top panel
		egui::TopPanel::top("top_panel").show(ctx, |ui| {
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
								Ok(table) => *card_table = Some(table),
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
						match (&file_path, &card_table) {
							(Some(file_path), Some(card_table)) => match Self::save_card_table(file_path, card_table) {
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
			});
		});

		egui::SidePanel::left("side_panel", 200.0).show(ctx, |ui| {
			ui.heading("Card list");

			ui.vertical(|ui| {
				ui.label("Search");
				ui.text_edit_singleline(card_search);
			});

			// If we have a card table, display all cards
			if let Some(card_table) = &card_table {
				let names = card_table
					.digimons
					.iter()
					.map(|digimon| digimon.name.as_str())
					.chain(card_table.items.iter().map(|item| item.name.as_str()))
					.chain(card_table.digivolves.iter().map(|digivolve| digivolve.name.as_str()))
					.enumerate()
					.filter(|(_, name)| name.contains(card_search.as_str())); // TODO: Switch to better searching

				egui::ScrollArea::auto_sized().show(ui, |ui| {
					for (idx, name) in names {
						// If clicked, set the selected card index and flush the edits
						if ui.selectable_label(*selected_card_idx == Some(idx), name).clicked() {
							*selected_card_idx = Some(idx);
							*cur_card_edit_state = None;
							*cur_card_edit_status = None;
						}
					}
				});
			}
		});

		// If we have a selected card, show a screen for it
		if let Some(selected_card_idx) = *selected_card_idx {
			let card = Self::get_card_from_idx(
				card_table.as_mut().expect("Had a selected card without a card table"),
				selected_card_idx,
			);

			// Header for the card
			egui::TopPanel::top("card_header_name").show(ctx, |ui| {
				ui.heading(card.name());
				ui.label(match card {
					Card::Digimon(_) => "Digimon",
					Card::Item(_) => "Item",
					Card::Digivolve(_) => "Digivolve",
				})
			});

			egui::CentralPanel::default().show(ctx, |ui| {
				egui::ScrollArea::auto_sized().show(ui, |ui| {
					match card {
						Card::Digimon(digimon) => {
							// Get the current card edit state as digimon
							let edit_state = cur_card_edit_state
								.get_or_insert_with(|| CardEditState::digimon(digimon))
								.as_digimon_mut()
								.expect("Edit state wasn't a digimon when a digimon was selected");

							// Then render it
							self::render_digimon_card(ui, digimon, edit_state, cur_card_edit_status);
						},
						Card::Item(_) | Card::Digivolve(_) => {
							ui.heading("TODO");
						},
					}
				});
			});
		}
	}

	fn on_exit(&mut self) {
		// Ask user if they want to save before leaving
		// TODO: Only do this if we have any changes
		let wants_to_save = match (&self.file_path, &self.card_table) {
			(Some(_), Some(_)) => MessageDialog::new()
				.set_text("Do you want to save?")
				.set_type(MessageType::Warning)
				.show_confirm()
				.expect("Unable to ask user for confirmation"),

			// If we have no file or card table wasn't loaded, user won't want to save
			_ => false,
		};

		if wants_to_save {
			let file_path = self.file_path.as_ref().expect("No file path was set");
			let card_table = self.card_table.as_ref().expect("No card table");

			match Self::save_card_table(file_path, card_table) {
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
		}
	}

	fn name(&self) -> &str {
		"Dcb card editor"
	}
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

/// Renders a digimon card
fn render_digimon_card(
	ui: &mut egui::Ui, digimon: &mut dcb::Digimon, edit_state: &mut DigimonEditState,
	cur_card_edit_status: &mut Option<Cow<'static, str>>,
) {
	// Keeps track if any of the `edit_state` fields were changed, so we can apply the changes
	// at the end
	let mut any_edit_state_changed = false;

	// Name
	ui.horizontal(|ui| {
		ui.label("Name");
		any_edit_state_changed |= ui.text_edit_singleline(&mut edit_state.name).changed();
	});

	// Speciality
	ui.horizontal(|ui| {
		ui.label("Speciality");
		egui::ComboBox::from_id_source("digimon_speciality")
			.selected_text(digimon.speciality.as_str())
			.show_ui(ui, |ui| {
				for &speciality in Speciality::ALL {
					ui.selectable_value(&mut digimon.speciality, speciality, speciality.as_str());
				}
			});
	});

	ui.horizontal(|ui| {
		ui.label("Level");
		egui::ComboBox::from_id_source("digimon_level")
			.selected_text(digimon.level.as_str())
			.show_ui(ui, |ui| {
				for &level in Level::ALL {
					ui.selectable_value(&mut digimon.level, level, level.as_str());
				}
			});
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

	for (name, mv, mv_name) in std::array::IntoIter::new([
		("Circle", &mut digimon.move_circle, &mut edit_state.move_circle_name),
		(
			"Triangle",
			&mut digimon.move_triangle,
			&mut edit_state.move_triangle_name,
		),
		("Cross", &mut digimon.move_cross, &mut edit_state.move_cross_name),
	]) {
		ui.group(|ui| {
			ui.heading(name);
			ui.horizontal(|ui| {
				ui.label("Name");
				any_edit_state_changed |= ui.text_edit_singleline(mv_name).changed();
			});
			ui.horizontal(|ui| {
				ui.label("Power");
				ui.add(egui::Slider::new(&mut mv.power, 0..=2000));
			});
		});
	}

	ui.group(|ui| {
		ui.label("Cross move effect");
		ui.horizontal_wrapped(|ui| {
			for cross_move_effect in std::iter::once(None).chain(CrossMoveEffect::ALL.iter().map(Some)) {
				let text = match cross_move_effect {
					Some(effect) => effect.as_str(),
					None => "None",
				};
				ui.radio_value(&mut digimon.cross_move_effect, cross_move_effect.copied(), text);
			}
		});
	});

	ui.group(|ui| {
		ui.label("Effect description");
		for line in &mut edit_state.effect_description {
			any_edit_state_changed |= ui.text_edit_singleline(line).changed();
		}
	});

	ui.group(|ui| {
		ui.label("Effect arrow color");
		ui.horizontal(|ui| {
			for arrow_color in std::iter::once(None).chain(ArrowColor::ALL.iter().map(Some)) {
				let text = match arrow_color {
					Some(arrow_color) => arrow_color.as_str(),
					None => "None",
				};
				ui.radio_value(&mut digimon.effect_arrow_color, arrow_color.copied(), text);
			}
		});
	});

	ui.group(|ui| {
		ui.label("Effect conditions");
		for cond in &mut digimon.effect_conditions {
			ui.group(|ui| {
				match cond {
					Some(cond) => {
						ui.vertical(|ui| {
							ui.checkbox(&mut cond.misfire, "Misfire");

							ui.label("Property cmp");
							ui.horizontal_wrapped(|ui| {
								for &digimon_property in DigimonProperty::ALL {
									ui.radio_value(&mut cond.property_cmp, digimon_property, digimon_property.as_str());
								}
							});

							ui.label("Arg property");
							ui.horizontal_wrapped(|ui| {
								for digimon_property in
									std::iter::once(None).chain(DigimonProperty::ALL.iter().map(Some))
								{
									let text = match digimon_property {
										Some(property) => property.as_str(),
										None => "None",
									};
									ui.radio_value(&mut cond.arg_property, digimon_property.copied(), text);
								}
							});

							ui.horizontal(|ui| {
								ui.label("Num property");
								ui.add(egui::Slider::new(&mut cond.arg_num, 0..=10));
							});

							ui.label("Operation");
							ui.horizontal_wrapped(|ui| {
								for &operation in EffectConditionOperation::ALL {
									ui.radio_value(&mut cond.operation, operation, operation.as_str());
								}
							});
						});
					},
					None => {
						ui.label("None");
						// TODO: Be able to add new once all fields are figured out.
					},
				}
			});
		}
	});

	ui.group(|ui| {
		ui.label("Effects");
		for effect in &mut digimon.effects {
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
						ui.heading("Change Property");
						ui.vertical(|ui| {
							ui.label("Property");
							ui.horizontal_wrapped(|ui| {
								for &digimon_property in DigimonProperty::ALL {
									ui.radio_value(property, digimon_property, digimon_property.as_str());
								}
							});

							ui.label("A");
							ui.horizontal_wrapped(|ui| {
								for digimon_property in
									std::iter::once(None).chain(DigimonProperty::ALL.iter().map(Some))
								{
									let text = match digimon_property {
										Some(property) => property.as_str(),
										None => "None",
									};
									ui.radio_value(a, digimon_property.copied(), text);
								}
							});
							ui.label("B");
							ui.horizontal_wrapped(|ui| {
								for digimon_property in
									std::iter::once(None).chain(DigimonProperty::ALL.iter().map(Some))
								{
									let text = match digimon_property {
										Some(property) => property.as_str(),
										None => "None",
									};
									ui.radio_value(b, digimon_property.copied(), text);
								}
							});
							ui.label("C");
							ui.horizontal_wrapped(|ui| {
								for digimon_property in
									std::iter::once(None).chain(DigimonProperty::ALL.iter().map(Some))
								{
									let text = match digimon_property {
										Some(property) => property.as_str(),
										None => "None",
									};
									ui.radio_value(c, digimon_property.copied(), text);
								}
							});

							ui.horizontal(|ui| {
								ui.label("X");
								ui.add(egui::Slider::new(x, 0..=500));
							});
							ui.horizontal(|ui| {
								ui.label("Y");
								ui.add(egui::Slider::new(y, 0..=500));
							});

							ui.label("Operation");
							ui.horizontal_wrapped(|ui| {
								for &operation in EffectOperation::ALL {
									ui.radio_value(op, operation, operation.as_str());
								}
							});
						});
					},
					Some(Effect::UseAttack { player, attack }) => {
						ui.heading("Use attack");

						ui.label("Player");
						ui.horizontal_wrapped(|ui| {
							for &player_type in PlayerType::ALL {
								ui.radio_value(player, player_type, player_type.as_str());
							}
						});

						ui.label("Attack");
						ui.horizontal_wrapped(|ui| {
							for &attack_type in AttackType::ALL {
								ui.radio_value(attack, attack_type, attack_type.as_str());
							}
						});
					},
					Some(Effect::SetTempSlot { a, b, c, op }) => {
						ui.heading("Set temp slot");
						ui.vertical(|ui| {
							ui.label("A");
							ui.horizontal_wrapped(|ui| {
								for digimon_property in
									std::iter::once(None).chain(DigimonProperty::ALL.iter().map(Some))
								{
									let text = match digimon_property {
										Some(property) => property.as_str(),
										None => "None",
									};
									ui.radio_value(a, digimon_property.copied(), text);
								}
							});
							ui.label("B");
							ui.horizontal_wrapped(|ui| {
								for digimon_property in
									std::iter::once(None).chain(DigimonProperty::ALL.iter().map(Some))
								{
									let text = match digimon_property {
										Some(property) => property.as_str(),
										None => "None",
									};
									ui.radio_value(b, digimon_property.copied(), text);
								}
							});
							ui.label("C");
							ui.horizontal_wrapped(|ui| {
								for digimon_property in
									std::iter::once(None).chain(DigimonProperty::ALL.iter().map(Some))
								{
									let text = match digimon_property {
										Some(property) => property.as_str(),
										None => "None",
									};
									ui.radio_value(c, digimon_property.copied(), text);
								}
							});

							ui.label("Operation");
							ui.horizontal_wrapped(|ui| {
								for &operation in EffectOperation::ALL {
									ui.radio_value(op, operation, operation.as_str());
								}
							});
						});
					},
					Some(Effect::MoveCards {
						player,
						source,
						destination,
						count,
					}) => {
						ui.heading("Move cards");

						ui.label("Player");
						ui.horizontal_wrapped(|ui| {
							for &player_type in PlayerType::ALL {
								ui.radio_value(player, player_type, player_type.as_str());
							}
						});

						ui.label("Source");
						ui.horizontal_wrapped(|ui| {
							for &slot in Slot::ALL {
								ui.radio_value(source, slot, slot.as_str());
							}
						});

						ui.label("Destination");
						ui.horizontal_wrapped(|ui| {
							for &slot in Slot::ALL {
								ui.radio_value(destination, slot, slot.as_str());
							}
						});

						ui.horizontal(|ui| {
							ui.label("Count");
							ui.add(egui::Slider::new(count, 0..=40));
						});
					},
					Some(Effect::ShuffleOnlineDeck { player }) => {
						ui.heading("Shuffle online deck");

						ui.label("Player");
						ui.horizontal_wrapped(|ui| {
							for &player_type in PlayerType::ALL {
								ui.radio_value(player, player_type, player_type.as_str());
							}
						});
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
						ui.horizontal_wrapped(|ui| {
							for &player_type in PlayerType::ALL {
								ui.radio_value(player, player_type, player_type.as_str());
							}
						});

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
						ui.horizontal_wrapped(|ui| {
							for &player_type in PlayerType::ALL {
								ui.radio_value(player, player_type, player_type.as_str());
							}
						});
					},
					None => {
						ui.label("None");
						// TODO: Be able to add new once all fields are figured out.
					},
				}
			});
		}
	});


	// Then try to apply if anything was changed
	if any_edit_state_changed {
		let status = match edit_state.apply(digimon) {
			Ok(()) => Cow::Borrowed("All ok"),
			Err(err) => Cow::Owned(format!("Error: {:?}", err)),
		};

		*cur_card_edit_status = Some(status);
	}

	if let Some(cur_card_edit_status) = cur_card_edit_status {
		ui.separator();
		ui.label(&**cur_card_edit_status);
	}
}
