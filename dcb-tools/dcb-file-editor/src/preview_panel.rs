//! Preview panel

// Imports
use crate::GameFile;
use anyhow::Context;
use dcb_io::game_file::Path;
use dcb_tim::{Tim, Tis};
use dcb_util::task::{self, ValueFuture};
use eframe::{
	egui::{self, Color32, TextureId},
	epi::TextureAllocator,
};
use std::{io::BufReader, sync::Arc};

/// Preview panel
#[derive(PartialEq, Clone)]
pub enum PreviewPanel {
	/// Tim image
	Tim(TimDisplay),

	/// Tim collection
	Tis {
		/// All tims
		tims: Vec<TimDisplay>,

		/// Current tim
		cur_tim: usize,
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
			PreviewPanel::Tim(tim) => {
				for texture_id in tim.pallettes.drain(..) {
					tex_allocator.free(texture_id);
				}
			},
			PreviewPanel::Tis { tims, .. } => {
				for tim in tims {
					for texture_id in tim.pallettes.drain(..) {
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
			PreviewPanel::Tim(tim) => {
				tim.pallettes.drain(..);
			},
			PreviewPanel::Tis { tims, .. } => {
				tims.drain(..);
			},
			_ => (),
		}
	}

	/// Displays this panel
	pub fn display(&mut self, ctx: &egui::CtxRef) {
		egui::CentralPanel::default().show(ctx, |ui| match self {
			Self::Tim(tim) => {
				tim.display(ctx, ui);
			},
			Self::Tis { tims, cur_tim } => {
				egui::ScrollArea::auto_sized().show(ui, |ui| {
					egui::TopBottomPanel::bottom(tims as *const _).show(ctx, |ui| {
						ui.label("Image");
						ui.horizontal_wrapped(|ui| {
							for n in 0..tims.len() {
								if ui.selectable_label(*cur_tim == n, format!("{n}")).clicked() {
									*cur_tim = n;
								}
							}
						});
					});

					tims[*cur_tim].display(ctx, ui);
				});
			},
			Self::Error { ref err } => {
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
			PreviewPanel::Tim(tim) => tim.pallettes.len(),
			PreviewPanel::Tis { tims, .. } => tims.iter().map(|tim| tim.pallettes.len()).sum(),
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
	pub fn new(game_file: Arc<GameFile>, path: String) -> ValueFuture<Self> {
		task::spawn(move || {
			let res: Result<_, anyhow::Error> = try {
				match path {
					path if path.ends_with(".TIM") => {
						// Deserialize the tim
						let path = Path::from_ascii(&path).context("Unable to create path")?;
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
				Self::Tim { tim, pallette_pixels } => PreviewPanel::Tim(TimDisplay {
					pallettes: pallette_pixels
						.into_iter()
						.map(|pixels| {
							let [width, height] = tim.size();
							tex_allocator.alloc_srgba_premultiplied((width, height), &*pixels)
						})
						.collect(),
					tim,
					cur_pallette: 0,
				}),
				Self::Tis { tims } => PreviewPanel::Tis {
					tims:    tims
						.into_iter()
						.map(|(tim, pallette_pixels)| {
							let [width, height] = tim.size();
							Ok(TimDisplay {
								tim,
								pallettes: pallette_pixels
									.into_iter()
									.map(|pixels| tex_allocator.alloc_srgba_premultiplied((width, height), &*pixels))
									.collect(),
								cur_pallette: 0,
							})
						})
						.collect::<Result<Vec<_>, anyhow::Error>>()?,
					cur_tim: 0,
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

/// Tim display
#[derive(PartialEq, Clone)]
pub struct TimDisplay {
	/// Image
	tim: Tim,

	/// All textures
	pallettes: Vec<TextureId>,

	/// Current pallette
	cur_pallette: usize,
}

impl TimDisplay {
	/// Displays a tim along with it's pallettes
	fn display(&mut self, ctx: &egui::CtxRef, ui: &mut egui::Ui) {
		egui::TopBottomPanel::bottom(&self.tim as *const _).show(ctx, |ui| {
			ui.label("Pallette");
			ui.horizontal_wrapped(|ui| {
				for n in 0..self.pallettes.len() {
					if ui.selectable_label(self.cur_pallette == n, format!("{n}")).clicked() {
						self.cur_pallette = n;
					}
				}
			});
		});

		egui::ScrollArea::auto_sized().show(ui, |ui| {
			ui.image(self.pallettes[self.cur_pallette], self.tim.size().map(|dim| dim as f32));
		});
	}
}
