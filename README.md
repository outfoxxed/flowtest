# Flowtest
Tests that depend on other tests

See [the docs](https://docs.rs/flowtest) for details.

## Example
```rs
#[test]
#[flowtest]
fn init_complex_type() -> i32 {
    // test that initialization works for our complex type
    if false { panic!("oh no!") };
    42
}

#[test]
#[flowtest(init_complex_type: value)]
fn mutate_complex_type() -> Result<i32, ComplexTypeInitFailed> {
    // mutate our type in a way that could fail
    if false {
        Err(ComplexTypeInitFailed)
    } else {
        Ok(value + 5)
    }
}

#[test]
#[flowtest(init_complex_type: value)]
fn mutate_complex_type_differently() -> i32 {
    // mutate our type in a different way
    if false {
        panic!("oh no!")
    } else {
        value + 5
    }
}

#[test]
#[flowtest(
    mutate_complex_type,
    mutate_complex_type_differently,
)]
fn ensure_mutations_are_consistent() {
    assert_eq!(mutate_complex_type, mutate_complex_type_differently);
}
```
