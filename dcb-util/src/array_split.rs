//! Array splitters

// Export `arrayref` to use in macros
#[doc(hidden)]
pub use ::arrayref;

/// Splits an array into various members
#[macro_export]
macro_rules! array_split {
	(
		$arr:expr,
		$(
			$name:ident :

			$( [$arr_size:expr]    )?
			$(  $val_size:literal  )?

		),* $(,)?
	) => {{
		// Struct holding all fields
		struct Fields<'a, T> {
			$(
				$name:

				$( &'a [T; $arr_size], )?
				$( &'a T, #[cfg(invalid)] __field: [u8; $val_size], )?
			)*
			__phantom: ::std::marker::PhantomData<&'a T>,
		}

		// Get everything from `array_refs`
		#[allow(
			clippy::used_underscore_binding,
			clippy::ptr_offset_with_cast,
			clippy::indexing_slicing,
		)]
		let (
			$(
				$name
			),*
		) = $crate::array_split::arrayref::array_refs!(
			$arr,
			$(
				$( $arr_size )?
				$( $val_size )?
			),*
		);

		// And return the fields
		Fields {
			$(
				$name
				$( : &( $name[$val_size - $val_size] ) )?
				,
			)*
			__phantom: ::std::marker::PhantomData,
		}
	}}
}

/// Splits an array into various members mutably
#[allow(clippy::module_name_repetitions)] // `_mut` version should be in the same module
#[macro_export]
macro_rules! array_split_mut {
	(
		$arr:expr,
		$(
			$name:ident :

			$( [$arr_size:expr]    )?
			$(  $val_size:literal  )?

		),* $(,)?
	) => {{
		// Struct holding all fields
		struct Fields<'a, T> {
			$(
				$name:

				$( &'a mut [T; $arr_size], )?
				// Note: This `cfg` is simply done so that `__field` never appears.
				//       The `__field` serves to identify when this part should be written.
				$( &'a mut T, #[cfg(invalid)] __field: [u8; $val_size], )?
			)*
			__phantom: ::std::marker::PhantomData<&'a mut T>,
		}

		// Get everything from `mut_array_refs`
		#[allow(
			clippy::used_underscore_binding,
			clippy::ptr_offset_with_cast,
			clippy::indexing_slicing,
		)]
		let (
			$(
				$name
			),*
		) = $crate::array_split::arrayref::mut_array_refs!(
			$arr,
			$(
				$( $arr_size )?
				$( $val_size )?
			),*
		);

		// And return the fields
		Fields {
			$(
				$name
				// Note: This serves to turn a `&mut [u8; 1]` into a `&mut u8`.
				$( : &mut ( $name[$val_size - $val_size] ) )?
				,
			)*
			__phantom: ::std::marker::PhantomData,
		}
	}}
}
