//! Peekable iterators

/// Iterators which are peekable
pub trait PeekableIter: Iterator {
	/// Peeks the next element
	fn peek(&self) -> Option<Self::Item>;

	/// Consumes the next element if `f` returns true
	fn next_if(&mut self, f: impl FnOnce(Self::Item) -> bool) -> bool;

	/// Consumes the next element if `f` returns `Some`
	fn try_next<T: std::ops::Try>(&mut self, f: impl FnOnce(Self::Item) -> T) -> Option<Result<T::Ok, T::Error>>;
}

impl<I: Iterator + Clone> PeekableIter for I {
	fn peek(&self) -> Option<Self::Item> {
		self.clone().next()
	}

	fn next_if(&mut self, f: impl FnOnce(Self::Item) -> bool) -> bool {
		matches!(self.try_next(move |value| f(value).then_some(())), Some(Ok(())))
	}

	fn try_next<T: std::ops::Try>(&mut self, f: impl FnOnce(Self::Item) -> T) -> Option<Result<T::Ok, T::Error>> {
		let mut iter = self.clone();
		match iter.next().map(f)?.into_result() {
			Ok(value) => {
				*self = iter;
				Some(Ok(value))
			},
			Err(err) => Some(Err(err)),
		}
	}
}
