//! Standard [Executor] and [Continuation] implementations.

use std::sync::{Condvar, Mutex};

use super::{Continuation, Executor};

pub struct StandardExecutor;

impl Executor for StandardExecutor {
	type Continuation<T: Clone> = StandardContinuation<T>;

	fn init() -> Self {
		Self
	}

	fn wait<T: Clone>(&mut self, continuation: &Self::Continuation<T>) -> T {
		let StandardContinuation { mutex, condvar } = continuation;
		let mut guard = mutex.lock().unwrap();

		loop {
			if let Some(state) = &*guard {
				break state.clone()
			}

			guard = condvar.wait(guard).unwrap();
		}
	}

	fn execute<T: Clone, P>(
		self,
		tester: impl FnOnce() -> (T, P),
		continuation: &Self::Continuation<T>,
	) -> P {
		let StandardContinuation { mutex, condvar } = continuation;
		let mut guard = mutex.lock().unwrap();

		let (r, passed) = tester();

		*guard = Some(r);
		condvar.notify_all();

		passed
	}
}

pub struct StandardContinuation<T> {
	mutex: Mutex<Option<T>>,
	condvar: Condvar,
}

impl<T> Continuation for StandardContinuation<T> {
	const INITIAL: Self = Self {
		mutex: Mutex::new(None),
		condvar: Condvar::new(),
	};
}
