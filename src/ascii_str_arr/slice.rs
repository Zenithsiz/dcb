//! Slicing operations for [`AsciiStrArr`]

// Imports
use super::AsciiStrArr;
use ascii::AsciiChar;
use std::ops::{Index, IndexMut};

/// Helper trait for slicing a [`AsciiStrArr`]
// Note: Adapted from `std::slice::SliceIndex`
pub trait SliceIndex<I>
where
	I: ?Sized,
{
	/// Output type for this slicing
	type Output: ?Sized;

	/// Tries to get the `idx`th element
	fn get(&self, idx: I) -> Option<&Self::Output>;

	/// Tries to get the `idx`th element mutably
	fn get_mut(&mut self, idx: I) -> Option<&mut Self::Output>;
}

impl<I, const N: usize> SliceIndex<I> for AsciiStrArr<N>
where
	I: std::slice::SliceIndex<[AsciiChar]>,
{
	type Output = <I as std::slice::SliceIndex<[AsciiChar]>>::Output;

	fn get(&self, idx: I) -> Option<&Self::Output> {
		self.as_ascii_str().as_slice().get(idx)
	}

	fn get_mut(&mut self, idx: I) -> Option<&mut Self::Output> {
		self.as_ascii_str_mut().as_mut_slice().get_mut(idx)
	}
}

impl<I, const N: usize> Index<I> for AsciiStrArr<N>
where
	Self: SliceIndex<I>,
{
	type Output = <Self as SliceIndex<I>>::Output;

	fn index(&self, idx: I) -> &Self::Output {
		self.get(idx).expect("Invalid index access")
	}
}

impl<I, const N: usize> IndexMut<I> for AsciiStrArr<N>
where
	Self: SliceIndex<I>,
{
	fn index_mut(&mut self, idx: I) -> &mut Self::Output {
		self.get_mut(idx).expect("Invalid index access")
	}
}
