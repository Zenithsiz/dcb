// Unit tests

// Lints
#![allow(clippy::panic)] // Unit tests are supposed to panic on error.

// Imports
use super::*;

#[test]
fn bytes() {
	// Valid moves with no warnings
	#[rustfmt::skip]
	let valid_moves: &[(Move, <Move as Bytes>::ByteArray)] = &[(
		Move {
			name:    ascii::AsciiString::from_ascii("Digimon").expect("Unable to convert string to ascii"),
			power:   LittleEndian::read_u16(&[1, 2]),
			unknown: LittleEndian::read_u32(&[1, 2, 3, 4]),
		},
		[
			// Power
			1, 2,
			
			// Unknown,
			1, 2, 3, 4,
			
			// Name
			b'D', b'i', b'g', b'i', b'm', b'o', b'n', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0',
			b'\0', b'\0',
		],
	)];

	for (mov, move_bytes) in valid_moves {
		// Check that we can create the move from bytes
		assert_eq!(&Move::from_bytes(move_bytes).expect("Unable to convert move from bytes"), mov);

		// Make sure the validation succeeds
		let validation = mov.validate();
		assert!(validation.successful());
		assert!(validation.warnings().is_empty());

		// Then serialize it to bytes and make sure it's equal
		let mut bytes = <Move as Bytes>::ByteArray::default();
		Move::to_bytes(mov, &mut bytes).expect("Unable to convert move to bytes");
		assert_eq!(&bytes, move_bytes);
	}
}
