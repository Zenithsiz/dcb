//! Card editor

// Features
#![feature(
	array_map,
	with_options,
	format_args_capture,
	once_cell,
	never_type,
	try_blocks,
	hash_drain_filter,
	iter_map_while
)]

// Modules
pub mod drv_tree;
pub mod game_file;
pub mod preview_panel;
pub mod swap_window;

// Imports
use anyhow::Context;
use dcb_util::{alert, task};
use eframe::{egui, epi, NativeOptions};
use game_file::GameFile;
use native_dialog::FileDialog;
use preview_panel::{PreviewPanel, PreviewPanelBuilder};
use std::{fs, io::Write, path::PathBuf, sync::Arc};
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
	game_file: Option<Arc<GameFile>>,

	/// Game file future
	game_file_future: Option<task::ValueFuture<Result<GameFile, anyhow::Error>>>,

	/// File search
	file_search: String,

	/// Swap window
	swap_window: Option<SwapWindow>,

	/// Preview panel
	preview_panel: PreviewPanel,

	/// Preview panel builder future
	preview_panel_builder_future: Option<task::ValueFuture<PreviewPanelBuilder>>,
}

impl Default for FileEditor {
	fn default() -> Self {
		Self {
			file_path:                    None,
			game_file:                    None,
			game_file_future:             None,
			file_search:                  String::new(),
			swap_window:                  None,
			preview_panel:                PreviewPanel::Empty,
			preview_panel_builder_future: None,
		}
	}
}

impl epi::App for FileEditor {
	fn update(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) {
		let Self {
			file_path,
			game_file,
			game_file_future,
			file_search,
			swap_window,
			preview_panel,
			preview_panel_builder_future,
		} = self;

		// If the game file finished loading, get it
		if let Some(res) = game_file_future.as_mut().and_then(|fut| fut.get()) {
			*game_file_future = None;
			match res {
				Ok(game) => *game_file = Some(Arc::new(game)),
				Err(err) => alert::error(&format!("Unable to open file: {:?}", err)),
			};
		}

		if let Some(panel) = preview_panel_builder_future.as_mut().and_then(|fut| fut.get()) {
			// Drop the future
			*preview_panel_builder_future = None;

			// Drop any textures and assign the new panel
			preview_panel.drop_textures(frame.tex_allocator());
			*preview_panel = panel.build(frame.tex_allocator());
		}

		// Top panel
		egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
			egui::menu::bar(ui, |ui| {
				egui::menu::menu(ui, "File", |ui| {
					// On open, ask the user and open the file
					if ui
						.add(egui::Button::new("Open").enabled(game_file_future.is_none()))
						.clicked()
					{
						// Ask the user for the file
						let cur_dir_path = std::env::current_dir().expect("Unable to get current directory path");
						*file_path = FileDialog::new()
							.set_location(&cur_dir_path)
							.add_filter("Game file", &["bin"])
							.show_open_single_file()
							.expect("Unable to ask user for file");

						// Then open the file
						if let Some(file_path) = file_path.clone() {
							*game_file_future = Some(task::spawn(move || {
								let file = fs::File::with_options()
									.read(true)
									.write(true)
									.open(file_path)
									.context("Unable to open file")?;

								GameFile::new(file).context("Unable to load game")
							}));
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
			if let Some(game_file) = game_file {
				egui::ScrollArea::auto_sized().show(ui, |ui| {
					let results = game_file.display(ui, file_search, swap_window);

					// Update the preview if a new file was clicked
					if let Some(path) = results.preview_path {
						// Note: If we're already loading, we discard it
						*preview_panel_builder_future = Some(PreviewPanelBuilder::new(Arc::clone(game_file), path));
					}
				});
			}
		});

		// Then draw the preview panel
		preview_panel.display(ctx);

		// And swap window
		if let (Some(swap_window), Some(game_file)) = (swap_window, game_file) {
			swap_window.display(ctx, &*game_file)
		}
	}

	fn on_exit(&mut self) {
		// Forget all images we have
		// TODO: Somehow get a frame here to drop them?
		self.preview_panel.forget_textures();

		// Flush the file if we have it
		if let Some(game_file) = &mut self.game_file {
			match game_file.game_file().cdrom().flush() {
				Ok(()) => (),
				Err(err) => alert::error(&format!("Unable to flush file tod isk: {:?}", err)),
			}
		}
	}

	fn name(&self) -> &str {
		"Dcb file editor"
	}
}
