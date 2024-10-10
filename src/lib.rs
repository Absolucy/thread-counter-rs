// SPDX-License-Identifier: MIT OR Apache-2.0
//! A lightweight, thread-safe library for counting and synchronizing concurrent
//! operations.
//!
//! This crate provides a [`ThreadCounter`] type that can be used to keep track
//! of the number of active threads or operations, and to synchronize the
//! completion of these operations. It's particularly useful for scenarios where
//! you need to wait for a group of tasks to complete before proceeding.
//!
//! ## Features
//!
//! - Thread-safe counting of active operations.
//! - RAII-based automatic decrementing using [`Ticket`]s.
//! - Ability to wait for all operations to complete, with optional timeout.
//!
//! ## Usage
//!
//! Here's a basic example of how to use the [`ThreadCounter`]:
//!
//! ```rust
//! use std::{thread, time::Duration};
//! use thread_counter::ThreadCounter;
//!
//! let counter = ThreadCounter::default();
//!
//! // Spawn some threads
//! for _ in 0..5 {
//! 	thread::spawn(move || {
//! 		// Take a ticket, incrementing the counter.
//! 		let ticket = counter.ticket();
//! 		// Simulate some work
//! 		thread::sleep(Duration::from_millis(100));
//! 		// `ticket` is automatically dropped here, decrementing the counter
//! 	});
//! }
//!
//! // Wait for all threads to complete, timing out after 200ms.
//! counter.wait(Duration::from_millis(200));
//! println!("All threads have completed!");
//! ```

#![forbid(unsafe_code)]
#![warn(
	clippy::correctness,
	clippy::suspicious,
	clippy::complexity,
	clippy::perf,
	clippy::style
)]
#![allow(clippy::tabs_in_doc_comments)]

use parking_lot::{Condvar, Mutex};
use std::{ops::Deref, sync::Arc, time::Duration};

/// A thread-safe counter for tracking the number of active threads or
/// operations.
///
/// This struct provides a high-level interface for incrementing, decrementing,
/// and waiting on a thread counter. It internally uses [`Arc`] to allow for
/// cheap cloning and shared ownership.
#[derive(Default, Clone)]
pub struct ThreadCounter {
	inner: Arc<RawThreadCounter>,
}

impl ThreadCounter {
	/// Creates a new [`Ticket`] from this thread counter.
	///
	/// This method increments the thread count and returns a [`Ticket`] that
	/// will automatically decrement the count when dropped.
	///
	/// # Returns
	/// A new [`Ticket`] instance associated with this counter.
	pub fn ticket(&self) -> Ticket {
		self.increment();
		Ticket {
			counter: self.clone(),
		}
	}
}

impl Deref for ThreadCounter {
	type Target = RawThreadCounter;

	fn deref(&self) -> &Self::Target {
		&self.inner
	}
}

impl AsRef<RawThreadCounter> for ThreadCounter {
	fn as_ref(&self) -> &RawThreadCounter {
		&self.inner
	}
}

/// The internal implementation of the thread counter.
///
/// This struct handles the actual counting and synchronization mechanisms.
pub struct RawThreadCounter {
	count: Mutex<usize>,
	condvar: Condvar,
}

impl RawThreadCounter {
	/// Increments the thread counter.
	///
	/// # Note
	/// It's preferable to use [`ThreadCounter::ticket()`] instead, which
	/// ensures that the count is automatically decremented when the ticket is
	/// dropped.
	pub fn increment(&self) {
		let mut count = self.count.lock();
		*count += 1;
	}

	/// Decrements the thread counter.
	///
	/// If the count reaches zero, it notifies all waiting threads.
	///
	/// # Note
	/// It's preferable to use [`ThreadCounter::ticket()`] instead, which
	/// ensures that the count is automatically decremented when the ticket is
	/// dropped.
	pub fn decrement(&self) {
		let mut count = self.count.lock();
		*count -= 1;
		if *count == 0 {
			self.condvar.notify_all();
		}
	}

	/// Waits for the counter to reach zero, with an optional timeout.
	///
	/// # Arguments
	/// * `timeout` - An optional duration to wait. If `None`, waits
	///   indefinitely.
	///
	/// # Returns
	/// * `true` if the count reached zero.
	/// * `false` if the timeout was reached before the count reached zero.
	pub fn wait(&self, timeout: impl Into<Option<Duration>>) -> bool {
		let mut count = self.count.lock();
		let condition = |count: &mut usize| *count > 0;
		match timeout.into() {
			Some(timeout) => !self
				.condvar
				.wait_while_for(&mut count, condition, timeout)
				.timed_out(),
			None => {
				self.condvar.wait_while(&mut count, condition);
				true
			}
		}
	}
}

impl Default for RawThreadCounter {
	fn default() -> Self {
		Self {
			count: Mutex::new(0),
			condvar: Condvar::new(),
		}
	}
}

/// A RAII guard for automatically managing the thread count.
///
/// When a `Ticket` is created, it increments the associated thread counter.
/// When the `Ticket` is dropped, it automatically decrements the counter.
pub struct Ticket {
	counter: ThreadCounter,
}

impl Drop for Ticket {
	fn drop(&mut self) {
		self.counter.decrement();
	}
}
