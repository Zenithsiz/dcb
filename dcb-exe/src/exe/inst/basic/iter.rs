//! Iterator over instructions

// Imports
use super::BasicInst;

/// Iterator over instructions
#[derive(PartialEq, Eq, Debug)]
pub struct InstIter<'a, I: Iterator<Item = u32> + Clone> {
	/// Underlying iterator
	iter: &'a mut I,
}

impl<'a, I: Iterator<Item = u32> + Clone> InstIter<'a, I> {
	/// Creates a new instruction iterator
	pub fn new(iter: &'a mut I) -> Self {
		Self { iter }
	}

	/// Reborrows this iterator with a smaller lifetime
	pub fn reborrow<'b>(&'b mut self) -> InstIter<'b, I>
	where
		'a: 'b,
	{
		InstIter { iter: self.iter }
	}

	/// Peeks the next element
	pub fn peeker<'b>(&mut self) -> InstPeeker<'b, I>
	where
		'a: 'b,
	{
		InstPeeker::new(self)
	}
}

/// Instruction Peeker
///
/// On drop, the peeker is applied.
#[derive(PartialEq, Eq, Debug)]
pub struct InstPeeker<'a, I: Iterator<Item = u32> + Clone> {
	/// Original iterator to update
	iter: InstIter<'a, I>,

	/// Last iterator
	last_iter: Option<I>,

	/// Current iterator
	cur_iter: I,
}

impl<'a, I: Iterator<Item = u32> + Clone> InstPeeker<'a, I> {
	/// Creates a new peeker
	pub(self) fn new(iter: InstIter<'a, I>) -> Self {
		Self {
			iter,
			last_iter: None,
			cur_iter: iter.iter.clone(),
		}
	}

	/// Reverts the last element peeked
	pub fn undo(&mut self) {
		match self.last_iter.take() {
			Some(last_iter) => self.cur_iter = last_iter,
			None => self.cur_iter = self.iter.clone(),
		}
	}

	/// Applies this peeker into the original iterator.
	pub fn apply(self) {
		// Apply changes to the original iter.
		*self.iter.iter = self.cur_iter;
	}
}

impl<'a, I: Iterator<Item = u32> + Clone> Iterator for InstPeeker<'a, I> {
	type Item = Option<BasicInst>;

	fn next(&mut self) -> Option<Self::Item> {
		// Backup our current iter
		self.last_iter = self.cur_iter.clone();

		// Then get the element from the current iterator.
		self.cur_iter.next()
	}
}
