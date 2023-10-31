use std::{
	error::Error,
	fmt::{self, Display, Formatter},
};

use flowtest::{
	standard_executor::{StandardContinuation, StandardExecutor},
	Continuation,
	Executor,
	__private::{self, TestFailedError},
};

type TestContinuation<T> = StandardContinuation<Result<T, TestFailedError>>;

#[derive(Debug)]
struct WasNot42Error(i32);
impl Error for WasNot42Error {}
impl Display for WasNot42Error {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		write!(f, "{} was not 42", self.0)
	}
}

static MK_GOOD: TestContinuation<i32> = TestContinuation::INITIAL;
static NORMAL_GOOD: TestContinuation<()> = TestContinuation::INITIAL;
static RESULT_GOOD: TestContinuation<()> = TestContinuation::INITIAL;
static BOTH_GOOD: TestContinuation<()> = TestContinuation::INITIAL;

#[test]
fn mk_good() {
	let executor = StandardExecutor::init();

	__private::exec_noresult(executor, &MK_GOOD, || 42)
}

#[test]
fn normal_good() {
	let mut executor = StandardExecutor::init();
	let Ok(result) = executor.wait(&MK_GOOD) else {
		panic!("test dependency `mk_good` panicked")
	};

	__private::exec_noresult(executor, &NORMAL_GOOD, || assert_eq!(42, result))
}

#[test]
fn result_good() -> Result<(), WasNot42Error> {
	let mut executor = StandardExecutor::init();
	let Ok(result) = executor.wait(&MK_GOOD) else {
		panic!("test dependency `mk_good` panicked")
	};

	__private::exec_result(executor, &RESULT_GOOD, || match result {
		42 => Ok(()),
		_ => Err(WasNot42Error(result)),
	})
}

#[test]
fn both_good() {
	let mut executor = StandardExecutor::init();

	let Ok(_) = executor.wait(&NORMAL_GOOD) else {
		panic!("test dependency `normal_good` panicked")
	};

	let Ok(_) = executor.wait(&RESULT_GOOD) else {
		panic!("test dependency `result_good` panicked")
	};

	__private::exec_noresult(executor, &BOTH_GOOD, || {})
}

static MK_BAD: TestContinuation<i32> = TestContinuation::INITIAL;
static NORMAL_BAD: TestContinuation<()> = TestContinuation::INITIAL;
static DEP_BAD: TestContinuation<()> = TestContinuation::INITIAL;

#[test]
fn mk_bad() {
	let executor = StandardExecutor::init();

	__private::exec_noresult(executor, &MK_BAD, || 41)
}

#[test]
#[should_panic]
fn normal_bad() {
	let mut executor = StandardExecutor::init();
	let Ok(result) = executor.wait(&MK_BAD) else {
		panic!("test dependency `mk_bad` panicked")
	};

	__private::exec_noresult(executor, &NORMAL_BAD, || assert_eq!(42, result))
}

#[test]
#[should_panic]
fn dep_bad() {
	let mut executor = StandardExecutor::init();

	let Ok(_) = executor.wait(&NORMAL_BAD) else {
		panic!("test dependency `normal_bad` panicked")
	};

	__private::exec_noresult(executor, &DEP_BAD, || {})
}
