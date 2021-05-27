//! Type families

/// Result family
#[sealed::sealed]
pub trait ResultFamily: Into<Result<Self::Ok, Self::Err>> {
	/// Ok type
	type Ok;

	/// Error type
	type Err;
}

#[sealed::sealed]
impl<T, E> ResultFamily for Result<T, E> {
	type Err = E;
	type Ok = T;
}
