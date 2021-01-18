//! Primary volume descriptor

// Modules
pub mod error;

// Exports
pub use error::FromBytesError;

// Imports
use super::super::{date_time::DecDateTime, entry::Entry, StrArrA, StrArrD};
use byteorder::{ByteOrder, LittleEndian};
use dcb_bytes::Bytes;
use dcb_util::array_split;

/// Primary volume descriptor
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct PrimaryVolumeDescriptor {
	/// System Id
	pub system_id: StrArrA<0x20>,

	/// Volume Id
	pub volume_id: StrArrD<0x20>,

	/// Volume space size
	pub volume_space_size: u32,

	/// Volume sequence_number
	pub volume_sequence_number: u16,

	/// Logical block size
	pub logical_block_size: u16,

	/// Path table size
	pub path_table_size: u32,

	/// Path table location
	pub path_table_location: u32,

	/// Path table optional location
	pub path_table_opt_location: u32,

	/// Root directory entry
	pub root_dir_entry: Entry,

	/// Volume set identifier
	pub volume_set_id: StrArrD<0x80>,

	/// Publisher identifier
	pub publisher_id: StrArrA<0x80>,

	/// Data preparer identifier
	pub data_preparer_id: StrArrA<0x80>,

	/// Application identifier
	pub application_id: StrArrA<0x80>,

	/// Copyright file identifier
	pub copyright_file_id: StrArrD<0x26>,

	/// Abstract file identifier
	pub abstract_file_id: StrArrD<0x24>,

	/// Bibliographic file identifier
	pub bibliographic_file_id: StrArrD<0x25>,

	/// Volume creation date time
	pub volume_creation_date_time: DecDateTime,

	/// Volume modification date time
	pub volume_modification_date_time: DecDateTime,

	/// Volume expiration date time
	pub volume_expiration_date_time: DecDateTime,

	/// Volume effective date time
	pub volume_effective_date_time: DecDateTime,
}

impl Bytes for PrimaryVolumeDescriptor {
	type ByteArray = [u8; 0x7f9];
	type FromError = FromBytesError;
	type ToError = !;

	fn from_bytes(bytes: &Self::ByteArray) -> Result<Self, Self::FromError> {
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
			path_table_lsb_location      : [0x4 ],
			path_table_lsb_opt_location  : [0x4 ],
			path_table_msb_location      : [0x4 ],
			path_table_msb_opt_location  : [0x4 ],
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

		Ok(Self {
			system_id:                     StrArrA::from_bytes(bytes.system_id).map_err(FromBytesError::SystemId)?,
			volume_id:                     StrArrD::from_bytes(bytes.volume_id).map_err(FromBytesError::VolumeId)?,
			volume_space_size:             LittleEndian::read_u32(bytes.volume_space_size_lsb),
			volume_sequence_number:        LittleEndian::read_u16(bytes.volume_sequence_number_lsb),
			logical_block_size:            LittleEndian::read_u16(bytes.logical_block_size_lsb),
			path_table_size:               LittleEndian::read_u32(bytes.path_table_size_lsb),
			path_table_location:           LittleEndian::read_u32(bytes.path_table_lsb_location),
			path_table_opt_location:       LittleEndian::read_u32(bytes.path_table_lsb_opt_location),
			root_dir_entry:                Entry::from_bytes(bytes.root_dir_entry).map_err(FromBytesError::RootDirEntry)?,
			volume_set_id:                 StrArrD::from_bytes(bytes.volume_set_id).map_err(FromBytesError::VolumeSetId)?,
			publisher_id:                  StrArrA::from_bytes(bytes.publisher_id).map_err(FromBytesError::PublisherId)?,
			data_preparer_id:              StrArrA::from_bytes(bytes.data_preparer_id).map_err(FromBytesError::DataPreparerId)?,
			application_id:                StrArrA::from_bytes(bytes.application_id).map_err(FromBytesError::ApplicationId)?,
			copyright_file_id:             StrArrD::from_bytes(bytes.copyright_file_id).map_err(FromBytesError::CopyrightFileId)?,
			abstract_file_id:              StrArrD::from_bytes(bytes.abstract_file_id).map_err(FromBytesError::AbstractFileId)?,
			bibliographic_file_id:         StrArrD::from_bytes(bytes.bibliographic_file_id).map_err(FromBytesError::BibliographicFileId)?,
			volume_creation_date_time:     DecDateTime::from_bytes(bytes.volume_creation_date_time)
				.map_err(FromBytesError::VolumeCreationDateTime)?,
			volume_modification_date_time: DecDateTime::from_bytes(bytes.volume_modification_date_time)
				.map_err(FromBytesError::VolumeModificationDateTime)?,
			volume_expiration_date_time:   DecDateTime::from_bytes(bytes.volume_expiration_date_time)
				.map_err(FromBytesError::VolumeExpirationDateTime)?,
			volume_effective_date_time:    DecDateTime::from_bytes(bytes.volume_effective_date_time)
				.map_err(FromBytesError::VolumeEffectiveDateTime)?,
		})
	}

	fn to_bytes(&self, _bytes: &mut Self::ByteArray) -> Result<(), Self::ToError> {
		todo!()
	}
}
