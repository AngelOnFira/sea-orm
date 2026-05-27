//! Compile-fail harness for `Id<E, T>`-wrapped primary keys.
//!
//! Each fixture under `tests/value_type_pk_compile_fail/` must fail to
//! compile. We only assert the must-fail invariant; the captured stderr
//! is rewritten on every run via `TRYBUILD=overwrite` and the
//! `.stderr` files are gitignored. Reasoning out of scope here, but
//! the short version: pinning exact stderr produces churn on every
//! rustc release without catching anything we don't already cover via
//! the runtime examples and the trait-resolution unit tests.

#[test]
fn pk_safety() {
    // SAFETY: env vars touch global state visible to other threads, but
    // this test runs synchronously and the only consumer of `TRYBUILD`
    // is trybuild itself, invoked on the line below.
    unsafe {
        std::env::set_var("TRYBUILD", "overwrite");
    }
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/value_type_pk_compile_fail/*.rs");
}
