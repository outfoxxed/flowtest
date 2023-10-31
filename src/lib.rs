//! Tests that depend on other tests
//!
//! # Example
//!
//! ```
//! #[test]
//! #[flowtest]
//! fn init_complex_type() -> i32 {
//!     // test that initialization works for our complex type
//!     if false { panic!("oh no!") };
//!     42
//! }
//!
//! # #[derive(Debug)]
//! # struct ComplexTypeInitFailed;
//! # impl std::error::Error for ComplexTypeInitFailed {}
//! # impl std::fmt::Display for ComplexTypeInitFailed {
//! #     fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
//! #         unimplemented!()
//! #     }
//! # }
//! #[test]
//! #[flowtest(init_complex_type: value)]
//! fn mutate_complex_type() -> Result<i32, ComplexTypeInitFailed> {
//!     // mutate our type in a way that could fail
//!     if false {
//!         Err(ComplexTypeInitFailed)
//!     } else {
//!         Ok(value + 5)
//!     }
//! }
//!
//! #[test]
//! #[flowtest(init_complex_type: value)]
//! fn mutate_complex_type_differently() -> i32 {
//!     // mutate our type in a different way
//!     if false {
//!         panic!("oh no!")
//!     } else {
//!         value + 5
//!     }
//! }
//!
//! #[test]
//! #[flowtest(
//!     mutate_complex_type,
//!     mutate_complex_type_differently,
//! )]
//! fn ensure_mutations_are_consistent() {
//!     assert_eq!(mutate_complex_type, mutate_complex_type_differently);
//! }
//! ```
//!
//! For details see [`#[flowtest]`](flowtest).

pub mod standard_executor;

#[doc(hidden)]
#[path = "macro_support.rs"]
pub mod __private;

/// Test execution state holder
///
/// Note: this type should not be considered stable.
pub trait Continuation {
	const INITIAL: Self;
}

/// Test executor
///
/// Alternate test executors can be used via
/// [`#[flowtest]`](flowtest)'s hidden `executor` parameter:
///
/// ```
/// # use flowtest::flowtest;
/// #[flowtest(
///    executor = flowtest::standard_executor::StandardExecutor,
///    // ...
/// )]
/// # fn test() {}
/// ```
///
/// Note: this type should not be considered stable.
pub trait Executor {
	type Continuation<T: Clone>: Continuation;

	/// Initialize the test executor
	///
	/// Called once per test.
	fn init() -> Self;

	/// Wait for another test's [Continuation]
	///
	/// Will be called once for each dependency, which might have already completed.
	fn wait<T: Clone>(&mut self, continuation: &Self::Continuation<T>) -> T;

	/// Execute flowtest's test harness
	///
	/// Note: The harness's second return value should be passed to the caller.
	fn execute<T: Clone, P>(
		self,
		tester: impl FnOnce() -> (T, P),
		continuation: &Self::Continuation<T>,
	) -> P;
}

/// This macro allows a test to wait on dependencies and be waited on by dependents
///
/// # Usage
/// The `#[flowtest]` attribute should be added to test functions that produce or consume
/// values from other tests. Values returned by the function will be consumable by other
/// tests, and test failures will cause all dependent tasks to fail.
/// Note that values must be `Clone` to send between tests.
///
/// *Note that you must also specify a normal test macro,
/// for example* `#[test]` *or* `#[tokio::test]` *for the test to run.*
///
/// Tests may return a result. Returning `Err` is considered a test failue and is
/// passed to the test harness. A test is determined to return a result if the return
/// type's name is `Result`, for example `std::result::Result<_, _>` or `anyhow::Result<_>`.
/// If you want return a result as a value to be consumed by another test then the test's
/// *return handler* can be set to `data`.
/// If you want to treat a type not named `Result` as a result, then the test's
/// *return handler* can be set to `result`.
///
/// ## Syntax
/// ```ignore
/// #[flowtest((
///     dependency1,
///     dependency2: renamed_return_value,
///     dependency3: (pattern, matched, return_value),
/// ) -> return_handler)]
/// ```
///
/// There is also a short form if you do not need to change the return handler.
///
/// ```
/// # use flowtest::flowtest;
/// # #[flowtest] fn dependency1() {};
/// # #[flowtest] fn dependency2() {};
/// # #[flowtest] fn dependency3() -> ((), (), ()) { ((), (), ()) };
/// #[flowtest(
///     dependency1,
///     dependency2: renamed_return_value,
///     dependency3: (pattern, matched, return_value),
/// )]
/// # fn test() {}
/// ```
///
/// # Footguns
/// - **Test macro requirement**: Flowtest does not automatically add `#[test]`.
///   You should add `#[test]` or a stand-in above the `#[flowtest]` attribute.
///   Failure to do so will result in the test never running and deadlocking all
///   dependent tasks as explained below.
/// - **Deadlocks**: Tests will deadlock if the test harness does not run enough
///   concurrent tests for all of a test's dependencies to be satisfied,
///   for example if the test harness only has one thread and a test with
///   an unsatisifed dependency is run. **Under normal circumstances this will
///   not be an issue**.
///   Note: The rust test harness currently runs tests in alphabetical order.
///   If this is an issue for you, you can try alphabetically ordering your
///   tests to match the dependency order, or give the test harness more threads.
/// - **Attribute Macro Order**: Attribute macros are applied in the order they are
///   listed on a function as shown below.
///   ```
///   // Anything this macro does will be wrapped by flowtest's executor.
///   // This means it will not start until all test dependencies have been resolved.
///   // Most attributes should go here.
///   # use flowtest::flowtest;
///   # use test as runs_before;
///   # use test as runs_after;
///   #[runs_before]
///   #[flowtest]
///   // Anything this macro does will wrap flowtest's executor.
///   // This means it will wait an arbitrary amount of time for
///   // the thread to resume and the test to run.
///   #[runs_after]
///   fn test() {}
///   ```
///
/// # Example
/// See the [crate level docs](crate).
pub use flowtest_macro::flowtest;
