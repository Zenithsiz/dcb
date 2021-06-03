//! `.tmd` extractor

// Features
#![feature(format_args_capture, array_map)]

// Modules
mod cli;

// Imports
use anyhow::Context;
use byteorder::{LittleEndian, ReadBytesExt};
use cli::CliData;
use std::{
	fs,
	io::{self, Seek, SeekFrom},
	path::{Path, PathBuf},
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
	for input_file_path in &cli_data.input_files {
		// If we don't have an output, try the input filename with `.obj`
		let output_file = match &cli_data.output_file {
			Some(output) => output.to_path_buf(),
			None => {
				let mut path = input_file_path.as_os_str().to_os_string();
				path.push(".obj");
				PathBuf::from(path)
			},
		};

		// Then extract the tree
		if let Err(err) = self::extract_file(&input_file_path, &output_file) {
			log::error!("Unable to extract files from {}: {:?}", input_file_path.display(), err);
		}
	}

	Ok(())
}

/// Extracts a `.tmd` file to `output_dir`.
fn extract_file(input_file_path: &Path, _output_file: &Path) -> Result<(), anyhow::Error> {
	// Open the file and parse a `pak` filesystem from it.
	let input_file = fs::File::open(input_file_path).context("Unable to open input file")?;
	let mut input_file = io::BufReader::new(input_file);

	let _id = input_file.read_u32::<LittleEndian>()?;
	let _flags = input_file.read_u32::<LittleEndian>()?;
	let objs_len = input_file.read_u32::<LittleEndian>()?;

	let objs = (0..objs_len)
		.map(|_| {
			let obj: Obj = Obj::read(&mut input_file).context("Unable to read object")?;

			input_file
				.seek(SeekFrom::Start(0xc + u64::from(obj.vertices_ptr)))
				.context("Unable to seek")?;

			let vertices = (0..obj.vertices_len)
				.map(|idx| Vertex::read(&mut input_file).with_context(|| format!("Unable to read vertex {idx}")))
				.collect::<Result<Vec<_>, anyhow::Error>>()
				.context("Unable to read vertices")?;

			input_file
				.seek(SeekFrom::Start(0xc + u64::from(obj.normals_ptr)))
				.context("Unable to seek")?;

			let normals = (0..obj.normals_len)
				.map(|_| Normal::read(&mut input_file))
				.collect::<Result<Vec<_>, anyhow::Error>>()
				.context("Unable to read normals")?;

			input_file
				.seek(SeekFrom::Start(0xc + u64::from(obj.indices_ptr)))
				.context("Unable to seek")?;

			let index_header = IndexHeader::read(&mut input_file).context("Unable to read index header")?;
			let indices = (0..obj.indices_len)
				.map(|_| Index::read(&mut input_file, index_header))
				.collect::<Result<Vec<_>, anyhow::Error>>()
				.context("Unable to read indices")?;


			Ok((obj, vertices, normals, index_header, indices))
		})
		.collect::<Result<Vec<_>, anyhow::Error>>()
		.context("Unable to read objects")?;

	for (_obj, vertices, normals, _, indices) in objs {
		for Vertex { x, y, z } in vertices {
			println!("v {x} {y} {z}");
		}
		for Normal { x, y, z } in normals {
			println!("vn {x} {y} {z}");
		}
		for Index::Quad {
			uvs: _,
			vertices,
			normals,
		} in indices
		{
			let [v0, v1, v2, v3] = vertices.map(|v| v + 1);
			let [n0, n1, n2, n3] = normals.map(|n| n + 1);
			println!("f {v1}//{n0} {v0}//{n1} {v2}//{n2} {v3}//{n3}");
		}
	}

	//dbg!(&objs);

	Ok(())
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
struct Obj {
	pub vertices_ptr: u32,
	pub vertices_len: u32,
	pub normals_ptr:  u32,
	pub normals_len:  u32,
	pub indices_ptr:  u32,
	pub indices_len:  u32,
}

impl Obj {
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
	Quad {
		uvs:      [[u8; 2]; 4],
		vertices: [u16; 4],
		normals:  [u16; 4],
	},
}

impl Index {
	fn read<R: io::Read>(reader: &mut R, header: IndexHeader) -> Result<Self, anyhow::Error> {
		let index = match (header.olen, header.ilen, header.flag, header.mode) {
			(0xc, 0x8, 0x0, 0x3c) => {
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

				reader.read_u32::<LittleEndian>()?;

				Self::Quad {
					uvs:      [uv0, uv1, uv2, uv3],
					vertices: [v0, v1, v2, v3],
					normals:  [n0, n1, n2, n3],
				}
			},
			_ => return Err(anyhow::anyhow!("Unknown header")),
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
