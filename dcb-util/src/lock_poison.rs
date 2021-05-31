//! Locking with poison

// Imports
use std::sync::{Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};

/// Helper trait for locking `Mutex`s without handling poisoning
pub trait MutexPoison {
	/// Guard type
	type Guard;

	/// Locks this mutex, panicking if poisoned
	fn lock_unwrap(self) -> Self::Guard;
}

impl<'a, T> MutexPoison for &'a Mutex<T> {
	type Guard = MutexGuard<'a, T>;

	#[track_caller]
	fn lock_unwrap(self) -> Self::Guard {
		Mutex::lock(self).expect("Poisoned")
	}
}

/// Helper trait for locking `RwLock`s without handling poisoning
pub trait RwLockPoison {
	/// Read guard type
	type ReadGuard;

	/// Write guard type
	type WriteGuard;

	/// Locks this rwlock for reading, panicking if poisoned
	fn read_unwrap(self) -> Self::ReadGuard;

	/// Locks this rwlock for writing, panicking if poisoned
	fn write_unwrap(self) -> Self::WriteGuard;
}

impl<'a, T> RwLockPoison for &'a RwLock<T> {
	type ReadGuard = RwLockReadGuard<'a, T>;
	type WriteGuard = RwLockWriteGuard<'a, T>;

	#[track_caller]
	fn read_unwrap(self) -> Self::ReadGuard {
		RwLock::read(self).expect("Poisoned")
	}

	#[track_caller]
	fn write_unwrap(self) -> Self::WriteGuard {
		RwLock::write(self).expect("Poisoned")
	}
}
