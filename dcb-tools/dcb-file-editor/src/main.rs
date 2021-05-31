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
pub mod drv_tree;
pub mod game_file;
pub mod preview_panel;
pub mod swap_window;

// Imports
use anyhow::Context;
use dcb_cdrom_xa::CdRomCursor;
use eframe::{egui, epi, NativeOptions};
use game_file::GameFile;
use native_dialog::{FileDialog, MessageDialog, MessageType};
use preview_panel::PreviewPanel;
use std::{fs, io::Write, path::PathBuf};
use swap_window::SwapWindow;

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

	/// Game file
	game_file: Option<GameFile>,

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
			game_file:     None,
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
			game_file,
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
						// Ask the user for the file
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
								let file = dcb_io::GameFile::new(cdrom);

								*game_file = Some(GameFile::new(file).context("Unable to load game")?);
							};

							if let Err(err) = res {
								self::alert_error(&format!("Unable to open file: {:?}", err));
							}
						}
					}

					if ui.button("Quit").clicked() {
						frame.quit();
					}
				});

				egui::menu::menu(ui, "Edit", |ui| {
					if game_file.is_some() && ui.button("Swap").clicked() {
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

			// If we have a game file, display it and update the preview
			if let Some(game_file) = game_file.as_mut() {
				egui::ScrollArea::auto_sized().show(ui, |ui| {
					let results = game_file.display(ui, file_search, swap_window);

					// Update the preview if a new file was clicked
					if let Some(path) = results.preview_path {
						let panel = PreviewPanel::new(game_file, &path, frame.tex_allocator())
							.context("Unable to preview file");

						// Drop previous images, if they exist
						if let Some(preview_panel) = preview_panel {
							preview_panel.drop_textures(frame.tex_allocator());
						}

						*preview_panel = panel.unwrap_or_else(|err| {
							let err = format!("{err:?}");
							Some(PreviewPanel::Error { err })
						});
					}
				});
			}
		});

		if let Some(preview_panel) = preview_panel {
			preview_panel.display(ctx);
		}

		if let (Some(swap_window), Some(game_file)) = (swap_window, game_file) {
			swap_window.display(ctx, game_file)
		}
	}

	fn on_exit(&mut self) {
		// Forget all images we have
		// TODO: Somehow get a frame here to drop them?
		if let Some(preview_panel) = &mut self.preview_panel {
			preview_panel.forget_textures();
		}

		// Flush the file if we have it
		if let Some(game_file) = &mut self.game_file {
			match game_file.game_file_mut().cdrom().flush() {
				Ok(()) => (),
				Err(err) => self::alert_error(&format!("Unable to flush file tod isk: {:?}", err)),
			}
		}
	}

	fn name(&self) -> &str {
		"Dcb file editor"
	}
}

/// Alerts an error to the user
fn alert_error(msg: &str) {
	MessageDialog::new()
		.set_text(msg)
		.set_type(MessageType::Error)
		.show_alert()
		.expect("Unable to alert user")
}

/// Alerts a warning to the user
fn alert_warn(msg: &str) {
	MessageDialog::new()
		.set_text(msg)
		.set_type(MessageType::Warning)
		.show_alert()
		.expect("Unable to alert user")
}

/// Alerts info to the user
fn alert_info(msg: &str) {
	MessageDialog::new()
		.set_text(msg)
		.set_type(MessageType::Info)
		.show_alert()
		.expect("Unable to alert user")
}
