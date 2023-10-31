use std::panic::{catch_unwind, resume_unwind, AssertUnwindSafe};

use crate::Executor;

#[derive(Clone, Copy)]
pub struct TestFailedError;

pub fn exec_noresult<T: Clone, Ex: Executor>(
	executor: Ex,
	continuation: &Ex::Continuation<Result<T, TestFailedError>>,
	f: impl FnOnce() -> T,
) {
	let r = executor.execute(
		|| match catch_unwind(AssertUnwindSafe(f)) {
			Ok(v) => (Ok(v), Ok(())),
			Err(p) => (Err(TestFailedError), Err(p)),
		},
		continuation,
	);

	match r {
		Ok(()) => {},
		Err(p) => resume_unwind(p),
	}
}

pub fn exec_result<T: Clone, E, Ex: Executor>(
	executor: Ex,
	continuation: &Ex::Continuation<Result<T, TestFailedError>>,
	f: impl FnOnce() -> Result<T, E>,
) -> Result<(), E> {
	let r = executor.execute(
		|| match catch_unwind(AssertUnwindSafe(f)) {
			Ok(Ok(v)) => (Ok(v), Ok(Ok(()))),
			Ok(Err(e)) => (Err(TestFailedError), Ok(Err(e))),
			Err(p) => (Err(TestFailedError), Err(p)),
		},
		continuation,
	);

	match r {
		Ok(r) => r,
		Err(p) => resume_unwind(p),
	}
}
