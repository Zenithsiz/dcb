//! Abstraction over the game file.
//! 
//! See [`GameFile`] for details

// Addresses
use crate::io::address::{Real as RealAddress, Data as DataAddress, RealToDataError};

// Read / Write
use std::io::{Read, Write, Seek};

/// A type that abstracts over a the game reader.
/// 
/// # Game reader
/// The game file is a `.bin` file, of the type `MODE2/2352`.
/// 
/// This means that the file is divided into sectors of size
/// 2352 bytes, each with it's data structure.
/// 
/// For us the only thing that matters is the data section
/// of each sector, which is 2048 bytes long.
/// 
/// This type allows reading and writing in `DataAddress` addresses,
/// which are reader offsets in terms of the 2048 byte data section,
/// instead of the 2352 byte sectors.
/// 
/// # Parameters
/// `GameFile` is generic over `R`, this being any type that implements
/// `Read`, `Write` and `Seek`, thus being able to read from either a
/// reader, a buffer in memory or even some remote network location.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Default, Hash, Debug)]
pub struct GameFile<R: Read + Write + Seek> {
	/// The type to read and write from
	reader: R,
}

/// Error type for [`GameFile::from_reader`]
#[derive(Debug)]
#[derive(derive_more::Display)]
pub enum NewGameFileError {
	/// Unable to seek reader to data section
	#[display(fmt = "Unable to seek reader to data section")]
	SeekData( std::io::Error ),
}

impl std::error::Error for NewGameFileError {
	fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
		match self {
			Self::SeekData(err) => Some(err),
		}
	}
}

// Constructors
impl<R: Read + Write + Seek> GameFile<R> {
	/// Constructs a `GameFile` given a reader
	pub fn from_reader(mut reader: R) -> Result<Self, NewGameFileError> {
		// Seek the reader to the beginning of the data section
		reader.seek( std::io::SeekFrom::Start(
			RealAddress::DATA_START
		)).map_err(NewGameFileError::SeekData)?;
		
		Ok( Self { reader } )
	}
}

/// `Read` for `GameFile`
/// 
/// # Implementation guarantees
/// Currently `Read` guarantees that if an error is returned, then
/// the buffer isn't modified, but this implementation cannot make
/// that guarantee.
impl<R: Read + Write + Seek> Read for GameFile<R>
{
	fn read(&mut self, mut buf: &mut [u8]) -> std::io::Result<usize>
	{
		// Total length of the buffer to fill
		let total_buf_len = buf.len();
		
		// While the buffer isn't empty
		while !buf.is_empty()
		{
			// Get the current real address we're at in the reader
			// Note: If we can't get the position, we return immediatly
			let cur_real_address = RealAddress::from( self.reader.stream_position()? );
			
			// Get the data section end
			let data_section_end = cur_real_address.data_section_end();
			
			// If we're at the end of the data section, seek to the next data section
			if cur_real_address == data_section_end {
				// Seek ahead by skiping the footer and next header
				self.reader.seek( std::io::SeekFrom::Current(
					(RealAddress::FOOTER_BYTE_SIZE +
					 RealAddress::HEADER_BYTE_SIZE) as i64
				))?;
				
				// And restart this loop
				continue;
			}
			
			// We always guarantee that the current address lies within the data sections
			// Note: We only check it here, because `cur_real_address` may be `data_section_end`
			//       during seeking.
			assert!( cur_real_address.in_data_section(), "Real offset {} [Sector {}, Offset {}] could not be read as it was not in the data section",
				cur_real_address,
				cur_real_address.sector(),
				cur_real_address.offset()
			);
			
			// Check how many bytes we can read
			let mut bytes_to_read = (data_section_end - cur_real_address) as usize;
			
			// If we were to read more bytes than the buffer has, read less
			if bytes_to_read > buf.len() {
				bytes_to_read = buf.len();
			}
			
			// Read either until the end of the data section or until buffer is full
			// Note: If any fail, we immediatly return Err
			let bytes_read = self.reader.read( &mut buf[0..bytes_to_read] )?;
			
			// If 0 bytes were read, EOF was reached, so return with however many we've read
			if bytes_read == 0 {
				return Ok( total_buf_len - buf.len() );
			}
			
			// Discard what we've already read
			buf = &mut buf[bytes_read..]; // If `bytes_to_read == buf.len()` this does not panic
		}
		
		// And return the bytes we read
		Ok( total_buf_len )
	}
}

/// Write for `GameFile`
/// 
/// # Implementation guarantees
/// Currently `Read` guarantees that if an error is returned, then
/// the buffer isn't modified, but this implementation cannot make
/// that guarantee.
impl<R: Read + Write + Seek> Write for GameFile<R>
{
	fn write(&mut self, mut buf: &[u8]) -> std::io::Result<usize>
	{
		// Total length of the buffer to write
		let total_buf_len = buf.len();
		
		// While the buffer isn't empty
		while !buf.is_empty()
		{
			// Get the current real address we're at in the reader
			// Note: If we can't get the position, we return immediatly
			let cur_real_address = RealAddress::from( self.reader.stream_position()? );
			
			// Get the data section end
			let data_section_end = cur_real_address.data_section_end();
			
			// If we're at the end of the data section, seek to the next data section
			if cur_real_address == data_section_end {
				// Seek ahead by skiping the footer and next header
				self.reader.seek( std::io::SeekFrom::Current(
					(RealAddress::FOOTER_BYTE_SIZE +
					 RealAddress::HEADER_BYTE_SIZE) as i64
				))?;
				
				// And restart this loop
				continue;
			}
			
			// We always guarantee that the current address lies within the data sections
			// Note: We only check it here, because `cur_real_address` may be `data_section_end`
			//       during seeking.
			assert!( cur_real_address.in_data_section(), "Real offset {} [Sector {}, Offset {}] could not be written as it was not in the data section",
				cur_real_address,
				cur_real_address.sector(),
				cur_real_address.offset()
			);
			
			// Check how many bytes we can write
			let mut bytes_to_write = (data_section_end - cur_real_address) as usize;
			
			// If we were to write more bytes than the buffer has, write less
			if bytes_to_write > buf.len() {
				bytes_to_write = buf.len();
			}
			
			// Write either until the end of the data section or until buffer runs out
			// Note: If this fails, we immediatly return Err
			let bytes_written = self.reader.write( &buf[0..bytes_to_write] )?;
			
			// If 0 bytes were written, EOF was reached, so return with however many we've read
			if bytes_written == 0 {
				return Ok( total_buf_len - buf.len() );
			}
			
			// Discard what we've already written
			buf = &buf[bytes_to_write..]; // If `bytes_to_write == buf.len()` this does not panic
		}
		
		// And return the bytes we read
		Ok( total_buf_len )
	}
	
	fn flush(&mut self) -> std::io::Result<()> {
		self.reader.flush()
	}
}

/// Error type for `Seek for GameFile`.
/// Returned when, after seeking, we ended up in a non-data section
#[derive(Debug)]
#[derive(derive_more::Display)]
#[display(fmt = "Reader seeked into a non-data section")]
pub struct SeekNonDataError(RealToDataError);

impl std::error::Error for SeekNonDataError {
	fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
		Some(&self.0)
	}
}

impl<R: Read + Write + Seek> Seek for GameFile<R>
{
	fn seek(&mut self, data_pos: std::io::SeekFrom) -> std::io::Result<u64> {
		use std::{
			io::SeekFrom,
			convert::TryFrom,
		};
		
		// Calculate the real position
		let real_pos = match data_pos {
			SeekFrom::Start(data_address) => SeekFrom::Start( u64::from( RealAddress::from( DataAddress::from(data_address) ) ) ),
			SeekFrom::Current(data_offset) => SeekFrom::Start(
				u64::from(RealAddress::from(
					DataAddress::try_from( RealAddress::from( self.reader.stream_position()? ) )
						.map_err(SeekNonDataError)
						.map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err))? +
					data_offset
				))
			),
			SeekFrom::End(_) => {
				todo!("SeekFrom::End isn't currently implemented");
			}
		};
		
		// Seek to the real position and get where we are right now
		let cur_real_address = self.reader.seek(real_pos)?;
		
		// Get the data address
		let data_address = DataAddress::try_from( RealAddress::from(cur_real_address) )
			.map_err(SeekNonDataError)
			.map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err))?;
		
		// And return the new data address
		Ok( u64::from(data_address) )
	}
}
