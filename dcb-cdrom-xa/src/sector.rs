#![doc(include = "sector.md")]

// Modules
pub mod ecc;
pub mod edc;
pub mod error;
pub mod header;

// Exports
pub use ecc::Ecc;
pub use edc::Edc;
pub use error::{FromBytesError, NewError, ToBytesError};
pub use header::Header;

// Imports
use self::header::{subheader::SubMode, SubHeader};
use dcb_bytes::Bytes;
use dcb_util::{array_split, array_split_mut};
use header::Address;

/// A CD-ROM/XA Sector
///
/// See the module-level documentation for more details.
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Sector {
	/// Header
	pub header: Header,

	/// Data
	pub data: Data,
}

impl Sector {
	/// Creates a new sector given it's data, sector position and subheader data
	pub fn new(data: impl Into<Data>, sector_pos: usize, subheader: SubHeader) -> Result<Self, NewError> {
		let header = Header {
			address: Address::from_sector_pos(sector_pos).map_err(NewError::Address)?,
			subheader,
		};

		Ok(Self { header, data: data.into() })
	}
}


impl Bytes for Sector {
	type ByteArray = [u8; 0x930];
	type FromError = FromBytesError;
	type ToError = ToBytesError;

	fn from_bytes(byte_array: &Self::ByteArray) -> Result<Self, Self::FromError> {
		let bytes = array_split!(byte_array,
			header: [0x18 ],
			rest  : [0x918],
		);

		let header = Header::from_bytes(bytes.header).map_err(FromBytesError::Header)?;

		let data = match header.subheader.submode.contains(SubMode::FORM) {
			false => {
				let bytes = array_split!(bytes.rest,
					data  : [0x800],
					edc   : [0x4  ],
					ecc   : [0x114],
				);

				// TODO: Verify & correct ecc

				// Verify edc
				let edc = Edc::from_bytes(bytes.edc).into_ok();
				let edc_bytes = &byte_array[0x10..0x818];
				if let Err(calculated) = edc.is_valid(edc_bytes) {
					return Err(FromBytesError::WrongEdc {
						found:      edc.crc,
						calculated: calculated.crc,
					});
				}


				Data::Form1(*bytes.data)
			},

			true => {
				let bytes = array_split!(bytes.rest,
					data  : [0x914],
					edc   : [0x4  ],
				);

				// Verify edc
				let edc = Edc::from_bytes(bytes.edc).into_ok();
				let edc_bytes = &byte_array[0x10..0x92c];
				if let Err(calculated) = edc.is_valid(edc_bytes) {
					return Err(FromBytesError::WrongEdc {
						found:      edc.crc,
						calculated: calculated.crc,
					});
				}

				Data::Form2(*bytes.data)
			},
		};

		Ok(Self { header, data })
	}

	fn to_bytes(&self, bytes: &mut Self::ByteArray) -> Result<(), Self::ToError> {
		// Calculate edc before writing
		let edc = match self.data {
			Data::Form1(_) => Edc::calc_ecc(&bytes[0x10..0x818]),
			Data::Form2(_) => Edc::calc_ecc(&bytes[0x10..0x92c]),
		};


		let bytes = array_split_mut!(bytes,
			header: [0x18 ],
			rest  : [0x918],
		);

		self.header.to_bytes(bytes.header).map_err(ToBytesError::Header)?;

		match self.data {
			Data::Form1(data) => {
				let bytes = array_split_mut!(bytes.rest,
					data  : [0x800],
					edc   : [0x4  ],
					ecc   : [0x114],
				);

				// Write the data
				*bytes.data = data;

				// Write the edc
				edc.to_bytes(bytes.edc).into_ok();

				// TODO: Ecc
			},

			Data::Form2(data) => {
				let bytes = array_split_mut!(bytes.rest,
					data  : [0x914],
					edc   : [0x4  ],
				);

				// Write the data
				*bytes.data = data;

				// Write the edc
				edc.to_bytes(bytes.edc).into_ok();
			},
		}

		Ok(())
	}
}

/// Data
#[derive(PartialEq, Eq, Clone, Debug)]
#[allow(clippy::large_enum_variant)] // TODO: Check if it's worth it
pub enum Data {
	/// Form 1
	Form1([u8; 2048]),

	/// Form 2
	Form2([u8; 2324]),
}

impl Data {
	/// Returns this data as form 1
	#[must_use]
	pub const fn as_form1(&self) -> Option<&[u8; 2048]> {
		match self {
			Self::Form1(v) => Some(v),
			_ => None,
		}
	}

	/// Returns this data as form 2
	#[must_use]
	pub const fn as_form2(&self) -> Option<&[u8; 2324]> {
		match self {
			Self::Form2(v) => Some(v),
			_ => None,
		}
	}
}

impl From<[u8; 2048]> for Data {
	fn from(arr: [u8; 2048]) -> Self {
		Self::Form1(arr)
	}
}

impl From<[u8; 2324]> for Data {
	fn from(arr: [u8; 2324]) -> Self {
		Self::Form2(arr)
	}
}

impl AsRef<[u8]> for Data {
	fn as_ref(&self) -> &[u8] {
		match self {
			Data::Form1(data) => data,
			Data::Form2(data) => data,
		}
	}
}
