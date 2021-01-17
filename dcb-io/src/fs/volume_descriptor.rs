//! A volume descriptor

// Modules
pub mod error;
pub mod type_code;

// Exports
pub use error::{FromBytesError, ParseBootRecordError, ParsePrimaryError};
pub use type_code::TypeCode;

// Imports
use super::{date_time::DecDateTime, StrArrA, StrArrD};
use byteorder::{ByteOrder, LittleEndian};
use dcb_bytes::Bytes;
use dcb_util::array_split;

/// A volume descriptor
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum VolumeDescriptor {
	/// Boot record
	BootRecord {
		/// System Id
		system_id: StrArrA<0x20>,

		/// Boot identifier
		boot_id: StrArrA<0x20>,

		/// Data
		data: [u8; 0x7b9],
	},

	/// Primary
	Primary {
		/// System Id
		system_id: StrArrA<0x20>,

		/// Volume Id
		volume_id: StrArrD<0x20>,

		/// Volume space size
		volume_space_size: u32,

		/// Volume sequence_number
		volume_sequence_number: u16,

		/// Logical block size
		logical_block_size: u16,

		/// Path table size
		path_table_size: u32,

		/// Path table location
		path_table_location: u32,

		/// Path table optional location
		path_table_opt_location: u32,

		/// Root directory entry
		root_dir_entry: [u8; 0x22],

		/// Volume set identifier
		volume_set_id: StrArrD<0x80>,

		/// Publisher identifier
		publisher_id: StrArrA<0x80>,

		/// Data preparer identifier
		data_preparer_id: StrArrA<0x80>,

		/// Application identifier
		application_id: StrArrA<0x80>,

		/// Copyright file identifier
		copyright_file_id: StrArrD<0x26>,

		/// Abstract file identifier
		abstract_file_id: StrArrD<0x24>,

		/// Bibliographic file identifier
		bibliographic_file_id: StrArrD<0x25>,

		/// Volume creation date time
		volume_creation_date_time: DecDateTime,

		/// Volume modification date time
		volume_modification_date_time: DecDateTime,

		/// Volume expiration date time
		volume_expiration_date_time: DecDateTime,

		/// Volume effective date time
		volume_effective_date_time: DecDateTime,
	},
}

impl VolumeDescriptor {
	/// Magic
	pub const MAGIC: [u8; 5] = *b"CD001";
	/// Version
	pub const VERSION: u8 = 0x1;
}

impl VolumeDescriptor {
	/// Parses a boot record volume descriptor
	pub fn parse_boot_record(bytes: &[u8; 0x7f9]) -> Result<Self, ParseBootRecordError> {
		let bytes = array_split!(bytes,
			system_id: [0x20],
			boot_id  : [0x20],
			data     : [0x7b9],
		);

		// Parse both ids
		let system_id = StrArrA::from_bytes(bytes.system_id).map_err(ParseBootRecordError::SystemId)?;
		let boot_id = StrArrA::from_bytes(bytes.boot_id).map_err(ParseBootRecordError::BootId)?;

		Ok(Self::BootRecord {
			system_id,
			boot_id,
			data: *bytes.data,
		})
	}

	/// Parses a primary volume descriptor
	pub fn parse_primary(bytes: &[u8; 0x7f9]) -> Result<Self, ParsePrimaryError> {
		let bytes = array_split!(bytes,
			zeroes0                      :  0x1,
			system_id                    : [0x20],
			volume_id                    : [0x20],
			zeroes1                      : [0x8 ],
			volume_space_size_lsb        : [0x4 ],
			volume_space_size_msb        : [0x4 ],
			zeroes2                      : [0x20],
			volume_set_size_lsb          : [0x2 ],
			volume_set_size_msb          : [0x2 ],
			volume_sequence_number_lsb   : [0x2 ],
			volume_sequence_number_msb   : [0x2 ],
			logical_block_size_lsb       : [0x2 ],
			logical_block_size_msb       : [0x2 ],
			path_table_size_lsb          : [0x4 ],
			path_table_size_msb          : [0x4 ],
			path_table_location_lsb      : [0x4 ],
			path_table_opt_location_lsb  : [0x4 ],
			path_table_location_msb      : [0x4 ],
			path_table_opt_location_msb  : [0x4 ],
			root_dir_entry               : [0x22],
			volume_set_id                : [0x80],
			publisher_id                 : [0x80],
			data_preparer_id             : [0x80],
			application_id               : [0x80],
			copyright_file_id            : [0x26],
			abstract_file_id             : [0x24],
			bibliographic_file_id        : [0x25],
			volume_creation_date_time    : [0x11],
			volume_modification_date_time: [0x11],
			volume_expiration_date_time  : [0x11],
			volume_effective_date_time   : [0x11],
			file_structure_version       :  0x1,
			zeroes3                      :  0x1,
			data                         : [0x200],
			reserved                     : [0x28d],
		);

		Ok(Self::Primary {
			system_id:                     StrArrA::from_bytes(bytes.system_id).map_err(ParsePrimaryError::SystemId)?,
			volume_id:                     StrArrD::from_bytes(bytes.volume_id).map_err(ParsePrimaryError::VolumeId)?,
			volume_space_size:             LittleEndian::read_u32(bytes.volume_space_size_lsb),
			volume_sequence_number:        LittleEndian::read_u16(bytes.volume_sequence_number_lsb),
			logical_block_size:            LittleEndian::read_u16(bytes.logical_block_size_lsb),
			path_table_size:               LittleEndian::read_u32(bytes.path_table_size_lsb),
			path_table_location:           LittleEndian::read_u32(bytes.path_table_location_lsb),
			path_table_opt_location:       LittleEndian::read_u32(bytes.path_table_opt_location_lsb),
			root_dir_entry:                *bytes.root_dir_entry,
			volume_set_id:                 StrArrD::from_bytes(bytes.volume_set_id).map_err(ParsePrimaryError::VolumeSetId)?,
			publisher_id:                  StrArrA::from_bytes(bytes.publisher_id).map_err(ParsePrimaryError::PublisherId)?,
			data_preparer_id:              StrArrA::from_bytes(bytes.data_preparer_id).map_err(ParsePrimaryError::DataPreparerId)?,
			application_id:                StrArrA::from_bytes(bytes.application_id).map_err(ParsePrimaryError::ApplicationId)?,
			copyright_file_id:             StrArrD::from_bytes(bytes.copyright_file_id).map_err(ParsePrimaryError::CopyrightFileId)?,
			abstract_file_id:              StrArrD::from_bytes(bytes.abstract_file_id).map_err(ParsePrimaryError::AbstractFileId)?,
			bibliographic_file_id:         StrArrD::from_bytes(bytes.bibliographic_file_id).map_err(ParsePrimaryError::BibliographicFileId)?,
			volume_creation_date_time:     DecDateTime::from_bytes(bytes.volume_creation_date_time)
				.map_err(ParsePrimaryError::VolumeCreationDateTime)?,
			volume_modification_date_time: DecDateTime::from_bytes(bytes.volume_modification_date_time)
				.map_err(ParsePrimaryError::VolumeModificationDateTime)?,
			volume_expiration_date_time:   DecDateTime::from_bytes(bytes.volume_expiration_date_time)
				.map_err(ParsePrimaryError::VolumeExpirationDateTime)?,
			volume_effective_date_time:    DecDateTime::from_bytes(bytes.volume_effective_date_time)
				.map_err(ParsePrimaryError::VolumeEffectiveDateTime)?,
		})
	}
}

impl Bytes for VolumeDescriptor {
	type ByteArray = [u8; 0x800];
	type FromError = FromBytesError;
	type ToError = !;

	fn from_bytes(bytes: &Self::ByteArray) -> Result<Self, Self::FromError> {
		let bytes = array_split!(bytes,
			type_code :  0x1,
			magic     : [0x5],
			version   :  0x1,
			descriptor: [0x7f9],
		);

		// Get the type code
		let type_code = TypeCode::from_bytes(bytes.type_code).into_ok();

		// If the magic is wrong, return Err
		if bytes.magic != &Self::MAGIC {
			return Err(FromBytesError::InvalidMagic(*bytes.magic));
		}

		// If this isn't our version, return Err
		if bytes.version != &Self::VERSION {
			return Err(FromBytesError::InvalidVersion(*bytes.version));
		}

		// Check the type code and parse the descriptor itself
		match type_code {
			TypeCode::BootRecord => Self::parse_boot_record(bytes.descriptor).map_err(FromBytesError::ParseBootRecord),
			TypeCode::Primary => Self::parse_primary(bytes.descriptor).map_err(FromBytesError::ParsePrimary),
			TypeCode::Supplementary | TypeCode::VolumePartition | TypeCode::SetTerminator | TypeCode::Reserved(_) => {
				Err(FromBytesError::TypeCode(type_code))
			},
		}
	}

	fn to_bytes(&self, _bytes: &mut Self::ByteArray) -> Result<(), Self::ToError> {
		todo!()
	}
}
