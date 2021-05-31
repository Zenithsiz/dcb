//! Game tree

use crate::{
	drv_tree::{self, DrvTree},
	swap_window::SwapWindow,
};
use anyhow::Context;
use dcb_cdrom_xa::CdRomCursor;
use eframe::egui;
use std::fs;

/// Game file
pub struct GameFile {
	/// Game file
	file: fs::File,

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

impl GameFile {
	/// Creates a new game
	pub fn new(mut file: fs::File) -> Result<Self, anyhow::Error> {
		let mut game_file = dcb_io::GameFile::new(CdRomCursor::new(&mut file));
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

		Ok(Self {
			file,
			a_tree,
			b_tree,
			c_tree,
			e_tree,
			f_tree,
			g_tree,
			p_tree,
		})
	}

	/// Reloads the game
	pub fn reload(&mut self) -> Result<(), anyhow::Error> {
		let mut game_file = dcb_io::GameFile::new(CdRomCursor::new(&mut self.file));
		self.a_tree
			.reload(&mut game_file.a_drv().context("Unable to get `A` drive")?)
			.context("Unable to reload `A` drive")?;
		self.b_tree
			.reload(&mut game_file.b_drv().context("Unable to get `B` drive")?)
			.context("Unable to reload `B` drive")?;
		self.c_tree
			.reload(&mut game_file.c_drv().context("Unable to get `C` drive")?)
			.context("Unable to reload `C` drive")?;
		self.e_tree
			.reload(&mut game_file.e_drv().context("Unable to get `E` drive")?)
			.context("Unable to reload `E` drive")?;
		self.f_tree
			.reload(&mut game_file.f_drv().context("Unable to get `F` drive")?)
			.context("Unable to reload `F` drive")?;
		self.g_tree
			.reload(&mut game_file.g_drv().context("Unable to get `G` drive")?)
			.context("Unable to reload `G` drive")?;
		self.p_tree
			.reload(&mut game_file.p_drv().context("Unable to get `P` drive")?)
			.context("Unable to reload `P` drive")?;

		Ok(())
	}

	/// Displays the game file tree
	pub fn display(
		&mut self, ui: &mut egui::Ui, file_search: &mut String, swap_window: &mut Option<SwapWindow>,
	) -> DisplayResults {
		let mut preview_path = None;
		let mut display_ctx = drv_tree::DisplayCtx {
			search_str:    &file_search,
			on_file_click: |path: &str| {
				// If we have a swap window, call it's on file click
				if let Some(swap_window) = swap_window {
					swap_window.on_file_click(path);
				}

				// Then set the path to preview
				preview_path = Some(path.to_owned());
			},
		};

		egui::CollapsingHeader::new("A:\\").show(ui, |ui| self.a_tree.display(ui, "A:\\", &mut display_ctx));
		egui::CollapsingHeader::new("B:\\").show(ui, |ui| self.b_tree.display(ui, "B:\\", &mut display_ctx));
		egui::CollapsingHeader::new("C:\\").show(ui, |ui| self.c_tree.display(ui, "C:\\", &mut display_ctx));
		egui::CollapsingHeader::new("E:\\").show(ui, |ui| self.e_tree.display(ui, "E:\\", &mut display_ctx));
		egui::CollapsingHeader::new("F:\\").show(ui, |ui| self.f_tree.display(ui, "F:\\", &mut display_ctx));
		egui::CollapsingHeader::new("G:\\").show(ui, |ui| self.g_tree.display(ui, "G:\\", &mut display_ctx));
		egui::CollapsingHeader::new("P:\\").show(ui, |ui| self.p_tree.display(ui, "P:\\", &mut display_ctx));

		DisplayResults { preview_path }
	}

	/// Returns a game file
	pub fn game_file(&mut self) -> dcb_io::GameFile<&mut fs::File> {
		dcb_io::GameFile::new(CdRomCursor::new(&mut self.file))
	}
}

/// Display results
pub struct DisplayResults {
	/// Preview path
	pub preview_path: Option<String>,
}
