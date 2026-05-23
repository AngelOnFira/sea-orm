//! Trybuild harness pinning the compile-time safety contract for
//! `Id<E, T>`-wrapped primary keys.
//!
//! These fixtures must not compile. If a future change accidentally
//! re-opens the `find_by_id(raw_int)` or cross-PK confusion footguns,
//! the corresponding fixture will start compiling and this test fails.
//!
//! ## Why this is skipped on CI
//!
//! Trybuild does an exact-string comparison against `.stderr` files.
//! rustc's error messages — specifically how it abbreviates type paths
//! — change between versions. For example, rustc 1.88 emits
//! `sea_orm::Id<post::Entity, i32>` while later versions emit
//! `Id<post::Entity, i32>`. Either form is correct; they're just
//! different output for the same underlying error.
//!
//! Rather than pin a specific rustc or constantly rebless the fixtures
//! across stable releases, the trybuild assertion runs only outside CI.
//! Locally, contributors should run this test to verify the diagnostic
//! messages still read well after changes to `FindByIdArg`'s
//! `on_unimplemented` attribute or the role-wrapper naming.
//!
//! The fixtures themselves remain in source — they're still meaningful
//! examples of what should fail to compile, and any breakage there is
//! caught by the per-database integration suites which (a) compile the
//! same test crate and (b) fail loudly if a fixture started compiling.

#[test]
fn pk_safety() {
    // Skip in CI because trybuild's exact `.stderr` match is sensitive
    // to rustc version. See the module docs above.
    if std::env::var("CI").is_ok() {
        eprintln!(
            "Skipping trybuild fixtures: stderr is rustc-version-sensitive. \
             Run locally to verify the diagnostic messages still read well."
        );
        return;
    }
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/value_type_pk_compile_fail/*.rs");
}
