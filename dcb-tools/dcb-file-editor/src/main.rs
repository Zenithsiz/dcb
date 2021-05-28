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

// Modules
pub mod tree;

// Imports
use anyhow::Context;
use dcb_cdrom_xa::CdRomCursor;
use dcb_io::GameFile;
use eframe::{egui, epi, NativeOptions};
use native_dialog::{FileDialog, MessageDialog, MessageType};
use std::{fs, io::Write, mem, path::PathBuf};
use tree::FsTree;

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
								let mut game_file = GameFile::new(cdrom);

								let mut a_reader = game_file.a_drv().context("Unable to get `a` drive")?;
								let a_tree = FsTree::new(&mut a_reader).context("Unable to load `a` drive")?;
								let mut b_reader = game_file.b_drv().context("Unable to get `b` drive")?;
								let b_tree = FsTree::new(&mut b_reader).context("Unable to load `b` drive")?;
								let mut c_reader = game_file.c_drv().context("Unable to get `c` drive")?;
								let c_tree = FsTree::new(&mut c_reader).context("Unable to load `c` drive")?;
								let mut e_reader = game_file.e_drv().context("Unable to get `e` drive")?;
								let e_tree = FsTree::new(&mut e_reader).context("Unable to load `e` drive")?;
								let mut f_reader = game_file.f_drv().context("Unable to get `f` drive")?;
								let f_tree = FsTree::new(&mut f_reader).context("Unable to load `f` drive")?;
								let mut g_reader = game_file.g_drv().context("Unable to get `g` drive")?;
								let g_tree = FsTree::new(&mut g_reader).context("Unable to load `g` drive")?;
								let mut p_reader = game_file.p_drv().context("Unable to get `p` drive")?;
								let p_tree = FsTree::new(&mut p_reader).context("Unable to load `p` drive")?;

								*loaded_game = Some(LoadedGame {
									game_file,
									a_tree,
									b_tree,
									c_tree,
									e_tree,
									f_tree,
									g_tree,
									p_tree,
								});
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
					let mut ctx = tree::DisplayCtx {
						search_str:    &file_search,
						on_file_click: |path: &str| {
							if let Some(swap_window) = swap_window {
								if swap_window.first.is_setting() {
									swap_window.first = SwapFileStatus::Set(path.to_owned());
								}
								if swap_window.second.is_setting() {
									swap_window.second = SwapFileStatus::Set(path.to_owned());
								}
							}
						},
					};

					egui::CollapsingHeader::new("A:\\").show(ui, |ui| loaded_game.a_tree.display(ui, "A:\\", &mut ctx));
					egui::CollapsingHeader::new("B:\\").show(ui, |ui| loaded_game.b_tree.display(ui, "B:\\", &mut ctx));
					egui::CollapsingHeader::new("C:\\").show(ui, |ui| loaded_game.c_tree.display(ui, "C:\\", &mut ctx));
					egui::CollapsingHeader::new("E:\\").show(ui, |ui| loaded_game.e_tree.display(ui, "E:\\", &mut ctx));
					egui::CollapsingHeader::new("F:\\").show(ui, |ui| loaded_game.f_tree.display(ui, "F:\\", &mut ctx));
					egui::CollapsingHeader::new("G:\\").show(ui, |ui| loaded_game.g_tree.display(ui, "G:\\", &mut ctx));
					egui::CollapsingHeader::new("P:\\").show(ui, |ui| loaded_game.p_tree.display(ui, "P:\\", &mut ctx));
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

	/// `A` drive tree
	a_tree: FsTree,

	/// `B` drive tree
	b_tree: FsTree,

	/// `C` drive tree
	c_tree: FsTree,

	/// `E` drive tree
	e_tree: FsTree,

	/// `F` drive tree
	f_tree: FsTree,

	/// `G` drive tree
	g_tree: FsTree,

	/// `P` drive tree
	p_tree: FsTree,
}
