//! `.tmd` extractor

// Features
#![feature(format_args_capture, array_map, seek_stream_len)]

// Modules
mod cli;

// Imports
use anyhow::Context;
use byteorder::{ByteOrder, LittleEndian, ReadBytesExt};
use cli::CliData;
use dcb_util::{AsciiStrArr, IoCursor, NullAsciiString};
use std::{
	convert::TryFrom,
	fs,
	io::{self, SeekFrom},
	path::Path,
};


fn main() -> Result<(), anyhow::Error> {
	// Initialize the logger
	simplelog::TermLogger::init(
		log::LevelFilter::Info,
		simplelog::Config::default(),
		simplelog::TerminalMode::Stderr,
	)
	.expect("Unable to initialize logger");

	// Get all data from cli
	let cli_data = CliData::new();

	// For each input file, extract it
	for input_path in &cli_data.input_files {
		// If we don't have an output, try the input filename with `.obj`
		let output_dir = match &cli_data.output_dir {
			Some(output) => output.to_path_buf(),
			None => {
				input_path.with_extension("")
				//let mut path = input_path.as_os_str().to_os_string();
				//path.push(".obj");
				//PathBuf::from(path)
			},
		};

		// Then extract the tree
		if let Err(err) = self::extract_file(input_path, &output_dir) {
			log::error!("Unable to extract files from {}: {:?}", input_path.display(), err);
		}
	}

	Ok(())
}

/// Extracts a `.tmd` file to `output_dir`.
fn extract_file(input_path: &Path, output_dir: &Path) -> Result<(), anyhow::Error> {
	// Open the file and parse a `pak` filesystem from it.
	let file = fs::File::open(input_path).context("Unable to open input file")?;
	let mut file = io::BufReader::new(file);

	let m3d: M3d = M3d::read(&mut file).context("Unable to read m3d")?;

	dcb_util::try_create_folder(output_dir).context("Unable to create output directory")?;

	let mut output = fs::File::create(output_dir.join("final.obj")).context("Unable to create output file")?;

	let mut vertex_idx_offset = 0;
	let mut normal_idx_offset = 0;
	for (tmd, entry) in m3d.tmds.iter().zip(&m3d.entries) {
		let offset = [entry.x, entry.y, entry.z];

		tmd.objs[0]
			.display(&mut output, offset, entry.len, vertex_idx_offset, normal_idx_offset)
			.context("Unable to write object")?;

		vertex_idx_offset += u16::try_from(tmd.objs[0].vertices.len()).expect("Too many vertices");
		normal_idx_offset += u16::try_from(tmd.objs[0].normals.len()).expect("Too many vertices");
	}

	Ok(())
}

#[derive(PartialEq, Eq, Clone, Debug)]
struct M3d {
	/// Name
	name: AsciiStrArr<0xb>,

	/// Flags
	flags: u32,

	/// Entry
	entries: Vec<M3dEntry>,

	/// Tmds
	tmds: Vec<Tmd>,
}

impl M3d {
	fn read<R: io::Seek + io::Read>(mut reader: &mut R) -> Result<Self, anyhow::Error> {
		let mut name_bytes = [0; 0xc];
		reader.read_exact(&mut name_bytes).context("Unable to read name")?;
		let name = name_bytes.read_string().context("Unable to parse name")?;

		let flags = reader.read_u32::<LittleEndian>().context("Unable to read flags")?;

		match flags {
			0x0 | 0x6e5270 | 0xbff7b77b | 0xbff7b76c => (),
			_ => anyhow::bail!("Unknown flags {:#x}", flags),
		};

		let entries_len = reader
			.read_u32::<LittleEndian>()
			.context("Unable to read entries len")?;

		reader
			.seek(SeekFrom::Start(0x34))
			.context("Unable to seek to entries")?;

		let entries = (0..entries_len)
			.map(|idx| M3dEntry::read(reader).with_context(|| format!("Unable to read entry {idx}")))
			.collect::<Result<Vec<_>, anyhow::Error>>()
			.context("Unable to read entries")?;

		let tmds = (0..entries_len)
			.map(|idx| {
				let start_pos = reader.stream_position().context("Unable to get reader's position")?;
				let mut file =
					IoCursor::new(&mut reader, start_pos, u64::MAX).context("Unable to create inner file")?;

				Tmd::read(&mut file).with_context(|| format!("Unable to parse tmd {idx}@{start_pos:#x}"))
			})
			.collect::<Result<Vec<_>, anyhow::Error>>()
			.context("Unable to read tmds")?;

		Ok(Self {
			name,
			flags,
			entries,
			tmds,
		})
	}
}

/// M3d entry
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
struct M3dEntry {
	x:   i16,
	y:   i16,
	z:   i16,
	len: u32,
}

impl M3dEntry {
	fn read<R: io::Seek + io::Read>(reader: &mut R) -> Result<Self, anyhow::Error> {
		let mut bytes = [0; 0xc];
		reader.read_exact(&mut bytes).context("Unable to read entry")?;

		let bytes = dcb_util::array_split!(&bytes,
			len: [0x4],
			x: [0x2],
			y: [0x2],
			z: [0x2],
			zero: [0x2],
		);

		let x = LittleEndian::read_i16(bytes.x);
		let y = LittleEndian::read_i16(bytes.y);
		let z = LittleEndian::read_i16(bytes.z);
		let len = LittleEndian::read_u32(bytes.len);

		println!("{x:04x}|{y:04x}|{z:04x}|{len:08x}");

		anyhow::ensure!(*bytes.zero == [0; 2], "Zero wasn't 0");

		Ok(Self { x, y, z, len })
	}
}


#[derive(PartialEq, Eq, Clone, Debug)]
struct Tmd {
	/// All objects
	objs: Vec<Obj>,
}

impl Tmd {
	fn read<R: io::Seek + io::Read>(reader: &mut R) -> Result<Self, anyhow::Error> {
		let id = reader.read_u32::<LittleEndian>()?;
		let flags = reader.read_u32::<LittleEndian>()?;
		let objs_len = reader.read_u32::<LittleEndian>()?;

		anyhow::ensure!(id == 0x41, "Id was wrong");
		anyhow::ensure!(flags == 0x0, "Unknown flags");
		anyhow::ensure!(objs_len != 0x0, "No objects");

		let objs = (0..objs_len)
			.map(|_| {
				let header: ObjHeader = ObjHeader::read(reader).context("Unable to read object")?;

				let obj: Obj = Obj::read(reader, header).context("Unable to parse object")?;

				Ok(obj)
			})
			.collect::<Result<Vec<_>, anyhow::Error>>()
			.context("Unable to get all objects")?;

		Ok(Self { objs })
	}
}

#[derive(PartialEq, Eq, Clone, Debug)]
struct Obj {
	/// Vertices
	vertices: Vec<Vertex>,
	/// Normals
	normals:  Vec<Normal>,
	/// Indices
	indices:  Vec<Index>,
}

impl Obj {
	// TODO: Always leave reader at end instead of assuming it ends a at `normals`
	fn read<R: io::Seek + io::Read>(reader: &mut R, header: ObjHeader) -> Result<Self, anyhow::Error> {
		assert!(header.indices_ptr < header.vertices_ptr && header.vertices_ptr < header.normals_ptr);

		reader
			.seek(SeekFrom::Start(0xc + u64::from(header.indices_ptr)))
			.context("Unable to seek")?;

		let indices = (0..header.indices_len)
			.map(|_| Index::read(reader))
			.collect::<Result<Vec<_>, anyhow::Error>>()
			.context("Unable to read indices")?;

		reader
			.seek(SeekFrom::Start(0xc + u64::from(header.vertices_ptr)))
			.context("Unable to seek")?;

		let vertices = (0..header.vertices_len)
			.map(|idx| Vertex::read(reader).with_context(|| format!("Unable to read vertex {idx}")))
			.collect::<Result<Vec<_>, anyhow::Error>>()
			.context("Unable to read vertices")?;

		reader
			.seek(SeekFrom::Start(0xc + u64::from(header.normals_ptr)))
			.context("Unable to seek")?;

		let normals = (0..header.normals_len)
			.map(|_| Normal::read(reader))
			.collect::<Result<Vec<_>, anyhow::Error>>()
			.context("Unable to read normals")?;

		Ok(Self {
			vertices,
			normals,
			indices,
		})
	}

	pub fn display<W: io::Write>(
		&self, writer: &mut W, offset: [i16; 3], mode: u32, vertex_idx_offset: u16, normal_idx_offset: u16,
	) -> Result<(), anyhow::Error> {
		for &Vertex { x, y, z } in &self.vertices {
			/*
			let pos = Vector3::new(x as f32, y as f32, z as f32);

			let pos = cgmath::Matrix3::from_angle_x(Rad(offset[0] as f32)) *
				cgmath::Matrix3::from_angle_y(Rad(offset[1] as f32)) *
				cgmath::Matrix3::from_angle_z(Rad(offset[2] as f32)) *
				pos;

			let [x, y, z] = [pos.x, pos.y, pos.z].map(|c| c as i16);
			*/
			let [x, y, z] = match mode {
				0x0 => [x + offset[0], y + offset[1], z + offset[2]],
				0xffff_ffff => [x, y, z],

				_ => {
					log::warn!("Unknown mode {mode:#x}");
					[0, 0, 0]
					//continue;
				},
			};


			writeln!(writer, "v {x} {y} {z}")?;
		}
		for Normal { x, y, z } in &self.normals {
			writeln!(writer, "vn {x} {y} {z}")?;
		}
		for index in &self.indices {
			match index {
				Index::Quad {
					uvs: _,
					vertices,
					normals,
				} => {
					let [v0, v1, v2, v3] = vertices.map(|v| vertex_idx_offset + v + 1);
					let [n0, n1, n2, n3] = normals.map(|n| normal_idx_offset + n + 1);
					writeln!(writer, "f {v1}//{n0} {v0}//{n1} {v2}//{n2} {v3}//{n3}")?;
				},

				Index::Tri {
					uvs: _,
					vertices,
					normals,
				} => {
					let [v0, v1, v2] = vertices.map(|v| vertex_idx_offset + v + 1);
					let [n0, n1, n2] = normals.map(|n| normal_idx_offset + n + 1);
					writeln!(writer, "f {v1}//{n0} {v0}//{n1} {v2}//{n2}")?;
				},
			}
		}

		Ok(())
	}
}


#[derive(PartialEq, Eq, Clone, Copy, Debug)]
struct ObjHeader {
	pub vertices_ptr: u32,
	pub vertices_len: u32,
	pub normals_ptr:  u32,
	pub normals_len:  u32,
	pub indices_ptr:  u32,
	pub indices_len:  u32,
}

impl ObjHeader {
	fn read<R: io::Read>(reader: &mut R) -> Result<Self, anyhow::Error> {
		let vertices_ptr = reader.read_u32::<LittleEndian>()?;
		let vertices_len = reader.read_u32::<LittleEndian>()?;
		let normals_ptr = reader.read_u32::<LittleEndian>()?;
		let normals_len = reader.read_u32::<LittleEndian>()?;
		let indices_ptr = reader.read_u32::<LittleEndian>()?;
		let indices_len = reader.read_u32::<LittleEndian>()?;

		Ok(Self {
			vertices_ptr,
			vertices_len,
			normals_ptr,
			normals_len,
			indices_ptr,
			indices_len,
		})
	}
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
struct IndexHeader {
	pub olen: u8,
	pub ilen: u8,
	pub flag: u8,
	pub mode: u8,
}

impl IndexHeader {
	fn read<R: io::Read>(reader: &mut R) -> Result<Self, anyhow::Error> {
		let olen = reader.read_u8()?;
		let ilen = reader.read_u8()?;
		let flag = reader.read_u8()?;
		let mode = reader.read_u8()?;

		Ok(Self { olen, ilen, flag, mode })
	}
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum Index {
	Tri {
		uvs:      [[u8; 2]; 3],
		vertices: [u16; 3],
		normals:  [u16; 3],
	},

	Quad {
		uvs:      [[u8; 2]; 4],
		vertices: [u16; 4],
		normals:  [u16; 4],
	},
}

impl Index {
	fn read<R: io::Read>(reader: &mut R) -> Result<Self, anyhow::Error> {
		let header = IndexHeader::read(reader).context("Unable to read header")?;
		let index = match (header.olen, header.ilen, header.flag, header.mode) {
			(0xc, 0x8, 0x0, 0x3c) => {
				let mut bytes = [0; 32];
				reader.read_exact(&mut bytes)?;
				let bytes = dcb_util::array_split!(&bytes,
					uv0: [0x2],
					_unknown0: [0x2],
					uv1: [0x2],
					_unknown1: [0x2],
					uv2: [0x2],
					_unknown2: [0x2],
					uv3: [0x2],
					_unknown3: [0x2],
					n0: [0x2],
					v0: [0x2],
					n1: [0x2],
					v1: [0x2],
					n2: [0x2],
					v2: [0x2],
					n3: [0x2],
					v3: [0x2],
				);

				let uv0 = *bytes.uv0;
				let uv1 = *bytes.uv1;
				let uv2 = *bytes.uv2;
				let uv3 = *bytes.uv3;

				let n0 = LittleEndian::read_u16(bytes.n0);
				let v0 = LittleEndian::read_u16(bytes.v0);
				let n1 = LittleEndian::read_u16(bytes.n1);
				let v1 = LittleEndian::read_u16(bytes.v1);
				let n2 = LittleEndian::read_u16(bytes.n2);
				let v2 = LittleEndian::read_u16(bytes.v2);
				let n3 = LittleEndian::read_u16(bytes.n3);
				let v3 = LittleEndian::read_u16(bytes.v3);

				Self::Quad {
					uvs:      [uv0, uv1, uv2, uv3],
					vertices: [v0, v1, v2, v3],
					normals:  [n0, n1, n2, n3],
				}
			},
			(0xc, 0x8, 0x0, 0x3e) => {
				let uv0 = [reader.read_u8()?, reader.read_u8()?];
				let _ = reader.read_u16::<LittleEndian>();
				let uv1 = [reader.read_u8()?, reader.read_u8()?];
				let _ = reader.read_u16::<LittleEndian>()?;
				let uv2 = [reader.read_u8()?, reader.read_u8()?];
				let _ = reader.read_u16::<LittleEndian>()?;
				let uv3 = [reader.read_u8()?, reader.read_u8()?];
				let _ = reader.read_u16::<LittleEndian>()?;

				let n0 = reader.read_u16::<LittleEndian>()?;
				let v0 = reader.read_u16::<LittleEndian>()?;
				let n1 = reader.read_u16::<LittleEndian>()?;
				let v1 = reader.read_u16::<LittleEndian>()?;
				let n2 = reader.read_u16::<LittleEndian>()?;
				let v2 = reader.read_u16::<LittleEndian>()?;
				let n3 = reader.read_u16::<LittleEndian>()?;
				let v3 = reader.read_u16::<LittleEndian>()?;

				Self::Quad {
					uvs:      [uv0, uv1, uv2, uv3],
					vertices: [v0, v1, v2, v3],
					normals:  [n0, n1, n2, n3],
				}
			},
			(0x9, 0x6, 0x0, 0x36) => {
				let uv0 = [reader.read_u8()?, reader.read_u8()?];
				let _ = reader.read_u16::<LittleEndian>();
				let uv1 = [reader.read_u8()?, reader.read_u8()?];
				let _ = reader.read_u16::<LittleEndian>()?;
				let uv2 = [reader.read_u8()?, reader.read_u8()?];
				let _ = reader.read_u16::<LittleEndian>()?;

				let n0 = reader.read_u16::<LittleEndian>()?;
				let v0 = reader.read_u16::<LittleEndian>()?;
				let n1 = reader.read_u16::<LittleEndian>()?;
				let v1 = reader.read_u16::<LittleEndian>()?;
				let n2 = reader.read_u16::<LittleEndian>()?;
				let v2 = reader.read_u16::<LittleEndian>()?;

				Self::Tri {
					uvs:      [uv0, uv1, uv2],
					vertices: [v0, v1, v2],
					normals:  [n0, n1, n2],
				}
			},
			(0x9, 0x6, 0x0, 0x34) => {
				let uv0 = [reader.read_u8()?, reader.read_u8()?];
				let _ = reader.read_u16::<LittleEndian>();
				let uv1 = [reader.read_u8()?, reader.read_u8()?];
				let _ = reader.read_u16::<LittleEndian>()?;
				let uv2 = [reader.read_u8()?, reader.read_u8()?];
				let _ = reader.read_u16::<LittleEndian>()?;
				let n0 = reader.read_u16::<LittleEndian>()?;
				let v0 = reader.read_u16::<LittleEndian>()?;
				let n1 = reader.read_u16::<LittleEndian>()?;
				let v1 = reader.read_u16::<LittleEndian>()?;
				let n2 = reader.read_u16::<LittleEndian>()?;
				let v2 = reader.read_u16::<LittleEndian>()?;

				Self::Tri {
					uvs:      [uv0, uv1, uv2],
					vertices: [v0, v1, v2],
					normals:  [n0, n1, n2],
				}
			},
			_ => return Err(anyhow::anyhow!("Unknown header {:?}", header)),
		};

		Ok(index)
	}
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
struct Vertex {
	pub x: i16,
	pub y: i16,
	pub z: i16,
}

impl Vertex {
	fn read<R: io::Read>(reader: &mut R) -> Result<Self, anyhow::Error> {
		let x = reader.read_i16::<LittleEndian>()?;
		let y = reader.read_i16::<LittleEndian>()?;
		let z = reader.read_i16::<LittleEndian>()?;
		let _pad = reader.read_i16::<LittleEndian>()?;

		Ok(Self { x, y, z })
	}
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
struct Normal {
	pub x: i16,
	pub y: i16,
	pub z: i16,
}

impl Normal {
	fn read<R: io::Read>(reader: &mut R) -> Result<Self, anyhow::Error> {
		let x = reader.read_i16::<LittleEndian>()?;
		let y = reader.read_i16::<LittleEndian>()?;
		let z = reader.read_i16::<LittleEndian>()?;
		let _pad = reader.read_i16::<LittleEndian>()?;

		Ok(Self { x, y, z })
	}
}
