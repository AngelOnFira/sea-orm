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
//! ### Known coverage gap
//!
//! The fixtures under `tests/value_type_pk_compile_fail/` are only ever
//! compiled by trybuild. When this test returns early on CI, those
//! fixtures are skipped entirely — they are not wired in as a `[[test]]`
//! target and the per-database integration suites do not pull them in.
//! That means a regression which makes one of the fixtures start
//! compiling on the CI rustc version will not be caught here. The
//! workaround is to run this test locally before merging.
//!
//! A future improvement is to add a CI-friendly "compile-fail smoke" run
//! that invokes `rustc` on each fixture and checks only the exit code
//! (ignoring stderr), so the must-not-compile invariant is exercised
//! without the version-sensitive stderr diff.

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
