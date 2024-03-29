//! Filesystem tree

// Imports
use anyhow::Context;
use dcb_drv::{DirEntry, DirEntryKind, DirPtr, FilePtr};
use eframe::egui;
use std::io;
use zutil::{AsciiStrArr, StrContainsCaseInsensitive};

/// Drv tree
pub struct DrvTree {
	/// Root directory
	root: TreeDir,
}

impl DrvTree {
	/// Creates a new tree
	pub fn new<R: io::Seek + io::Read>(reader: &mut R) -> Result<Self, anyhow::Error> {
		// Parse the root directory
		let root = Self::parse_dir(reader, DirPtr::root())?;

		Ok(Self { root })
	}

	/// Reloads this tree
	pub fn reload<R: io::Seek + io::Read>(&mut self, reader: &mut R) -> Result<(), anyhow::Error> {
		self.root = Self::parse_dir(reader, DirPtr::root())?;

		Ok(())
	}

	/// Helper function to parse a directory
	fn parse_dir<R: io::Read + io::Seek>(reader: &mut R, ptr: DirPtr) -> Result<TreeDir, anyhow::Error> {
		// Read all entries
		let entries: Vec<DirEntry> = ptr
			.read_entries(reader)
			.with_context(|| format!("Unable to read entries for {:#x}", ptr.sector_pos))?
			.collect::<Result<_, _>>()
			.with_context(|| format!("Unable to read entry for {:#x}", ptr.sector_pos))?;

		// Then convert all dir entries to our entries
		let mut entries: Vec<_> = entries
			.into_iter()
			.map(|entry| {
				let kind = match entry.kind {
					DirEntryKind::Dir { ptr } => {
						let dir = Self::parse_dir(reader, ptr)?;
						TreeDirEntryKind::Dir(dir)
					},
					DirEntryKind::File { extension, ptr } => TreeDirEntryKind::File { extension, ptr },
				};

				Ok(TreeDirEntry { name: entry.name, kind })
			})
			.collect::<Result<_, anyhow::Error>>()?;

		// And sort them by folder and name
		entries.sort_unstable_by_key(|entry| (entry.kind.is_file(), entry.name));

		Ok(TreeDir { entries })
	}

	/// Displays this tree
	pub fn display(&self, ui: &mut egui::Ui, start_path: &str, ctx: &mut DisplayCtx<impl FnMut(&str)>) {
		self.root.display(ui, start_path, ctx);
	}
}

/// Tree directory
pub struct TreeDir {
	/// Pointer
	//ptr: DirPtr,

	/// All entries
	entries: Vec<TreeDirEntry>,
}

impl TreeDir {
	/// Displays this directory
	pub fn display(&self, ui: &mut egui::Ui, cur_path: &str, ctx: &mut DisplayCtx<impl FnMut(&str)>) {
		for entry in &self.entries {
			match &entry.kind {
				TreeDirEntryKind::Dir(dir) => {
					let cur_path = format!("{cur_path}{}\\", entry.name);
					egui::CollapsingHeader::new(entry.name)
						.id_source(dir as *const _)
						.show(ui, |ui| {
							dir.display(ui, &cur_path, ctx);
						});
				},
				TreeDirEntryKind::File { extension, .. } => {
					let filename = format!("{}.{}", entry.name, extension);
					let path = format!("{cur_path}{filename}");
					if !filename.contains_case_insensitive(ctx.search_str) {
						continue;
					}

					if ui.button(filename).clicked() {
						(ctx.on_file_click)(&path);
					}
				},
			}
		}
	}
}

/// Tree directory entry
pub struct TreeDirEntry {
	/// Name
	name: AsciiStrArr<0x10>,

	/// Date
	//date: NaiveDateTime,

	/// Kind
	kind: TreeDirEntryKind,
}

/// Tree directory entry kind
pub enum TreeDirEntryKind {
	/// File
	File {
		/// Extension
		extension: AsciiStrArr<0x3>,

		/// File pointer
		ptr: FilePtr,
	},

	/// Directory
	Dir(TreeDir),
}

impl TreeDirEntryKind {
	/// Returns `true` if the entry is a directory.
	#[must_use]
	pub fn is_dir(&self) -> bool {
		matches!(self, Self::Dir(..))
	}

	/// Returns `true` if the entry is a file.
	#[must_use]
	pub fn is_file(&self) -> bool {
		matches!(self, Self::File { .. })
	}
}

/// Display context
//#[derive(Copy)]
pub struct DisplayCtx<'a, OnFileClick> {
	/// Search string
	pub search_str: &'a str,

	/// Callback for file click
	pub on_file_click: OnFileClick,
}
