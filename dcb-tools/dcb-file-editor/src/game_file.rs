//! Game tree

use crate::{
	drv_tree::{self, DrvTree},
	swap_window::SwapWindow,
};
use anyhow::Context;
use dcb_cdrom_xa::CdRomCursor;
use dcb_util::MutexPoison;
use eframe::egui;
use std::{
	fs,
	io::{self, Write},
	sync::Mutex,
};

/// Game file
pub struct GameFile {
	/// Cd rom
	cdrom: Mutex<CdRomCursor<fs::File>>,

	/// Drives
	drives: Mutex<Drives>,
}

impl GameFile {
	/// Creates a new game file from it's file
	pub fn new(file: fs::File) -> Result<Self, anyhow::Error> {
		let mut cdrom = CdRomCursor::new(file);
		let mut game_file = dcb_io::GameFile::new(&mut cdrom);

		let mut a_reader = game_file.a_drv().context("Unable to get `a` drive")?;
		let a_tree = DrvTree::new(&mut a_reader).context("Unable to load `a` drive")?;
		let mut b_reader = game_file.b_drv().context("Unable to get `b` drive")?;
		let b_tree = DrvTree::new(&mut b_reader).context("Unable to load `b` drive")?;
		let mut c_reader = game_file.c_drv().context("Unable to get `c` drive")?;
		let c_tree = DrvTree::new(&mut c_reader).context("Unable to load `c` drive")?;
		let mut e_reader = game_file.e_drv().context("Unable to get `e` drive")?;
		let e_tree = DrvTree::new(&mut e_reader).context("Unable to load `e` drive")?;
		let mut f_reader = game_file.f_drv().context("Unable to get `f` drive")?;
		let f_tree = DrvTree::new(&mut f_reader).context("Unable to load `f` drive")?;
		let mut g_reader = game_file.g_drv().context("Unable to get `g` drive")?;
		let g_tree = DrvTree::new(&mut g_reader).context("Unable to load `g` drive")?;
		let mut p_reader = game_file.p_drv().context("Unable to get `p` drive")?;
		let p_tree = DrvTree::new(&mut p_reader).context("Unable to load `p` drive")?;

		let drives = Drives {
			a_tree,
			b_tree,
			c_tree,
			e_tree,
			f_tree,
			g_tree,
			p_tree,
		};

		Ok(Self {
			cdrom:  Mutex::new(cdrom),
			drives: Mutex::new(drives),
		})
	}

	/// Reloads the game
	pub fn reload(&self) -> Result<(), anyhow::Error> {
		// TODO: Avoid deadlock by acquiring these using some special algorithm
		let mut cdrom = self.cdrom.lock_unwrap();
		let mut game_file = dcb_io::GameFile::new(&mut *cdrom);
		let mut drives = self.drives.lock_unwrap();
		drives
			.a_tree
			.reload(&mut game_file.a_drv().context("Unable to get `A` drive")?)
			.context("Unable to reload `A` drive")?;
		drives
			.b_tree
			.reload(&mut game_file.b_drv().context("Unable to get `B` drive")?)
			.context("Unable to reload `B` drive")?;
		drives
			.c_tree
			.reload(&mut game_file.c_drv().context("Unable to get `C` drive")?)
			.context("Unable to reload `C` drive")?;
		drives
			.e_tree
			.reload(&mut game_file.e_drv().context("Unable to get `E` drive")?)
			.context("Unable to reload `E` drive")?;
		drives
			.f_tree
			.reload(&mut game_file.f_drv().context("Unable to get `F` drive")?)
			.context("Unable to reload `F` drive")?;
		drives
			.g_tree
			.reload(&mut game_file.g_drv().context("Unable to get `G` drive")?)
			.context("Unable to reload `G` drive")?;
		drives
			.p_tree
			.reload(&mut game_file.p_drv().context("Unable to get `P` drive")?)
			.context("Unable to reload `P` drive")?;

		Ok(())
	}

	/// Flushes the game file to disc
	pub fn flush(&self) -> Result<(), io::Error> {
		self.cdrom.lock_unwrap().flush()
	}

	/// Displays the game file tree
	pub fn display(
		&self, ui: &mut egui::Ui, file_search: &mut String, swap_window: &mut Option<SwapWindow>,
	) -> DisplayResults {
		let mut preview_path = None;
		let mut display_ctx = drv_tree::DisplayCtx {
			search_str:    file_search,
			on_file_click: |path: &str| {
				// If we have a swap window, call it's on file click
				if let Some(swap_window) = swap_window {
					swap_window.on_file_click(path);
				}

				// Then set the path to preview
				preview_path = Some(path.to_owned());
			},
		};

		let drives = self.drives.lock_unwrap();
		egui::CollapsingHeader::new("A:\\").show(ui, |ui| drives.a_tree.display(ui, "A:\\", &mut display_ctx));
		egui::CollapsingHeader::new("B:\\").show(ui, |ui| drives.b_tree.display(ui, "B:\\", &mut display_ctx));
		egui::CollapsingHeader::new("C:\\").show(ui, |ui| drives.c_tree.display(ui, "C:\\", &mut display_ctx));
		egui::CollapsingHeader::new("E:\\").show(ui, |ui| drives.e_tree.display(ui, "E:\\", &mut display_ctx));
		egui::CollapsingHeader::new("F:\\").show(ui, |ui| drives.f_tree.display(ui, "F:\\", &mut display_ctx));
		egui::CollapsingHeader::new("G:\\").show(ui, |ui| drives.g_tree.display(ui, "G:\\", &mut display_ctx));
		egui::CollapsingHeader::new("P:\\").show(ui, |ui| drives.p_tree.display(ui, "P:\\", &mut display_ctx));

		DisplayResults { preview_path }
	}

	/// Performs an operation using the game file
	pub fn with_game_file<T>(&self, f: impl FnOnce(dcb_io::GameFile<&mut CdRomCursor<fs::File>>) -> T) -> T {
		// We shouldn't be called from the main thread, as locking might
		// TODO: Get a better way of making sure it's the main thread and not some
		//       other thread with the name 'main'?
		debug_assert_ne!(
			std::thread::current().name(),
			Some("main"),
			"Main thread should not call this function, as it might block for an indeterminate amount of time"
		);

		// Get the game file
		let mut cdrom = self.cdrom.lock_unwrap();
		let game_file = dcb_io::GameFile::new(&mut *cdrom);
		f(game_file)
	}
}

/// Drives
pub struct Drives {
	/// `A` drive tree
	a_tree: DrvTree,

	/// `B` drive tree
	b_tree: DrvTree,

	/// `C` drive tree
	c_tree: DrvTree,

	/// `E` drive tree
	e_tree: DrvTree,

	/// `F` drive tree
	f_tree: DrvTree,

	/// `G` drive tree
	g_tree: DrvTree,

	/// `P` drive tree
	p_tree: DrvTree,
}

/// Display results
pub struct DisplayResults {
	/// Preview path
	pub preview_path: Option<String>,
}
