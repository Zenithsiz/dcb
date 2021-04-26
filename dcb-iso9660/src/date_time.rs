//! Date time

// Modules
pub mod error;

// Exports
pub use error::FromBytesError;

// Imports
use super::StrArrD;
use dcb_bytes::Bytes;
use dcb_util::{array_split, array_split_mut};
use std::{cmp::Ordering, fmt};

/// Decimal date time
#[derive(PartialEq, Eq, Clone, Copy)]
pub struct DecDateTime {
	/// Year
	pub year: StrArrD<4>,

	/// Month
	pub month: StrArrD<2>,

	/// Day
	pub day: StrArrD<2>,

	/// Hour
	pub hour: StrArrD<2>,

	/// Minute
	pub minutes: StrArrD<2>,

	/// Second
	pub seconds: StrArrD<2>,

	/// Hundredths of seconds
	pub hundredths_sec: StrArrD<2>,

	/// Time zone
	pub time_zone: u8,
}

impl Bytes for DecDateTime {
	type ByteArray = [u8; 0x11];
	type FromError = FromBytesError;
	type ToError = !;

	fn from_bytes(bytes: &Self::ByteArray) -> Result<Self, Self::FromError> {
		let bytes = array_split!(bytes,
			year          : [0x4],
			month         : [0x2],
			day           : [0x2],
			hour          : [0x2],
			minutes       : [0x2],
			seconds       : [0x2],
			hundredths_sec: [0x2],
			time_zone     :  0x1 ,
		);

		// TODO: Validate time zone.
		Ok(Self {
			year:           self::parse_decimal_string(bytes.year, [b'0', b'0', b'0', b'0'], [b'9', b'9', b'9', b'9'])
				.ok_or_else(|| FromBytesError::Year(*bytes.year))?,
			month:          self::parse_decimal_string(bytes.month, [b'0', b'0'], [b'1', b'2'])
				.ok_or_else(|| FromBytesError::Month(*bytes.month))?,
			day:            self::parse_decimal_string(bytes.day, [b'0', b'0'], [b'3', b'1'])
				.ok_or_else(|| FromBytesError::Day(*bytes.day))?,
			hour:           self::parse_decimal_string(bytes.hour, [b'0', b'0'], [b'2', b'3'])
				.ok_or_else(|| FromBytesError::Hour(*bytes.hour))?,
			minutes:        self::parse_decimal_string(bytes.minutes, [b'0', b'0'], [b'5', b'9'])
				.ok_or_else(|| FromBytesError::Minute(*bytes.minutes))?,
			seconds:        self::parse_decimal_string(bytes.seconds, [b'0', b'0'], [b'5', b'9'])
				.ok_or_else(|| FromBytesError::Second(*bytes.seconds))?,
			hundredths_sec: self::parse_decimal_string(bytes.hundredths_sec, [b'0', b'0'], [b'9', b'9'])
				.ok_or_else(|| FromBytesError::HundredthsSec(*bytes.hundredths_sec))?,
			time_zone:      *bytes.time_zone,
		})
	}

	// TODO: Error checking
	fn to_bytes(&self, bytes: &mut Self::ByteArray) -> Result<(), Self::ToError> {
		let bytes = array_split_mut!(bytes,
			year          : [0x4],
			month         : [0x2],
			day           : [0x2],
			hour          : [0x2],
			minute        : [0x2],
			second        : [0x2],
			hundredths_sec: [0x2],
			time_zone     :  0x1 ,
		);


		*bytes.year = *self.year.as_bytes_arr();
		*bytes.month = *self.month.as_bytes_arr();
		*bytes.day = *self.day.as_bytes_arr();
		*bytes.hour = *self.hour.as_bytes_arr();
		*bytes.minute = *self.minutes.as_bytes_arr();
		*bytes.second = *self.seconds.as_bytes_arr();
		*bytes.hundredths_sec = *self.hundredths_sec.as_bytes_arr();
		*bytes.time_zone = self.time_zone;

		Ok(())
	}
}

impl fmt::Debug for DecDateTime {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let Self {
			year,
			month,
			day,
			hour,
			minutes,
			seconds,
			hundredths_sec,
			time_zone,
		} = self;

		let time_zone_15_min_offset: i16 = -48 + i16::from(*time_zone);

		let time_zone_hours = time_zone_15_min_offset / 4;
		#[allow(clippy::modulo_arithmetic)] // We take the absolute value before-hand
		let time_zone_minutes = (time_zone_15_min_offset.abs() % 4) * 15;

		write!(
			f,
			"\"{year}-{month}-{day}T{hour}:{minutes}:{seconds}:{hundredths_sec}{time_zone_hours:+}:\
			 {time_zone_minutes}\""
		)
	}
}

/// Ensures a decimal encoded string is valid up to a certain value
#[must_use]
#[allow(clippy::needless_range_loop)] // We want to index both strings
#[allow(clippy::indexing_slicing)] // can't panic, `n < N`
fn parse_decimal_string<const N: usize>(bytes: &[u8; N], min: [u8; N], max: [u8; N]) -> Option<StrArrD<N>> {
	// Parse it as a string first
	let string = StrArrD::from_bytes(bytes).ok()?;

	// Then make sure it's in range
	// TODO: Check that this is right
	for n in 0..N {
		match string.as_bytes()[n].cmp(&min[n]) {
			Ordering::Less => return None,
			Ordering::Equal => continue,
			Ordering::Greater => break,
		}
	}
	for n in 0..N {
		match string.as_bytes()[n].cmp(&max[n]) {
			Ordering::Less => break,
			Ordering::Equal => continue,
			Ordering::Greater => return None,
		}
	}

	Some(string)
}
