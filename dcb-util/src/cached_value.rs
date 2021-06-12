//! Cached value

// Imports
use std::{hash::Hash, marker::PhantomData, ops};


/// A cached, update-able value
// TODO: Switch to only requiring `<T, Args>` but allow for `for<'a> (&'a u32)` and stuff.
pub struct CachedValue<T, F> {
	/// Value
	value: T,

	/// Hash of inputs
	input_hash: u64,

	/// Phantom data
	phantom: PhantomData<F>,
}

impl<T, F> CachedValue<T, F> {
	/// Creates a new cached value from arguments
	pub fn new<Args: Hash, F2>(args: Args, f: F2) -> Self
	where
		F: Fn<Args>,
		F2: FnOnce<Args, Output = T>,
	{
		Self::try_new(args, FnResultWrapper(f)).into_ok()
	}

	/// Tries to creates a new cached value from arguments
	pub fn try_new<Args: Hash, E, F2>(args: Args, f: F2) -> Result<Self, E>
	where
		F: Fn<Args>,
		F2: FnOnce<Args, Output = Result<T, E>>,
	{
		// Get the hash of the input
		let input_hash = crate::hash_of(&args);

		// Then try to get the value
		let value = f.call_once(args)?;

		Ok(Self {
			value,
			input_hash,
			phantom: PhantomData,
		})
	}

	/// Updates a cached value given it's arguments and function
	pub fn update<Args: Hash, F2>(this: &mut Self, args: Args, f: F2)
	where
		F: Fn<Args>,
		F2: FnOnce<Args, Output = T>,
	{
		Self::try_update(this, args, FnResultWrapper(f)).into_ok();
	}

	/// Tries to update a cached value given it's arguments and function
	pub fn try_update<Args: Hash, E, F2>(this: &mut Self, args: Args, f: F2) -> Result<(), E>
	where
		F: Fn<Args>,
		F2: FnOnce<Args, Output = Result<T, E>>,
	{
		// If the hash of the inputs is the same, return
		let input_hash = crate::hash_of(&args);
		if input_hash == this.input_hash {
			return Ok(());
		}

		// Else update our value and our hash
		// Note: Only update the hash if we successfully got the value
		this.value = f.call_once(args)?;
		this.input_hash = input_hash;

		Ok(())
	}

	/// Creates or updates a cached value
	pub fn new_or_update<Args: Hash, F2>(this: &mut Option<Self>, args: Args, f: F2) -> &mut Self
	where
		F: Fn<Args>,
		F2: FnOnce<Args, Output = T>,
	{
		// Note: Checking first saves a hash check on `Self::update`
		match this {
			Some(this) => {
				Self::update(this, args, f);
				this
			},
			None => this.insert(Self::new(args, f)),
		}
	}
}

impl<T, F> ops::Deref for CachedValue<T, F> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		&self.value
	}
}


/// Wraps a function that returns `T` to make it return `Result<T, !>`
struct FnResultWrapper<F>(F);

impl<F: FnOnce<Args>, Args> FnOnce<Args> for FnResultWrapper<F> {
	type Output = Result<F::Output, !>;

	extern "rust-call" fn call_once(self, args: Args) -> Self::Output {
		Ok(self.0.call_once(args))
	}
}
