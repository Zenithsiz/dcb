//! Card editor

// Features
#![feature(
	array_map,
	with_options,
	format_args_capture,
	once_cell,
	never_type,
	try_blocks,
	hash_drain_filter
)]

// Imports
use anyhow::Context;
use dcb_cdrom_xa::CdRomCursor;
use dcb_drv::cursor::DirEntryCursorKind;
use dcb_io::GameFile;
use eframe::{egui, epi, NativeOptions};
use native_dialog::{FileDialog, MessageDialog, MessageType};
use std::{fs, io::Write, mem, path::PathBuf};

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
	let app = FileEditor::default();
	eframe::run_native(Box::new(app), NativeOptions::default());
}

pub struct FileEditor {
	/// File path
	file_path: Option<PathBuf>,

	/// Loaded game
	loaded_game: Option<LoadedGame>,

	/// File search
	file_search: String,

	/// Swap window
	swap_window: Option<SwapWindow>,
}

impl Default for FileEditor {
	fn default() -> Self {
		Self {
			file_path:   None,
			loaded_game: None,
			file_search: String::new(),
			swap_window: None,
		}
	}
}

impl epi::App for FileEditor {
	fn update(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) {
		let Self {
			file_path,
			loaded_game,
			file_search,
			swap_window,
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

						// Then open the file
						if let Some(file_path) = file_path {
							let res: Result<_, anyhow::Error> = try {
								let file = fs::File::with_options()
									.read(true)
									.write(true)
									.open(file_path)
									.context("Unable to open file")?;
								let cdrom = CdRomCursor::new(file);
								let game_file = GameFile::new(cdrom).context("Unable to load game file")?;
								*loaded_game = Some(LoadedGame { game_file });
							};

							if let Err(err) = res {
								MessageDialog::new()
									.set_text(&format!("Unable to open file: {:?}", err))
									.set_type(MessageType::Error)
									.show_alert()
									.expect("Unable to alert user");
							}
						}
					}

					if ui.button("Quit").clicked() {
						frame.quit();
					}
				});

				egui::menu::menu(ui, "Edit", |ui| {
					if loaded_game.is_some() && ui.button("Swap").clicked() {
						*swap_window = Some(SwapWindow::default())
					}
				});
			});
		});

		egui::SidePanel::left("side_panel", 400.0).show(ctx, |ui| {
			ui.heading("File list");

			ui.vertical(|ui| {
				ui.label("Search");
				ui.text_edit_singleline(file_search);
			});

			// If we have a loaded game, display all files
			if let Some(loaded_game) = loaded_game.as_mut() {
				egui::ScrollArea::auto_sized().show(ui, |ui| {
					fn show_dir(
						dir: &dcb_drv::cursor::DirCursor, dir_path: &str, ui: &mut egui::Ui, file_search: &str,
						swap_window: &mut Option<SwapWindow>,
					) {
						for entry in dir.entries() {
							match &entry.kind() {
								DirEntryCursorKind::Dir(dir) => {
									let dir_path = format!("{dir_path}{}\\", entry.name());
									egui::CollapsingHeader::new(entry.name())
										.id_source(dir as *const _)
										.show(ui, |ui| {
											show_dir(dir, &dir_path, ui, file_search, swap_window);
										});
								},
								DirEntryCursorKind::File(file) => {
									let filename = format!("{}.{}", entry.name(), file.extension());
									let path = format!("{dir_path}{filename}");
									if !self::contains_case_insensitive(&filename, file_search) {
										continue;
									}

									if ui.button(&filename).clicked() {
										if let Some(swap_window) = swap_window {
											if swap_window.first.is_setting() {
												swap_window.first = SwapFileStatus::Set(path.clone());
											}
											if swap_window.second.is_setting() {
												swap_window.second = SwapFileStatus::Set(path);
											}
										}
									}
								},
							}
						}
					}

					let a_dir = loaded_game.game_file.a_drv_cursor().root_dir();
					let b_dir = loaded_game.game_file.b_drv_cursor().root_dir();
					let c_dir = loaded_game.game_file.c_drv_cursor().root_dir();
					let e_dir = loaded_game.game_file.e_drv_cursor().root_dir();
					let f_dir = loaded_game.game_file.f_drv_cursor().root_dir();
					let g_dir = loaded_game.game_file.g_drv_cursor().root_dir();
					let p_dir = loaded_game.game_file.p_drv_cursor().root_dir();
					egui::CollapsingHeader::new("A:\\")
						.show(ui, |ui| show_dir(a_dir, "A:\\", ui, file_search, swap_window));
					egui::CollapsingHeader::new("B:\\")
						.show(ui, |ui| show_dir(b_dir, "B:\\", ui, file_search, swap_window));
					egui::CollapsingHeader::new("C:\\")
						.show(ui, |ui| show_dir(c_dir, "C:\\", ui, file_search, swap_window));
					egui::CollapsingHeader::new("E:\\")
						.show(ui, |ui| show_dir(e_dir, "E:\\", ui, file_search, swap_window));
					egui::CollapsingHeader::new("F:\\")
						.show(ui, |ui| show_dir(f_dir, "F:\\", ui, file_search, swap_window));
					egui::CollapsingHeader::new("G:\\")
						.show(ui, |ui| show_dir(g_dir, "G:\\", ui, file_search, swap_window));
					egui::CollapsingHeader::new("P:\\")
						.show(ui, |ui| show_dir(p_dir, "P:\\", ui, file_search, swap_window));
				});
			}
		});

		if let Some(swap_window) = swap_window {
			egui::Window::new("Swap screen").show(ctx, |ui| {
				ui.horizontal(|ui| {
					ui.label(swap_window.first.as_str().unwrap_or("None"));
					let text = match swap_window.first.is_setting() {
						true => "...",
						false => "Set",
					};
					if ui.button(text).clicked() {
						swap_window.first.toggle();
					}
				});
				ui.horizontal(|ui| {
					ui.label(swap_window.second.as_str().unwrap_or("None"));
					let text = match swap_window.second.is_setting() {
						true => "...",
						false => "Set",
					};
					if ui.button(text).clicked() {
						swap_window.second.toggle();
					}
				});

				if ui.button("Swap").clicked() {
					match (&swap_window.first, &swap_window.second) {
						(SwapFileStatus::Set(_first), SwapFileStatus::Set(_second)) => {
							todo!()
						},
						_ => MessageDialog::new()
							.set_text("You must set both files before swapping")
							.set_type(MessageType::Warning)
							.show_alert()
							.expect("Unable to alert user"),
					}
				}
			});
		}
	}

	fn on_exit(&mut self) {
		// Flush the file if we have it
		if let Some(loaded_game) = &mut self.loaded_game {
			let _ = loaded_game.game_file.cdrom().flush();
		}
	}

	fn name(&self) -> &str {
		"Dcb file editor"
	}
}

/// Swap window
#[derive(PartialEq, Clone, Default)]
pub struct SwapWindow {
	/// First file
	first: SwapFileStatus,

	/// Second file
	second: SwapFileStatus,
}

/// Status of a file being swapped
#[derive(PartialEq, Clone)]
pub enum SwapFileStatus {
	/// Unset
	Unset,

	/// Setting
	Setting(Option<String>),

	/// Set
	Set(String),
}

impl SwapFileStatus {
	/// Toggles the current setting
	pub fn toggle(&mut self) {
		*self = match mem::take(self) {
			Self::Unset => Self::Setting(None),
			Self::Setting(s) => match s {
				Some(s) => Self::Set(s),
				None => Self::Unset,
			},
			Self::Set(s) => Self::Setting(Some(s)),
		};
	}

	/// Returns this status as a string
	pub fn as_str(&self) -> Option<&str> {
		match self {
			Self::Setting(s) => s.as_deref(),
			Self::Set(s) => Some(s),
			_ => None,
		}
	}

	/// Returns `true` if the swap_file_status is [`Setting`].
	pub fn is_setting(&self) -> bool {
		matches!(self, Self::Setting(..))
	}

	/// Returns `true` if the swap_file_status is [`Set`].
	pub fn is_set(&self) -> bool {
		matches!(self, Self::Set(..))
	}

	/// Returns `true` if the swap_file_status is [`Unset`].
	pub fn is_unset(&self) -> bool {
		matches!(self, Self::Unset)
	}
}

impl Default for SwapFileStatus {
	fn default() -> Self {
		Self::Unset
	}
}


/// Loaded game
pub struct LoadedGame {
	/// Game file
	game_file: GameFile<fs::File>,
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
