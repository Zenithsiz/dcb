//! Filesystem

// Lints
// TODO: Check all usages and remove
#![allow(clippy::as_conversions)]

// Imports
use anyhow::Context;
use ascii::{AsciiChar, AsciiStr};
use dcb_drv::{DirEntry, DirEntryKind, DirPtr, FilePtr};
use fuser::Filesystem;
use itertools::Itertools;
use std::{
	collections::{hash_map, HashMap},
	convert::TryInto,
	ffi::{OsStr, OsString},
	fs,
	io::{Read, Seek, SeekFrom},
	os::unix::prelude::OsStrExt,
	time::{Duration, SystemTime},
};
use zutil::AsciiStrArr;


/// Drv filesystem
#[derive(Debug)]
pub struct DrvFs {
	/// Open file
	file: fs::File,

	/// Inodes
	inodes: HashMap<u64, Inode>,

	/// Ino by sector pos
	ino_by_sector_pos: HashMap<u32, u64>,
}

impl DrvFs {
	/// Time to live (?)
	const TTL: Duration = Duration::from_secs(1);

	/// Creates a new drv fs
	pub fn new(path: impl AsRef<std::path::Path>) -> Result<Self, anyhow::Error> {
		// Try to open the file
		let file = fs::File::open(path).context("Unable to open file")?;

		// Creates the inode maps
		let mut inodes_by_ino = HashMap::new();
		let mut ino_by_sector_pos = HashMap::new();

		// Insert the root inode
		inodes_by_ino.insert(1, Inode::root());
		ino_by_sector_pos.insert(0, 1);

		Ok(Self {
			file,
			inodes: inodes_by_ino,
			ino_by_sector_pos,
		})
	}
}

impl DrvFs {
	fn lookup(
		&mut self, _req: &fuser::Request<'_>, parent: u64, name: &std::ffi::OsStr,
	) -> Result<fuser::FileAttr, anyhow::Error> {
		// Get the parent inode
		let parent_inode = self.inodes.get(&parent).context("Unable to get inode")?;

		// It it's not a directory, return Err
		let dir = match parent_inode.kind {
			InodeKind::Dir { ptr } => ptr,
			_ => anyhow::bail!("Cannot lookup non-directories"),
		};

		// Get the name
		let name = AsciiStr::from_ascii(name.as_bytes()).context("Unable to get name as ascii")?;

		// Then find the entry
		let (_, entry) = dir.find_entry(&mut self.file, name).context("Unable to find entry")?;

		let ino = self
			.ino_by_sector_pos
			.get(&entry.kind.sector_pos())
			.context("Unable to get ino by sector pos")?;
		let inode = self.inodes.get(ino).context("Unable to get inode by ino")?;
		Ok(inode.attr())
	}

	fn get_attr(&mut self, _req: &fuser::Request<'_>, ino: u64) -> Result<fuser::FileAttr, anyhow::Error> {
		// Get the inode
		let inode = self.inodes.get(&ino).context("Unable to get inode")?;

		Ok(inode.attr())
	}

	// TODO: Not return a `Vec<u8>` of data
	#[allow(clippy::too_many_arguments)] // It's what we receive from `fuse`.
	fn read(
		&mut self, _req: &fuser::Request<'_>, ino: u64, _fh: u64, offset: i64, size: u32, _flags: i32,
		_lock_owner: Option<u64>,
	) -> Result<Vec<u8>, anyhow::Error> {
		// Get the inode
		let inode = self.inodes.get(&ino).context("Unable to get inode")?;

		// It it's not a file, return Err
		let file = match inode.kind {
			InodeKind::File { ptr } => ptr,
			_ => anyhow::bail!("Cannot read non-files"),
		};

		// Then create the cursor
		let mut cursor = file.cursor(&mut self.file).context("Unable to create cursor")?;

		// Then seek and read
		let offset = u64::min(offset as u64, u64::from(file.size));
		cursor
			.seek(SeekFrom::Start(offset))
			.context("Unable to seek to offset")?;
		let mut size = usize::min(size as usize, file.size as usize);
		if offset as usize + size > file.size as usize {
			size = file.size as usize - offset as usize;
		}

		let mut data = vec![0; size];
		cursor.read_exact(&mut data).context("Unable to read data")?;

		Ok(data)
	}

	fn read_dir(
		&mut self, _req: &fuser::Request, ino: u64, _fh: u64, offset: i64,
		mut add_entry: impl FnMut(u64, i64, fuser::FileType, AsciiStrArr<0x14>) -> bool,
	) -> Result<(), anyhow::Error> {
		// Get the inode
		let inode = self.inodes.get(&ino).context("Unable to get inode")?;

		// It it's not a directory, return Err
		let dir = match inode.kind {
			InodeKind::Dir { ptr } => ptr,
			_ => anyhow::bail!("Cannot read-dir non-directories"),
		};

		// Then read all entries
		// TODO: Skipping here still parses all entries, make `DirPtr::read_entries` return a proper
		//       iterator in order to implement `skip`.
		let entries = dir.read_entries(&mut self.file).context("Unable to read entries")?;
		for (entry, idx) in entries.skip(offset as usize).zip(0..) {
			let dir_entry: DirEntry = entry.context("Unable to read entry")?;

			// Get the name
			let mut name: AsciiStrArr<0x14> = AsciiStrArr::new();
			for &ch in dir_entry.name.as_ascii() {
				name.push(ch).expect("Must have enough space");
			}
			if let DirEntryKind::File { extension, .. } = dir_entry.kind {
				name.push(AsciiChar::Dot).expect("Must have enough space");
				for &ch in extension.as_ascii() {
					name.push(ch).expect("Must have enough space");
				}
			}

			// Then get it's inode
			let sector_pos = dir_entry.kind.sector_pos();
			let ino = match self.ino_by_sector_pos.entry(sector_pos) {
				hash_map::Entry::Occupied(entry) => *entry.get(),
				hash_map::Entry::Vacant(entry) => {
					// Create the new inode
					let inode = Inode {
						ino:   self.inodes.len() as u64 + 1,
						_name: OsStr::from_bytes(name.as_bytes()).to_os_string(),
						date:  SystemTime::UNIX_EPOCH + Duration::from_secs(dir_entry.date.timestamp() as u64),
						kind:  InodeKind::from(dir_entry.kind),
					};

					// TODO: Maybe use an `HashSet`?
					let ino = inode.ino;
					assert!(self.inodes.insert(ino, inode).is_none(), "Inode already existed");

					*entry.insert(ino)
				},
			};

			let file_kind = InodeKind::from(dir_entry.kind).file_type();
			if add_entry(ino, idx + 1, file_kind, name) {
				break;
			}
		}

		Ok(())
	}

	fn statfs(&mut self, _req: &fuser::Request<'_>, _ino: u64) -> Result<StatFs, anyhow::Error> {
		// Get the total file length
		let len = self.file.stream_len().context("Unable to get file len")?;

		// Then count the number of files
		fn count_files(dir_ptr: DirPtr, file: &mut fs::File) -> Result<usize, anyhow::Error> {
			let dir_entries = dir_ptr
				.read_entries(file)
				.context("Unable to read directory")?
				.try_collect::<_, Vec<_>, _>()
				.context("Unable to read entries")?;

			let mut total = 0;
			for entry in dir_entries {
				total += match entry.kind {
					DirEntryKind::File { .. } => 1,
					DirEntryKind::Dir { ptr } => 1 + count_files(ptr, file).context("Unable to count files")?,
				};
			}

			Ok(total)
		}
		let files = count_files(DirPtr::root(), &mut self.file).context("Unable to count files")?;

		Ok(StatFs {
			blocks:           (len + 0x7ff) / 0x800,
			blocks_free:      0,
			blocks_available: 0,
			files:            files.try_into().context("Unable to get file count as `u64`")?,
			files_free:       0,
			block_size:       0x800,
			name_len:         0x10,
			fragment_size:    0x800,
		})
	}
}

impl Filesystem for DrvFs {
	fn lookup(&mut self, req: &fuser::Request<'_>, parent: u64, name: &std::ffi::OsStr, reply: fuser::ReplyEntry) {
		match self.lookup(req, parent, name) {
			Ok(attr) => reply.entry(&Self::TTL, &attr, 0),
			Err(err) => {
				log::error!("Unable to lookup {name:?}@{parent}: {err:?}");
				reply.error(libc::ENOENT);
			},
		}
	}

	fn getattr(&mut self, req: &fuser::Request<'_>, ino: u64, reply: fuser::ReplyAttr) {
		match self.get_attr(req, ino) {
			Ok(attr) => reply.attr(&Self::TTL, &attr),
			Err(err) => {
				log::error!("Unable to get attributes for {ino}: {err:?}");
				reply.error(libc::ENOENT);
			},
		}
	}

	fn read(
		&mut self, req: &fuser::Request<'_>, ino: u64, fh: u64, offset: i64, size: u32, flags: i32,
		lock_owner: Option<u64>, reply: fuser::ReplyData,
	) {
		match self.read(req, ino, fh, offset, size, flags, lock_owner) {
			Ok(data) => reply.data(&data),
			Err(err) => {
				log::error!("Unable to read {offset}/{size}@{ino}: {err:?}");
				reply.error(libc::ENOENT);
			},
		}
	}

	fn readdir(&mut self, req: &fuser::Request, ino: u64, fh: u64, offset: i64, mut reply: fuser::ReplyDirectory) {
		let new_entry = |ino, offset, kind, name: AsciiStrArr<0x14>| reply.add(ino, offset, kind, name.as_str());

		match self.read_dir(req, ino, fh, offset, new_entry) {
			Ok(()) => reply.ok(),
			Err(err) => {
				log::error!("Unable to read directory {ino}/{offset}: {err:?}");
				reply.error(libc::ENOENT);
			},
		}
	}

	fn statfs(&mut self, req: &fuser::Request<'_>, ino: u64, reply: fuser::ReplyStatfs) {
		match self.statfs(req, ino) {
			Ok(stats) => reply.statfs(
				stats.blocks,
				stats.blocks_free,
				stats.blocks_available,
				stats.files,
				stats.files_free,
				stats.block_size,
				stats.name_len,
				stats.fragment_size,
			),
			Err(err) => {
				log::error!("Unable to get stats for {ino}: {err:?}");
				reply.error(libc::ENOENT);
			},
		}
	}
}

/// Inode
#[derive(Debug)]
pub struct Inode {
	/// Inode number
	ino: u64,

	/// Name
	_name: OsString,

	/// Date
	date: SystemTime,

	/// Kind
	kind: InodeKind,
}

impl Inode {
	/// Returns the root inode
	pub fn root() -> Self {
		Self {
			ino:   1,
			_name: OsString::from("<root>"),
			date:  SystemTime::now(),
			kind:  InodeKind::Dir { ptr: DirPtr::root() },
		}
	}

	/// Returns the attributes of this inode
	pub fn attr(&self) -> fuser::FileAttr {
		fuser::FileAttr {
			ino:     self.ino,
			size:    match self.kind {
				InodeKind::File { ptr } => u64::from(ptr.size),
				InodeKind::Dir { .. } => 0,
			},
			blocks:  match self.kind {
				InodeKind::File { ptr } => u64::from((ptr.size + 0x7ff) / 0x800),
				// TODO: Report actual number of entries here, instead of assuming 1
				InodeKind::Dir { .. } => 1,
			},
			atime:   self.date,
			mtime:   self.date,
			ctime:   self.date,
			crtime:  self.date,
			kind:    self.kind.file_type(),
			perm:    0o444,
			nlink:   1,
			uid:     1000,
			gid:     1001,
			rdev:    0,
			flags:   0,
			blksize: 0x800,
		}
	}
}

/// Inode kind
#[derive(Debug)]
pub enum InodeKind {
	/// File
	File { ptr: FilePtr },

	/// Directory
	Dir { ptr: DirPtr },
}

impl InodeKind {
	fn file_type(&self) -> fuser::FileType {
		match self {
			InodeKind::File { .. } => fuser::FileType::RegularFile,
			InodeKind::Dir { .. } => fuser::FileType::Directory,
		}
	}
}

impl From<DirEntryKind> for InodeKind {
	fn from(kind: DirEntryKind) -> Self {
		match kind {
			DirEntryKind::File { ptr, .. } => Self::File { ptr },
			DirEntryKind::Dir { ptr } => Self::Dir { ptr },
		}
	}
}

/// Filesystem stats
struct StatFs {
	pub blocks:           u64,
	pub blocks_free:      u64,
	pub blocks_available: u64,
	pub files:            u64,
	pub files_free:       u64,
	pub block_size:       u32,
	pub name_len:         u32,
	pub fragment_size:    u32,
}
