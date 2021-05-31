//! Tasks

// Imports
use std::{
	future::Future,
	pin::Pin,
	sync::{Arc, Mutex},
	task::Poll,
};

/// Spawns a task and returns a future for awaiting it's value
pub fn spawn<T: Send + 'static>(f: impl FnOnce() -> T + Send + 'static) -> ValueFuture<T> {
	// Create the value mutex
	let mutex = Arc::new(Mutex::new(None));
	let future = ValueFuture {
		value:     Arc::clone(&mutex),
		exhausted: false,
	};

	// Spawn the task
	rayon::spawn(move || {
		let value = f();
		*mutex.lock().expect("Poisoned") = Some(value);
	});

	// And return the future
	future
}

/// Value future
pub struct ValueFuture<T> {
	/// Underlying value
	value: Arc<Mutex<Option<T>>>,

	/// If the value was already retrieved
	exhausted: bool,
}

impl<T> ValueFuture<T> {
	/// Returns the value if finished
	pub fn get(&mut self) -> Option<T> {
		let mut cx = std::task::Context::from_waker(futures::task::noop_waker_ref());
		match Pin::new(self).poll(&mut cx) {
			Poll::Ready(value) => Some(value),
			Poll::Pending => None,
		}
	}
}

impl<T> Future for ValueFuture<T> {
	type Output = T;

	fn poll(mut self: Pin<&mut Self>, _cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
		let Self { value, exhausted } = &mut *self;

		// If we already retrieved the value, panic
		assert!(!*exhausted, "Cannot call `poll` on an exhausted future");

		// Else if we're done, return it
		match value.lock().expect("Poisoned").take() {
			Some(value) => {
				*exhausted = true;
				Poll::Ready(value)
			},
			None => Poll::Pending,
		}
	}
}
