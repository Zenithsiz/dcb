//! Preview panel

// Imports
use crate::LoadedGame;
use anyhow::Context;
use dcb_io::game_file::Path;
use dcb_tim::{Tim, Tis};
use eframe::{
	egui::{self, Color32, TextureId},
	epi::TextureAllocator,
};
use std::io::BufReader;

/// Preview panel
#[derive(PartialEq, Clone)]
pub enum PreviewPanel {
	/// Tim image
	Tim {
		/// Image
		image: Tim,

		/// All textures
		pallettes: Vec<TextureId>,
	},

	/// Tim collection
	Tis {
		/// Images
		images: Vec<(Tim, Vec<TextureId>)>,
	},

	/// Error
	Error {
		/// Error
		err: String,
	},
}

impl PreviewPanel {
	/// Creates a preview panel
	pub fn new(
		loaded_game: &mut LoadedGame, path: &str, tex_allocator: &mut dyn TextureAllocator,
	) -> Result<Option<Self>, anyhow::Error> {
		let panel = match path {
			path if path.ends_with(".TIM") => {
				// Deserialize the tim
				let path = Path::from_ascii(&path).context("Unable to create path")?;
				let mut file = loaded_game
					.game_file_mut()
					.open_file(path)
					.context("Unable to open file")?;
				let image: Tim = Tim::deserialize(&mut file).context("Unable to parse file")?;

				// Then create all pallettes
				let pallettes = create_image_pallettes(&image, tex_allocator)?;

				Self::Tim { image, pallettes }
			},
			path if path.ends_with(".TIS") => {
				// Deserialize the tis
				let path = Path::from_ascii(&path).context("Unable to create path")?;
				let file = loaded_game
					.game_file_mut()
					.open_file(path)
					.context("Unable to open file")?;
				let mut file = BufReader::new(file);
				let images: Tis = Tis::deserialize(&mut file).context("Unable to parse file")?;

				// Then create all images
				let images = images
					.tims
					.into_iter()
					.map(|image| {
						let pallettes = create_image_pallettes(&image, tex_allocator)
							.context("Unable to create image pallettes")?;

						Ok((image, pallettes))
					})
					.collect::<Result<Vec<_>, anyhow::Error>>()?;

				Self::Tis { images }
			},
			_ => return Ok(None),
		};

		Ok(Some(panel))
	}

	/// Drops all textures in this panel
	pub fn drop_textures(&mut self, tex_allocator: &mut dyn TextureAllocator) {
		match self {
			PreviewPanel::Tim { pallettes, .. } => {
				for texture_id in pallettes.drain(..) {
					tex_allocator.free(texture_id);
				}
			},
			PreviewPanel::Tis { images } => {
				for (_, textures) in images {
					for texture_id in textures.drain(..) {
						tex_allocator.free(texture_id);
					}
				}
			},
			_ => (),
		}
	}

	/// Displays this panel
	pub fn display(&self, ctx: &egui::CtxRef) {
		egui::CentralPanel::default().show(ctx, |ui| match self {
			Self::Tim { pallettes, image } => {
				egui::ScrollArea::auto_sized().show(ui, |ui| {
					for &texture_id in pallettes {
						ui.image(texture_id, image.size().map(|dim| dim as f32));
						ui.separator();
					}
				});
			},
			Self::Tis { images } => {
				egui::ScrollArea::auto_sized().show(ui, |ui| {
					for (image, pallettes) in images {
						for &texture_id in pallettes {
							ui.image(texture_id, image.size().map(|dim| dim as f32));
							ui.separator();
						}
					}
				});
			},
			Self::Error { err } => {
				ui.group(|ui| {
					ui.heading("Error");
					ui.label(err);
				});
			},
		});
	}
}

impl Drop for PreviewPanel {
	fn drop(&mut self) {
		// Make sure we don't have any textures remaining
		match self {
			PreviewPanel::Tim { pallettes, .. } => assert_eq!(pallettes.len(), 0),
			PreviewPanel::Tis { images } => {
				for (_, pallettes) in images {
					assert_eq!(pallettes.len(), 0)
				}
			},
			PreviewPanel::Error { .. } => {},
		};
	}
}

/// Creates all pallettes for an image
fn create_image_pallettes(
	image: &Tim, tex_allocator: &mut dyn TextureAllocator,
) -> Result<Vec<TextureId>, anyhow::Error> {
	let [width, height] = image.size();
	let textures = (0..image.pallettes())
		.map(|pallette| {
			let colors: Box<[_]> = image
				.colors(Some(pallette))
				.context("Unable to get image colors")?
				.to_vec()
				.into_iter()
				.map(|[r, g, b, a]| Color32::from_rgba_premultiplied(r, g, b, a))
				.collect();
			let texture_id = tex_allocator.alloc_srgba_premultiplied((width, height), &*colors);
			Ok(texture_id)
		})
		.collect::<Result<_, anyhow::Error>>()?;
	Ok(textures)
}
