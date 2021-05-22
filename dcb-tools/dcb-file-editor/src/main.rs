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

use anyhow::Context;
use dcb_cdrom_xa::CdRomCursor;
use dcb_drv::cursor::{DirEntryCursorKind, DrvFsCursor};
// Imports
use eframe::{egui, epi, NativeOptions};
use native_dialog::{FileDialog, MessageDialog, MessageType};
use std::{
	fs,
	io::{self, Write},
	mem,
	path::PathBuf,
};

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
								*loaded_game =
									Some(LoadedGame::new(CdRomCursor::new(file)).context("Unable to load all files")?);
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
						dir: &dcb_drv::cursor::DirCursor, ui: &mut egui::Ui, file_search: &str, dir_path: &str,
						swap_window: &mut Option<SwapWindow>,
					) {
						for entry in dir.entries() {
							match &entry.kind() {
								DirEntryCursorKind::Dir(dir) => {
									let dir_path = format!("{dir_path}{}\\", entry.name());
									ui.group(|ui| {
										ui.label(entry.name().as_str());
										ui.separator();
										show_dir(dir, ui, file_search, &dir_path, swap_window);
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

					ui.group(|ui| {
						ui.label("A.DRV");
						ui.separator();
						show_dir(
							loaded_game.a_drv_cursor.root_dir(),
							ui,
							file_search,
							"A:\\",
							swap_window,
						);
					});
					ui.group(|ui| {
						ui.label("B.DRV");
						ui.separator();
						show_dir(
							loaded_game.b_drv_cursor.root_dir(),
							ui,
							file_search,
							"B:\\",
							swap_window,
						);
					});
					ui.group(|ui| {
						ui.label("C.DRV");
						ui.separator();
						show_dir(
							loaded_game.c_drv_cursor.root_dir(),
							ui,
							file_search,
							"C:\\",
							swap_window,
						);
					});
					ui.group(|ui| {
						ui.label("E.DRV");
						ui.separator();
						show_dir(
							loaded_game.e_drv_cursor.root_dir(),
							ui,
							file_search,
							"E:\\",
							swap_window,
						);
					});
					ui.group(|ui| {
						ui.label("F.DRV");
						ui.separator();
						show_dir(
							loaded_game.f_drv_cursor.root_dir(),
							ui,
							file_search,
							"F:\\",
							swap_window,
						);
					});
					ui.group(|ui| {
						ui.label("G.DRV");
						ui.separator();
						show_dir(
							loaded_game.g_drv_cursor.root_dir(),
							ui,
							file_search,
							"G:\\",
							swap_window,
						);
					});
					ui.group(|ui| {
						ui.label("P.DRV");
						ui.separator();
						show_dir(
							loaded_game.p_drv_cursor.root_dir(),
							ui,
							file_search,
							"P:\\",
							swap_window,
						);
					});
				});
			}
		});

		if let Some(swap_window) = swap_window {
			egui::Window::new("Swap screen").show(ctx, |ui| {
				ui.horizontal(|ui| {
					ui.label(swap_window.first.as_str().unwrap_or("None"));
					if !swap_window.first.is_setting() && ui.button("Set").clicked() {
						swap_window.first.make_setting();
					}
					if swap_window.first.is_setting() && ui.button("Setting").clicked() {
						swap_window.first.make_not_setting();
					}
				});
				ui.horizontal(|ui| {
					ui.label(swap_window.second.as_str().unwrap_or("None"));
					if !swap_window.second.is_setting() && ui.button("Set").clicked() {
						swap_window.second.make_setting();
					}
					if swap_window.second.is_setting() && ui.button("Setting").clicked() {
						swap_window.second.make_not_setting();
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
			let _ = loaded_game.file.flush();
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
	/// Sets this status to `Setting`
	pub fn make_setting(&mut self) {
		*self = match mem::take(self) {
			Self::Unset => Self::Setting(None),
			status @ Self::Setting(_) => status,
			Self::Set(s) => Self::Setting(Some(s)),
		};
	}

	/// Sets this status from `Setting` to another
	pub fn make_not_setting(&mut self) {
		*self = match mem::take(self) {
			Self::Setting(s) => match s {
				Some(s) => Self::Set(s),
				None => Self::Unset,
			},
			_ => unreachable!(),
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
	/// File
	file: CdRomCursor<fs::File>,

	/// `A.DRV` cursor
	a_drv_cursor: DrvFsCursor,

	/// `B.DRV` cursor
	b_drv_cursor: DrvFsCursor,

	/// `C.DRV` cursor
	c_drv_cursor: DrvFsCursor,

	/// `E.DRV` cursor
	e_drv_cursor: DrvFsCursor,

	/// `F.DRV` cursor
	f_drv_cursor: DrvFsCursor,

	/// `G.DRV` cursor
	g_drv_cursor: DrvFsCursor,

	/// `P.DRV` cursor
	p_drv_cursor: DrvFsCursor,
}

impl LoadedGame {
	/// `A.DRV` Offset
	pub const A_OFFSET: u64 = 0xa1000;
	/// `A.DRV` Size
	pub const A_SIZE: u64 = 0xa78800;
	/// `B.DRV` Offset
	pub const B_OFFSET: u64 = 0xb19800;
	/// `B.DRV` Size
	pub const B_SIZE: u64 = 0x17ea800;
	/// `C.DRV` Offset
	pub const C_OFFSET: u64 = 0x2304000;
	/// `C.DRV` Size
	pub const C_SIZE: u64 = 0xb6c000;
	/// `E.DRV` Offset
	pub const E_OFFSET: u64 = 0x2e70000;
	/// `E.DRV` Size
	pub const E_SIZE: u64 = 0x1886800;
	/// `F.DRV` Offset
	pub const F_OFFSET: u64 = 0x46f6800;
	/// `F.DRV` SIze
	pub const F_SIZE: u64 = 0xf2f800;
	/// `G.DRV` Offset
	pub const G_OFFSET: u64 = 0x5626000;
	/// `G.DRV` Size
	pub const G_SIZE: u64 = 0x293000;
	/// `P.DRV` Offset
	pub const P_OFFSET: u64 = 0xc000;
	/// `P.DRV` Size
	pub const P_SIZE: u64 = 0x95000;

	pub fn new(mut file: CdRomCursor<fs::File>) -> Result<Self, anyhow::Error> {
		let mut a_drv = LoadedGame::a_drv(&mut file).context("Unable to open `A.DRV` file")?;
		let a_drv_cursor = DrvFsCursor::new(&mut a_drv).context("Unable to parse `A.DRV` file")?;
		mem::drop(a_drv);

		let mut b_drv = LoadedGame::b_drv(&mut file).context("Unable to open `B.DRV` file")?;
		let b_drv_cursor = DrvFsCursor::new(&mut b_drv).context("Unable to parse `B.DRV` file")?;
		mem::drop(b_drv);

		let mut c_drv = LoadedGame::c_drv(&mut file).context("Unable to open `C.DRV` file")?;
		let c_drv_cursor = DrvFsCursor::new(&mut c_drv).context("Unable to parse `C.DRV` file")?;
		mem::drop(c_drv);

		let mut e_drv = LoadedGame::e_drv(&mut file).context("Unable to open `E.DRV` file")?;
		let e_drv_cursor = DrvFsCursor::new(&mut e_drv).context("Unable to parse `E.DRV` file")?;
		mem::drop(e_drv);

		let mut f_drv = LoadedGame::f_drv(&mut file).context("Unable to open `F.DRV` file")?;
		let f_drv_cursor = DrvFsCursor::new(&mut f_drv).context("Unable to parse `F.DRV` file")?;
		mem::drop(f_drv);

		let mut g_drv = LoadedGame::g_drv(&mut file).context("Unable to open `G.DRV` file")?;
		let g_drv_cursor = DrvFsCursor::new(&mut g_drv).context("Unable to parse `G.DRV` file")?;
		mem::drop(g_drv);

		let mut p_drv = LoadedGame::p_drv(&mut file).context("Unable to open `P.DRV` file")?;
		let p_drv_cursor = DrvFsCursor::new(&mut p_drv).context("Unable to parse `P.DRV` file")?;
		mem::drop(p_drv);

		Ok(Self {
			file,
			a_drv_cursor,
			b_drv_cursor,
			c_drv_cursor,
			e_drv_cursor,
			f_drv_cursor,
			g_drv_cursor,
			p_drv_cursor,
		})
	}

	/// Returns the `A.DRV` file
	pub fn a_drv(file: &mut CdRomCursor<fs::File>) -> Result<impl io::Read + io::Write + io::Seek + '_, io::Error> {
		dcb_util::IoCursor::new(file, Self::A_OFFSET, Self::A_SIZE)
	}

	/// Returns the `B.DRV` file
	pub fn b_drv(file: &mut CdRomCursor<fs::File>) -> Result<impl io::Read + io::Write + io::Seek + '_, io::Error> {
		dcb_util::IoCursor::new(file, Self::B_OFFSET, Self::B_SIZE)
	}

	/// Returns the `C.DRV` file
	pub fn c_drv(file: &mut CdRomCursor<fs::File>) -> Result<impl io::Read + io::Write + io::Seek + '_, io::Error> {
		dcb_util::IoCursor::new(file, Self::C_OFFSET, Self::C_SIZE)
	}

	/// Returns the `E.DRV` file
	pub fn e_drv(file: &mut CdRomCursor<fs::File>) -> Result<impl io::Read + io::Write + io::Seek + '_, io::Error> {
		dcb_util::IoCursor::new(file, Self::E_OFFSET, Self::E_SIZE)
	}

	/// Returns the `F.DRV` file
	pub fn f_drv(file: &mut CdRomCursor<fs::File>) -> Result<impl io::Read + io::Write + io::Seek + '_, io::Error> {
		dcb_util::IoCursor::new(file, Self::F_OFFSET, Self::F_SIZE)
	}

	/// Returns the `G.DRV` file
	pub fn g_drv(file: &mut CdRomCursor<fs::File>) -> Result<impl io::Read + io::Write + io::Seek + '_, io::Error> {
		dcb_util::IoCursor::new(file, Self::G_OFFSET, Self::G_SIZE)
	}

	/// Returns the `P.DRV` file
	pub fn p_drv(file: &mut CdRomCursor<fs::File>) -> Result<impl io::Read + io::Write + io::Seek + '_, io::Error> {
		dcb_util::IoCursor::new(file, Self::P_OFFSET, Self::P_SIZE)
	}
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
