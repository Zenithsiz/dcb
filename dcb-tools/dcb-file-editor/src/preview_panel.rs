//! Preview panel

// Imports
use crate::GameFile;
use anyhow::Context;
use dcb_io::game_file::Path;
use dcb_tim::{Tim, Tis};
use dcb_util::{
	task::{self, ValueFuture},
	MutexPoison,
};
use eframe::{
	egui::{self, Color32, TextureId},
	epi::TextureAllocator,
};
use std::{
	io::BufReader,
	sync::{Arc, Mutex},
};

/// Preview panel
#[derive(PartialEq, Clone)]
pub enum PreviewPanel {
	/// Tim image
	Tim {
		/// Image
		tim: Tim,

		/// All textures
		pallettes: Vec<TextureId>,
	},

	/// Tim collection
	Tis {
		/// Images
		tims: Vec<(Tim, Vec<TextureId>)>,
	},

	/// Error
	Error {
		/// Error
		err: String,
	},

	/// Empty
	Empty,
}

impl PreviewPanel {
	/// Drops all textures in this panel
	pub fn drop_textures(&mut self, tex_allocator: &mut dyn TextureAllocator) {
		match self {
			PreviewPanel::Tim { pallettes, .. } => {
				for texture_id in pallettes.drain(..) {
					tex_allocator.free(texture_id);
				}
			},
			PreviewPanel::Tis { tims } => {
				for (_, textures) in tims {
					for texture_id in textures.drain(..) {
						tex_allocator.free(texture_id);
					}
				}
			},
			_ => (),
		}
	}

	/// Forgets all textures in this panel without freeing them
	pub fn forget_textures(&mut self) {
		match self {
			PreviewPanel::Tim { pallettes, .. } => {
				pallettes.drain(..);
			},
			PreviewPanel::Tis { tims } => {
				tims.drain(..);
			},
			_ => (),
		}
	}

	/// Displays this panel
	pub fn display(&self, ctx: &egui::CtxRef) {
		egui::CentralPanel::default().show(ctx, |ui| match self {
			Self::Tim { pallettes, tim } => {
				egui::ScrollArea::auto_sized().show(ui, |ui| {
					for &texture_id in pallettes {
						ui.image(texture_id, tim.size().map(|dim| dim as f32));
						ui.separator();
					}
				});
			},
			Self::Tis { tims } => {
				egui::ScrollArea::auto_sized().show(ui, |ui| {
					for (image, pallettes) in tims {
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
			Self::Empty => (),
		});
	}
}

impl Drop for PreviewPanel {
	fn drop(&mut self) {
		// Make sure we don't have any textures remaining
		let textures_len = match self {
			PreviewPanel::Tim { pallettes, .. } => pallettes.len(),
			PreviewPanel::Tis { tims } => tims.iter().map(|(_, pallettes)| pallettes.len()).sum(),
			_ => 0,
		};

		if textures_len != 0 {
			log::warn!("Leaking {} textures", textures_len);
		}
	}
}

/// Preview panel builder
pub enum PreviewPanelBuilder {
	/// Tim image
	Tim {
		tim:             Tim,
		pallette_pixels: Vec<Box<[Color32]>>,
	},

	/// Tis images
	Tis { tims: Vec<(Tim, Vec<Box<[Color32]>>)> },

	/// Error
	Error { err: String },

	/// Empty
	Empty,
}

impl PreviewPanelBuilder {
	/// Creates a new builder for a preview Panel
	pub fn new(game_file: Arc<Mutex<GameFile>>, path: String) -> ValueFuture<Self> {
		task::spawn(move || {
			let res: Result<_, anyhow::Error> = try {
				match path {
					path if path.ends_with(".TIM") => {
						// Deserialize the tim
						let path = Path::from_ascii(&path).context("Unable to create path")?;
						let mut game_file = game_file.lock_unwrap();
						let mut game_file = game_file.game_file();
						let mut file = game_file.open_file(path).context("Unable to open file")?;
						let tim = Tim::deserialize(&mut file).context("Unable to parse file")?;

						let pallette_pixels: Vec<Box<[_]>> = (0..tim.pallettes())
							.map(|pallette| {
								let pixels = tim
									.colors(Some(pallette))
									.context("Unable to get image colors")?
									.to_vec()
									.into_iter()
									.map(|[r, g, b, a]| Color32::from_rgba_premultiplied(r, g, b, a))
									.collect();
								Ok(pixels)
							})
							.collect::<Result<_, anyhow::Error>>()
							.context("Unable to get all pallette colors")?;

						Self::Tim { tim, pallette_pixels }
					},
					path if path.ends_with(".TIS") => {
						// Deserialize the tis
						let path = Path::from_ascii(&path).context("Unable to create path")?;
						let mut game_file = game_file.lock_unwrap();
						let mut game_file = game_file.game_file();
						let file = game_file.open_file(path).context("Unable to open file")?;
						let mut file = BufReader::new(file);
						let tis: Tis = Tis::deserialize(&mut file).context("Unable to parse file")?;

						let tims = tis
							.tims
							.into_iter()
							.map(|tim| {
								let pallette_pixels: Vec<Box<[_]>> = (0..tim.pallettes())
									.map(|pallette| {
										let pixels = tim
											.colors(Some(pallette))
											.context("Unable to get image colors")?
											.to_vec()
											.into_iter()
											.map(|[r, g, b, a]| Color32::from_rgba_premultiplied(r, g, b, a))
											.collect();
										Ok(pixels)
									})
									.collect::<Result<_, anyhow::Error>>()
									.context("Unable to get all pallette colors")?;
								Ok((tim, pallette_pixels))
							})
							.collect::<Result<_, anyhow::Error>>()
							.context("Unable to create images")?;

						Self::Tis { tims }
					},
					_ => Self::Empty,
				}
			};

			res.unwrap_or_else(|err| Self::Error {
				err: format!("{err:?}"),
			})
		})
	}

	/// Builds the preview panel
	pub fn build(self, tex_allocator: &mut dyn TextureAllocator) -> PreviewPanel {
		let res: Result<_, anyhow::Error> = try {
			match self {
				Self::Tim { tim, pallette_pixels } => PreviewPanel::Tim {
					pallettes: pallette_pixels
						.into_iter()
						.map(|pixels| {
							let [width, height] = tim.size();
							tex_allocator.alloc_srgba_premultiplied((width, height), &*pixels)
						})
						.collect(),
					tim,
				},
				Self::Tis { tims } => PreviewPanel::Tis {
					tims: tims
						.into_iter()
						.map(|(tim, pallette_pixels)| {
							let [width, height] = tim.size();
							Ok((
								tim,
								pallette_pixels
									.into_iter()
									.map(|pixels| tex_allocator.alloc_srgba_premultiplied((width, height), &*pixels))
									.collect(),
							))
						})
						.collect::<Result<Vec<_>, anyhow::Error>>()?,
				},
				Self::Error { err } => PreviewPanel::Error { err },
				Self::Empty => PreviewPanel::Empty,
			}
		};

		res.unwrap_or_else(|err| PreviewPanel::Error {
			err: format!("{err:?}"),
		})
	}
}
