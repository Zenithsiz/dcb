//! Slicing operations for [`AsciiStrArr`]

// Imports
use super::AsciiStrArr;
use ascii::AsciiChar;
use std::ops::{Index, IndexMut};

/// Helper trait for slicing a [`AsciiStrArr`]
// Note: Adapted from `std::slice::SliceIndex`
pub trait SliceIndex {
	/// Output type for this slicing
	type Output: ?Sized;

	/// Slices the string, if in bounds
	fn get<const N: usize>(self, ascii_str: &AsciiStrArr<N>) -> Option<&Self::Output>;

	/// Slices the string mutably, if in bounds
	fn get_mut<const N: usize>(self, ascii_str: &mut AsciiStrArr<N>) -> Option<&mut Self::Output>;

	/// Slices the string without checking bounds
	///
	/// # Safety
	/// Calling this method with an out-of-bounds index is undefined-behavior even if the resulting
	/// reference is not used
	unsafe fn get_unchecked<const N: usize>(self, ascii_str: &AsciiStrArr<N>) -> &Self::Output;

	/// Slices the string mutably without checking bounds
	///
	/// # Safety
	/// Calling this method with an out-of-bounds index is undefined-behavior even if the resulting
	/// reference is not used
	unsafe fn get_unchecked_mut<const N: usize>(self, ascii_str: &mut AsciiStrArr<N>) -> &mut Self::Output;
}

impl<I> SliceIndex for I
where
	I: std::slice::SliceIndex<[AsciiChar]>,
{
	type Output = <Self as std::slice::SliceIndex<[AsciiChar]>>::Output;

	fn get<const N: usize>(self, ascii_str: &AsciiStrArr<N>) -> Option<&Self::Output> {
		ascii_str.as_ascii().as_slice().get(self)
	}

	fn get_mut<const N: usize>(self, ascii_str: &mut AsciiStrArr<N>) -> Option<&mut Self::Output> {
		ascii_str.as_ascii_mut().as_mut_slice().get_mut(self)
	}

	unsafe fn get_unchecked<const N: usize>(self, ascii_str: &AsciiStrArr<N>) -> &Self::Output {
		ascii_str.as_ascii().as_slice().get_unchecked(self)
	}

	unsafe fn get_unchecked_mut<const N: usize>(self, ascii_str: &mut AsciiStrArr<N>) -> &mut Self::Output {
		ascii_str.as_ascii_mut().as_mut_slice().get_unchecked_mut(self)
	}
}

impl<I, const N: usize> Index<I> for AsciiStrArr<N>
where
	I: SliceIndex,
{
	type Output = <I as SliceIndex>::Output;

	fn index(&self, idx: I) -> &Self::Output {
		self.get(idx).expect("Invalid index access")
	}
}

impl<I, const N: usize> IndexMut<I> for AsciiStrArr<N>
where
	I: SliceIndex,
{
	fn index_mut(&mut self, idx: I) -> &mut Self::Output {
		self.get_mut(idx).expect("Invalid index access")
	}
}
