![Crates.io Version](https://img.shields.io/crates/v/thread-counter)
![Crates.io License](https://img.shields.io/crates/l/thread-counter)
![Crates.io Downloads (recent)](https://img.shields.io/crates/dr/thread-counter?label=recent%20downloads)
![Crates.io Size](https://img.shields.io/crates/size/thread-counter?label=size)
![Maintenance](https://img.shields.io/badge/maintenance-actively--developed-brightgreen.svg)

# thread-counter

A lightweight, thread-safe library for counting and synchronizing concurrent
operations.

This crate provides a `ThreadCounter` type that can be used to keep track
of the number of active threads or operations, and to synchronize the
completion of these operations. It's particularly useful for scenarios where
you need to wait for a group of tasks to complete before proceeding.

### Features

- Thread-safe counting of active operations.
- RAII-based automatic decrementing using `Ticket`s.
- Ability to wait for all operations to complete, with optional timeout.

### Usage

Here's a basic example of how to use the `ThreadCounter`:

```rust
use std::{thread, time::Duration};
use thread_counter::ThreadCounter;

let counter = ThreadCounter::default();

// Spawn some threads
for _ in 0..5 {
	thread::spawn(move || {
		// Take a ticket, incrementing the counter.
		let ticket = counter.ticket();
		// Simulate some work
		thread::sleep(Duration::from_millis(100));
		// `ticket` is automatically dropped here, decrementing the counter
	});
}

// Wait for all threads to complete, timing out after 200ms.
counter.wait(Duration::from_millis(200));
println!("All threads have completed!");
```

Current version: 0.1.0

Some additional info here

## License

`thread-counter` is dual-licensed under the [MIT license](LICENSE-MIT) and the [Apache License (Version 2.0)](LICENSE-APACHE).
