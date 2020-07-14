// Unit tests

// Lints
#![allow(clippy::panic)] // Unit tests are supposed to panic on error.

// Imports
use super::*;
use crate::Validatable;

#[test]
fn bytes() {
	// Valid moves with no warnings
	let valid_moves: &[(Move, <Move as Bytes>::ByteArray)] = &[
		(
			Move {
				name:    ascii::AsciiString::from_ascii("Digimon").expect("Unable to convert string to ascii"),
				power:   LittleEndian::read_u16(&[4, 1]),
				unknown: LittleEndian::read_u32(&[1, 2, 3, 4]),
			},
			[
				4, 1, 1, 2, 3, 4, b'D', b'i', b'g', b'i', b'm', b'o', b'n', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
			],
		),
		(
			Move {
				name:    ascii::AsciiString::from_ascii("123456789012345678901").expect("Unable to convert string to ascii"),
				power:   0,
				unknown: 0,
			},
			[
				0, 0, 0, 0, 0, 0, b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9',
				b'0', b'1', 0,
			],
		),
	];

	for (mov, bytes) in valid_moves {
		// Print the move and move bytes
		println!("Move: {:?}", mov);
		println!("Bytes: {:?}", bytes);

		// Check that we can create the move from bytes
		assert_eq!(&Move::from_bytes(bytes).expect("Unable to convert move from bytes"), mov);

		// Make sure the validation succeeds
		let validation = mov.validate();
		println!("Errors: {:?}", validation.errors());
		println!("Warnings: {:?}", validation.warnings());
		assert!(validation.successful());
		assert!(validation.warnings().is_empty());

		// Then serialize it to bytes and make sure it's equal
		let mut mov_bytes = <Move as Bytes>::ByteArray::default();
		Move::to_bytes(mov, &mut mov_bytes).expect("Unable to convert move to bytes");
		assert_eq!(&mov_bytes, bytes);
	}
}
