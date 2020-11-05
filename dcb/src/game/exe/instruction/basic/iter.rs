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
	pub fn new(iter: &mut I) -> Self {
		Self { iter }
	}

	/// Reborrows this iterator with a smaller lifetime
	pub fn reborrow<'b>(&mut self) -> InstIter<'b, I>
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
		InstPeeker {
			cur_iter: self.iter.clone(),
			iter:     self,
		}
	}
}

/// Instruction Peeker
#[derive(PartialEq, Eq, Debug)]
pub struct InstPeeker<'a, I: Iterator<Item = u32> + Clone> {
	/// Original iterator to update if consumed
	iter: InstIter<'a, I>,

	/// Current iterator
	cur_iter: I,
}

impl<'a, I: Iterator<Item = u32> + Clone> InstPeeker<'a, I> {
	/// Updates the iterator this is peeking to consume all
	pub fn update(&mut self) {
		*self.iter.iter = self.cur_iter;
	}
}

impl<'a, I: Iterator<Item = u32> + Clone> Iterator for InstPeeker<'a, I> {
	type Item = Option<BasicInst>;

	fn next(&mut self) -> Option<Self::Item> {
		// Consume our current iterator
		self.cur_iter.next()
	}
}
