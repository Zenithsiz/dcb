//! Data patches
//!
//! # Details
//! Patches data to the game file from several other files.
//!
//! # Syntax
//! The executable may be called as `./patcher <game file> <directory>`
//!
//! Use the command `./patcher --help` for more information.
//!
//! # Data patched
//! Currently only the following is patched:
//! - Card table

// Features
#![feature(box_syntax, backtrace, panic_info_message, bool_to_option)]
// Lints
#![warn(clippy::restriction, clippy::pedantic, clippy::nursery)]
#![allow(
	clippy::implicit_return,          // We prefer implicit returns where possible
	clippy::module_name_repetitions,  // This happens often due to separating things into modules finely
	clippy::wildcard_enum_match_arm,  // We only use wildcards when we truly only care about some variants
	clippy::result_expect_used,
	clippy::option_expect_used,       // We use expect when there is no alternative.
	clippy::used_underscore_binding,  // Useful for macros and such
	clippy::integer_arithmetic,
	clippy::float_arithmetic,         // We need to use numbers my guy
	clippy::as_conversions,
	clippy::cast_sign_loss,
	clippy::cast_possible_wrap,
	clippy::cast_possible_truncation, // Needed for converting between stuff
	clippy::items_after_statements,
)]

// Modules
mod cli;
#[path = "../logger.rs"]
mod logger;
#[path = "../panic.rs"]
mod panic;

// Gfx / GLutin
use gfx::Device;
use glutin::{Event, WindowEvent};

// Imgui
use imgui::{Context, FontConfig, FontGlyphRanges, FontSource, ImString};
use imgui_gfx_renderer::{Renderer, Shaders};
use imgui_winit_support::{HiDpiMode, WinitPlatform};

// Errors
use err_backtrace::ErrBacktraceExt;
use err_ext::ResultExt;
use err_panic::ErrorExtPanic;

// Itertools
use itertools::Itertools;

// Dcb
use dcb::game::{card::Table as CardTable, deck::Table as DeckTable};

#[allow(clippy::too_many_lines)] // TODO: Fix
fn main() {
	// Initialize the logger and set the panic handler
	logger::init();
	std::panic::set_hook(box panic::log_handler);

	// Get all data from cli
	let cli::CliData { data_dir } = cli::CliData::new();

	// Load all data files
	let cards_table_file = std::fs::File::open(data_dir.join("cards.yaml")).panic_err_msg("Unable to open `cards.yaml`");
	let decks_table_file = std::fs::File::open(data_dir.join("decks.yaml")).panic_err_msg("Unable to open `decks.yaml`");

	// Parse everything from yaml
	let mut cards_table: CardTable = serde_yaml::from_reader(cards_table_file).panic_err_msg("Unable to parse `cards.yaml`");
	let decks_table: DeckTable = serde_yaml::from_reader(decks_table_file).panic_err_msg("Unable to parse `decks.yaml`");

	// Create a new window
	let mut events_loop = glutin::EventsLoop::new();
	let builder = glutin::WindowBuilder::new()
		.with_title("Dcb Card Editor")
		.with_dimensions(glutin::dpi::LogicalSize::new(1024.0, 768.0));
	let context = glutin::ContextBuilder::new().with_vsync(true);
	let (windowed_context, mut device, mut factory, mut main_color, mut main_depth) =
		gfx_window_glutin::init::<gfx::format::Rgba8, gfx::format::DepthStencil>(builder, context, &events_loop)
			.panic_err_msg("Failed to initialize graphics");
	let mut encoder: gfx::Encoder<_, _> = factory.create_command_buffer().into();
	let shaders = {
		let version = device.get_info().shading_language;
		if version.is_embedded {
			if version.major >= 3 {
				Shaders::GlSlEs300
			} else {
				Shaders::GlSlEs100
			}
		} else if version.major >= 4 {
			Shaders::GlSl400
		} else if version.major >= 3 {
			if version.minor >= 2 {
				Shaders::GlSl150
			} else {
				Shaders::GlSl130
			}
		} else {
			Shaders::GlSl110
		}
	};

	// Create a new context for `imgui`
	let mut imgui_context = Context::create();
	let mut platform = WinitPlatform::init(&mut imgui_context);

	// Add our font to imgui
	let hidpi_factor = platform.hidpi_factor();
	let font_size = (13.0 * hidpi_factor) as f32;
	imgui_context.fonts().add_font(&[
		FontSource::DefaultFontData {
			config: Some(FontConfig {
				size_pixels: font_size,
				..FontConfig::default()
			}),
		},
		FontSource::TtfData {
			data: include_bytes!("../../resources/OpenSans-Regular.ttf"),
			size_pixels: font_size,
			config: Some(FontConfig {
				rasterizer_multiply: 1.75,
				glyph_ranges: FontGlyphRanges::japanese(),
				..FontConfig::default()
			}),
		},
	]);
	imgui_context.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;

	// Fix `sRGB` on style colors
	let style = imgui_context.style_mut();
	#[allow(clippy::indexing_slicing)] // False positive, this is a `[f32; 4]`
	for color in style.colors.iter_mut() {
		color[0] = color[0].powf(2.2);
		color[1] = color[1].powf(2.2);
		color[2] = color[2].powf(2.2);
		color[3] = 1.0 - (1.0 - color[3]).powf(2.2);
	}

	// Attach the window to the platform
	let mut renderer = Renderer::init(&mut imgui_context, &mut factory, shaders).panic_err_msg("Unable to initialize renderer");
	platform.attach_window(imgui_context.io_mut(), windowed_context.window(), HiDpiMode::Rounded);

	let mut last_frame = std::time::Instant::now();
	let mut run = true;

	// Get all digimon names as `ImString`s
	let mut digimon_names: Vec<_> = cards_table.digimons.iter().map(|digimon| ImString::new(digimon.name.clone())).collect();
	let mut digimon_filter_idx = 0;
	let mut digimon_filter_search = ImString::new("");

	while run {
		// Check for events
		events_loop.poll_events(|event| {
			platform.handle_event(imgui_context.io_mut(), windowed_context.window(), &event);

			if let Event::WindowEvent { event, .. } = event {
				match event {
					WindowEvent::Resized(_) => {
						gfx_window_glutin::update_views(&windowed_context, &mut main_color, &mut main_depth);
					},
					WindowEvent::CloseRequested => run = false,
					_ => (),
				}
			}
		});

		// Prepare and execute ui frame
		let io = imgui_context.io_mut();
		platform
			.prepare_frame(io, windowed_context.window())
			.ignore_with_err(|err| log::warn!("Unable to prepare frame: {}", err));
		last_frame = io.update_delta_time(last_frame);
		let ui = imgui_context.frame();
		{
			ui.text("Digimon: ");

			ui.input_text(imgui::im_str!("Digimon Filter"), &mut digimon_filter_search)
				.resize_buffer(true)
				.build();

			let digimon_filters: Vec<_> = digimon_names
				.iter()
				.enumerate()
				.map(|(idx, string)| {
					(
						float_ord::FloatOrd(1.0 - strsim::jaro(string.to_str(), digimon_filter_search.to_str())),
						string,
						idx,
					)
				})
				.sorted()
				.collect();

			let digimon_filter_names: Vec<_> = digimon_filters
				.iter()
				.filter_map(|(value, string, _)| (digimon_filter_search.is_empty() || value.0 < 0.5).then_some(string))
				.collect();

			ui.list_box(
				imgui::im_str!("Digimon List"),
				&mut digimon_filter_idx,
				digimon_filter_names.as_slice(),
				4,
			);

			ui.separator();

			/*
			if let Some((_, _, idx)) = digimon_filters.get(digimon_filter_idx as usize) {
				let idx = *idx;
				std::mem::drop(digimon_filters);
				if let Some(digimon_name_buffer) = digimon_names.get_mut(idx) {
					if ui.input_text(imgui::im_str!("Name"), digimon_name_buffer).resize_buffer(true).build() {
						if let Some(digimon) = cards_table.digimons.get_mut(idx) {
							match ascii::AsciiString::from_ascii(digimon_name_buffer.to_string()) {
								Ok(name) => {
									digimon.name = name;
								},
								Err(err) => {
									ui.text(format!("Unable to set digimon name:\n{}", err.err_backtrace()));
								},
							}
						}
					}
				}
			}
			*/

			//if let Some(digimon_name) = digimon_names.iter_mut().enumerate().find(|(_, name)| name == )

			/*
			if let Some((idx, digimon)) = cards_table.digimons.iter_mut().enumerate().find(|(_, digimon)| {
				digimon_filter_names
					.get(digimon_filter_idx as usize)
					.map_or(false, |&name| name.to_str() == digimon.name)
			}) {
				let mut name_buffer = ImString::new(digimon.name.clone());
				if ui.input_text(imgui::im_str!("Name"), &mut name_buffer).resize_buffer(true).build() {
					match ascii::AsciiString::from_ascii(name_buffer.to_string()) {
						Ok(name) => {
							digimon.name = name;
							if let Some(name) = digimon_names.get_mut(idx) {
								*name = name_buffer.clone();
							}
						},
						Err(err) => {
							ui.text(format!("Unable to set digimon name:\n{}", err.err_backtrace()));
						},
					}
				}
			}
			*/
		}

		// Clear, render everything and flush it out
		encoder.clear(&main_color, [0.2, 0.2, 0.2, 1.0]);
		platform.prepare_render(&ui, windowed_context.window());
		renderer
			.render(&mut factory, &mut encoder, &mut main_color, ui.render())
			.ignore_with_err(|err| log::warn!("Unable to render: {}", err.err_backtrace()));
		encoder.flush(&mut device);
		windowed_context
			.swap_buffers()
			.ignore_with_err(|err| log::warn!("Unable to swap buffers: {}", err.err_backtrace()));
		device.cleanup();
	}

	// Convert everything back to yaml
	let cards_table_yaml = serde_yaml::to_string(&cards_table).panic_err_msg("Unable to serialize cards table to yaml");
	let decks_table_yaml = serde_yaml::to_string(&decks_table).panic_err_msg("Unable to serialize decks table to yaml");

	// Ouput all data to devices
	std::fs::write(&data_dir.join("cards.yaml"), cards_table_yaml).panic_err_msg("Unable to write cards table to file");
	std::fs::write(&data_dir.join("decks.yaml"), decks_table_yaml).panic_err_msg("Unable to write decks table to file");
}
