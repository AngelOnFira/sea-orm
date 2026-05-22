//! Trybuild harness pinning the compile-time safety contract for
//! newtype-wrapped primary keys.
//!
//! These fixtures must not compile. If a future change accidentally
//! re-opens the `find_by_id(raw_int)` or cross-PK confusion footguns,
//! the corresponding fixture will start compiling and this test fails.

#[test]
fn pk_safety() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/value_type_pk_compile_fail/*.rs");
}
