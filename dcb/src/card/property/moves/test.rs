// Unit tests

// Imports
use super::*;
use crate::Validatable;
use std::convert::TryFrom;

#[test]
fn valid_bytes() {
	// Valid moves with no warnings
	#[rustfmt::skip]
	#[allow(clippy::as_conversions)]
	let valid_moves: &[(Move, <Move as Bytes>::ByteArray)] = &[
		(
			Move {
				name:    AsciiStrArr::try_from(b"Digimon" as &[u8]).expect("Unable to convert string to ascii"),
				power:   LittleEndian::read_u16(&[4, 1]),
				unknown: LittleEndian::read_u32(&[1, 2, 3, 4]),
			},
			[
				4, 1,                                        // Power
				1, 2, 3, 4,                                  // Unknown
				b'D', b'i', b'g', b'i', b'm', b'o', b'n', 0, // Name
				0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
			],
		),
		(
			Move {
				name:    AsciiStrArr::try_from(b"123456789012345678901" as &[u8]).expect("Unable to convert string to ascii"),
				power:   0,
				unknown: 0,
			},
			[
				0, 0,                                                       // Power
				0, 0, 0, 0,                                                 // Unknown
				b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'0', // Name
				b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'0', // ^^^^
				b'1', 0,                                                    // ^^^^
			],
		),
	];

	for (mov, bytes) in valid_moves {
		// Check that we can create the move from bytes
		assert_eq!(&Move::from_bytes(bytes).expect("Unable to convert move from bytes"), mov);

		// Make sure the validation succeeds
		let validation = mov.validate();
		assert!(validation.successful());
		assert!(validation.warnings().is_empty());

		// Then serialize it to bytes and make sure it's equal
		let mut mov_bytes = <Move as Bytes>::ByteArray::default();
		Move::to_bytes(mov, &mut mov_bytes).expect("Unable to convert move to bytes");
		assert_eq!(&mov_bytes, bytes);
	}
}

#[test]
fn invalid_bytes() {
	// Valid moves with no warnings
	#[rustfmt::skip]
	let invalid_moves: &[(<Move as Bytes>::ByteArray, FromBytesError)] = &[
		(
			[
				0, 0,                                                       // Power
				0, 0, 0, 0,                                                 // Unknown
				b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'0', // Name
				b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'0', // ^^^^
				b'1', 1,                                                    // ^^^^
			],
			FromBytesError::Name(null_ascii_string::ReadError::NoNull),
		),
	];

	for (bytes, err) in invalid_moves {
		// Check that we can create the move from bytes
		assert_eq!(Move::from_bytes(bytes), Err(*err));
	}
}
