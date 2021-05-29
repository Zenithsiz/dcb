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
use dcb_io::{game_file::Path, GameFile};
use dcb_tim::Tim;
use eframe::{
	egui::{self, Color32, TextureId},
	epi, NativeOptions,
};
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

	/// Preview panel
	preview_panel: Option<PreviewPanel>,
}

impl Default for FileEditor {
	fn default() -> Self {
		Self {
			file_path:     None,
			loaded_game:   None,
			file_search:   String::new(),
			swap_window:   None,
			preview_panel: None,
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
			preview_panel,
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

		egui::SidePanel::left("side_panel").show(ctx, |ui| {
			ui.heading("File list");

			ui.vertical(|ui| {
				ui.label("Search");
				ui.text_edit_singleline(file_search);
			});

			// If we have a loaded game, display all files
			if let Some(loaded_game) = loaded_game.as_mut() {
				egui::ScrollArea::auto_sized().show(ui, |ui| {
					let mut preview_path = None;
					let mut display_ctx = tree::DisplayCtx {
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

							preview_path = Some(path.to_owned());
						},
					};

					egui::CollapsingHeader::new("A:\\")
						.show(ui, |ui| loaded_game.a_tree.display(ui, "A:\\", &mut display_ctx));
					egui::CollapsingHeader::new("B:\\")
						.show(ui, |ui| loaded_game.b_tree.display(ui, "B:\\", &mut display_ctx));
					egui::CollapsingHeader::new("C:\\")
						.show(ui, |ui| loaded_game.c_tree.display(ui, "C:\\", &mut display_ctx));
					egui::CollapsingHeader::new("E:\\")
						.show(ui, |ui| loaded_game.e_tree.display(ui, "E:\\", &mut display_ctx));
					egui::CollapsingHeader::new("F:\\")
						.show(ui, |ui| loaded_game.f_tree.display(ui, "F:\\", &mut display_ctx));
					egui::CollapsingHeader::new("G:\\")
						.show(ui, |ui| loaded_game.g_tree.display(ui, "G:\\", &mut display_ctx));
					egui::CollapsingHeader::new("P:\\")
						.show(ui, |ui| loaded_game.p_tree.display(ui, "P:\\", &mut display_ctx));

					if let Some(path) = preview_path {
						let panel: Result<_, anyhow::Error> = match path {
							path if path.ends_with(".TIM") => {
								try {
									// Deserialize the tim
									let path = Path::from_ascii(&path).context("Unable to create path")?;
									let mut file =
										loaded_game.game_file.open_file(path).context("Unable to open file")?;
									let image = Tim::deserialize(&mut file).context("Unable to parse file")?;

									// Then create a texture with it
									let [width, height] = image.size();
									let colors: Box<[_]> = image
										.colors()
										.context("Unable to get image colors")?
										.to_vec()
										.into_iter()
										.map(|[r, g, b, a]| Color32::from_rgba_premultiplied(r, g, b, a))
										.collect();
									let texture_id = frame
										.tex_allocator()
										.alloc_srgba_premultiplied((width, height), &*colors);

									Some(PreviewPanel::Tim { image, texture_id })
								}
							},
							_ => Ok(None),
						};

						// Drop previous images, if they exist
						#[allow(clippy::single_match)] // We'll have more in the future
						match *preview_panel {
							Some(PreviewPanel::Tim { texture_id, .. }) => frame.tex_allocator().free(texture_id),
							_ => (),
						}

						match panel {
							Ok(panel) => *preview_panel = panel,
							Err(err) => {
								*preview_panel = Some(PreviewPanel::Error {
									err: format!("{err:?}"),
								})
							},
						}
					}
				});
			}
		});

		if let Some(preview_panel) = preview_panel {
			egui::CentralPanel::default().show(ctx, |ui| match *preview_panel {
				PreviewPanel::Tim { texture_id, ref image } => {
					egui::ScrollArea::auto_sized().show(ui, |ui| {
						ui.image(texture_id, image.size().map(|dim| dim as f32));
					});
				},
				PreviewPanel::Error { ref err } => {
					ui.group(|ui| {
						ui.heading("Error");
						ui.label(err);
					});
				},
			});
		}

		if let (Some(swap_window), Some(loaded_game)) = (swap_window, loaded_game) {
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
						(SwapFileStatus::Set(lhs), SwapFileStatus::Set(rhs)) => {
							let lhs = Path::from_ascii(lhs).expect("Lhs path wasn't valid");
							let rhs = Path::from_ascii(rhs).expect("Rhs path wasn't valid");

							let res: Result<_, anyhow::Error> = try {
								loaded_game
									.game_file
									.swap_files(lhs, rhs)
									.context("Unable to swap files")?;

								loaded_game.reload().context("Unable to reload the game")?;
							};

							match res {
								Ok(()) => {
									MessageDialog::new()
										.set_text("Successfully swapped!")
										.set_type(MessageType::Info)
										.show_alert()
										.expect("Unable to alert user");
								},
								Err(err) => MessageDialog::new()
									.set_text(&format!("Unable to swap files: {:?}", err))
									.set_type(MessageType::Error)
									.show_alert()
									.expect("Unable to alert user"),
							}
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

/// Preview panel
#[derive(PartialEq, Clone)]
pub enum PreviewPanel {
	/// Tim image
	Tim {
		/// Image
		image: Tim,

		/// Texture id
		texture_id: TextureId,
	},

	/// Error
	Error {
		/// Error
		err: String,
	},
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

impl LoadedGame {
	/// Reloads the game
	pub fn reload(&mut self) -> Result<(), anyhow::Error> {
		self.a_tree
			.reload(&mut self.game_file.a_drv().context("Unable to get `A` drive")?)
			.context("Unable to reload `A` drive")?;
		self.b_tree
			.reload(&mut self.game_file.b_drv().context("Unable to get `B` drive")?)
			.context("Unable to reload `B` drive")?;
		self.c_tree
			.reload(&mut self.game_file.c_drv().context("Unable to get `C` drive")?)
			.context("Unable to reload `C` drive")?;
		self.e_tree
			.reload(&mut self.game_file.e_drv().context("Unable to get `E` drive")?)
			.context("Unable to reload `E` drive")?;
		self.f_tree
			.reload(&mut self.game_file.f_drv().context("Unable to get `F` drive")?)
			.context("Unable to reload `F` drive")?;
		self.g_tree
			.reload(&mut self.game_file.g_drv().context("Unable to get `G` drive")?)
			.context("Unable to reload `G` drive")?;
		self.p_tree
			.reload(&mut self.game_file.p_drv().context("Unable to get `P` drive")?)
			.context("Unable to reload `P` drive")?;

		Ok(())
	}
}
