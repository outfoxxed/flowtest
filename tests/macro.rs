use std::{
	error::Error,
	fmt::{self, Display, Formatter},
};

use flowtest::flowtest;

#[derive(Debug, Clone, Copy)]
struct WasNot42Error(i32);
impl Error for WasNot42Error {}
impl Display for WasNot42Error {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		write!(f, "{} was not 42", self.0)
	}
}

#[test]
#[flowtest]
fn mk_good() -> i32 {
	42
}

#[flowtest]
#[test]
fn mk_good_opposite_attrs() -> i32 {
	42
}

#[test]
#[flowtest(() -> data)]
fn mk_good_data() -> i32 {
	42
}

type AliasedResult<T> = Result<T, ()>;

#[test]
#[flowtest(() -> result)]
fn mk_good_result() -> AliasedResult<i32> {
	Ok(42)
}

#[test]
#[flowtest(() -> data)]
fn mk_good_result_data() -> Result<i32, ()> {
	Ok(42)
}

#[test]
#[flowtest(
	mk_good,
	mk_good_opposite_attrs,
	mk_good_data,
	mk_good_result,
	mk_good_result_data
)]
fn test_good() {
	assert_eq!(42, mk_good);
	assert_eq!(42, mk_good_opposite_attrs);
	assert_eq!(42, mk_good_data);
	assert_eq!(42, mk_good_result);
	assert_eq!(Ok(42), mk_good_result_data);
}

#[test]
#[flowtest(mk_good: result)]
fn normal_good() {
	assert_eq!(42, result);
}

#[test]
#[flowtest(mk_good: result)]
fn result_good() -> Result<(), WasNot42Error> {
	match result {
		42 => Ok(()),
		_ => Err(WasNot42Error(result)),
	}
}

#[test]
#[flowtest(mk_good)]
fn normal_good_add1() -> i32 {
	assert_eq!(42, mk_good);

	mk_good + 1
}

#[test]
#[flowtest(mk_good)]
fn result_good_add2() -> Result<i32, WasNot42Error> {
	match mk_good {
		42 => Ok(mk_good + 1),
		_ => Err(WasNot42Error(mk_good)),
	}
}

#[test]
#[flowtest(
	executor = flowtest::standard_executor::StandardExecutor,
	mk_good: _,
	normal_good: _,
	result_good: _,
	normal_good_add1,
	result_good_add2,
)]
fn all_good() {
	assert_eq!(normal_good_add1, result_good_add2);
}
