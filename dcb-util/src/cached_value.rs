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
	pub fn new<Args: Hash>(args: Args, f: F) -> Self
	where
		F: FnOnce<Args, Output = T>,
	{
		// Get the hash of the input
		let input_hash = crate::hash_of(&args);

		// Then get the value
		let value = f.call_once(args);

		Self {
			value,
			input_hash,
			phantom: PhantomData,
		}
	}

	/// Tries to creates a new cached value from arguments
	pub fn try_new<Args: Hash, E>(args: Args, f: F) -> Result<Self, E>
	where
		F: FnOnce<Args, Output = Result<T, E>>,
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
	pub fn update<Args: Hash>(this: &mut Self, args: Args, f: F)
	where
		F: FnOnce<Args, Output = T>,
	{
		// If the hash of the inputs is the same, return
		let input_hash = crate::hash_of(&args);
		if input_hash == this.input_hash {
			return;
		}

		// Else update our value and our hash
		this.value = f.call_once(args);
		this.input_hash = input_hash;
	}

	/// Tries to update a cached value given it's arguments and function
	pub fn try_update<Args: Hash, E>(this: &mut Self, args: Args, f: F) -> Result<(), E>
	where
		F: FnOnce<Args, Output = Result<T, E>>,
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
	pub fn new_or_update<Args: Hash>(this: &mut Option<Self>, args: Args, f: F) -> &mut Self
	where
		F: FnOnce<Args, Output = T>,
	{
		// Note: Checking first saves a hash check on `Self::update`
		match this {
			Some(this) => {
				Self::update(this, args, f);
				this
			},
			None => this.get_or_insert_with(|| Self::new(args, f)),
		}
	}
}

impl<T, F> ops::Deref for CachedValue<T, F> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		&self.value
	}
}
