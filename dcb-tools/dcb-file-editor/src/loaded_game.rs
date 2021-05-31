//! Game tree

use crate::{
	preview_panel::PreviewPanel,
	swap_window::SwapWindow,
	tree::{self, FsTree},
};
use anyhow::Context;
use dcb_io::GameFile;
use eframe::{egui, epi};
use std::fs;

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
	/// Creates a new game
	pub fn new(mut game_file: GameFile<fs::File>) -> Result<Self, anyhow::Error> {
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

		Ok(Self {
			game_file,
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

	/// Displays this game tree
	pub fn display(
		&mut self, frame: &mut epi::Frame, ui: &mut egui::Ui, file_search: &mut String,
		swap_window: &mut Option<SwapWindow>, preview_panel: &mut Option<PreviewPanel>,
	) {
		egui::ScrollArea::auto_sized().show(ui, |ui| {
			let mut preview_path = None;
			let mut display_ctx = tree::DisplayCtx {
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

			if let Some(path) = preview_path {
				let panel = PreviewPanel::new(self, &path, frame.tex_allocator()).context("Unable to preview file");

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

	/// Get a mutable reference to the loaded game's game file.
	pub fn game_file_mut(&mut self) -> &mut GameFile<fs::File> {
		&mut self.game_file
	}
}
