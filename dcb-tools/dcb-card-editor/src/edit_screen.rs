//! Edit screen

// Imports
use crate::loaded_game::LoadedGame;
use anyhow::Context;
use dcb::{card::Card, CardTable};
use dcb_io::{game_file::Path, GameFile};
use dcb_tim::Tim;
use eframe::{
	egui::{self, Color32, TextureId},
	epi::TextureAllocator,
};
use std::io;
use zutil::alert;

/// An edit screen
pub struct EditScreen {
	/// Currently selected card
	card_idx: usize,

	/// Card image
	image: Option<(Tim, TextureId)>,
}

impl EditScreen {
	/// Creates a new edit screen from it's card index and loads it's image
	#[must_use]
	pub fn new<R: io::Seek + io::Read>(
		card_idx: usize, game_file: &mut GameFile<R>, tex_allocator: &mut dyn TextureAllocator,
	) -> Self {
		// Try to open the image
		let image: Result<_, anyhow::Error> = try {
			// Get the path
			let path = CardTable::card_image_path(card_idx);
			let path = Path::new(&path);

			// Open it and parse it
			let mut file = game_file.open_file(path).context("Unable to open image file")?;
			let tim = Tim::deserialize(&mut file).context("Unable to parse image file")?;

			// Then generate the image
			let [width, height] = tim.size();
			let pixels: Vec<_> = tim
				.colors(None)
				.context("Unable to get image colors")?
				.to_vec()
				.into_iter()
				.map(|[r, g, b, a]| Color32::from_rgba_premultiplied(r, g, b, a))
				.collect();
			let id = tex_allocator.alloc_srgba_premultiplied((width, height), &pixels);

			(tim, id)
		};

		// If we didn't manage to load the image, warn the user
		let image = match image {
			Ok(image) => Some(image),
			Err(err) => {
				alert::error!("Unable to load image for #{card_idx}: {err:?}");
				None
			},
		};

		Self { card_idx, image }
	}

	/// Returns the card index of this screen
	#[must_use]
	pub fn card_idx(&self) -> usize {
		self.card_idx
	}

	/// Displays all edit screens
	pub fn display_all(screens: &mut Vec<Self>, ctx: &egui::CtxRef, loaded_game: &mut LoadedGame) {
		let screen_width = ctx.available_rect().width() / (screens.len() as f32);
		for screen in screens {
			let card = &mut loaded_game.card_table.cards[screen.card_idx];

			egui::SidePanel::left(screen as *const _)
				.min_width(screen_width)
				.max_width(screen_width)
				.resizable(false)
				.show(ctx, |ui| {
					// Header for the card
					ui.vertical(|ui| {
						ui.heading(card.name().as_str());
						ui.label(match card {
							Card::Digimon(_) => "Digimon",
							Card::Item(_) => "Item",
							Card::Digivolve(_) => "Digivolve",
						});
						ui.separator();
					});

					egui::ScrollArea::auto_sized().show(ui, |ui| {
						// Card image
						if let Some((ref tim, id)) = screen.image {
							ui.vertical(|ui| {
								ui.add_space(ui.min_rect().width());
								ui.image(id, tim.size().map(|c| c as f32));
							});
							ui.separator();
						}

						crate::render_card(ui, card);
					});
				});
		}
	}
}
