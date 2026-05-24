//! Trybuild harness pinning the compile-time safety contract for
//! `Id<E, T>`-wrapped primary keys.
//!
//! Each fixture under `tests/value_type_pk_compile_fail/` must fail to
//! compile. The harness:
//!
//! 1. Runs trybuild with `TRYBUILD=overwrite`, so trybuild still
//!    enforces the must-fail invariant but silently refreshes the
//!    `.stderr` snapshot on cosmetic rustc-version drift.
//! 2. After compilation, reads each `.stderr` and checks that the
//!    `// expect-error: <substring>` directives at the top of the
//!    fixture all appear in the captured output.
//!
//! Directives let us pin "the error mentions our diagnostic / our
//! trait name / the offending type" without depending on exact rustc
//! formatting. The substrings are usually our own prose (the
//! `#[diagnostic::on_unimplemented]` messages) or our own type names —
//! both stable across rustc upgrades.
//!
//! Committed `.stderr` files are kept as a debugging reference but
//! are no longer authoritative: CI rewrites them on every run, and
//! contributors should glance at them rather than match them
//! character-for-character.

use std::fs;
use std::path::{Path, PathBuf};

const FIXTURE_DIR: &str = "tests/value_type_pk_compile_fail";

#[test]
fn pk_safety() {
    // Tell trybuild not to fail on stderr-snapshot mismatches; the
    // must-fail-to-compile check is still enforced (a fixture that
    // accidentally starts compiling still panics inside trybuild's
    // drop, failing this test).
    //
    // SAFETY: `set_var` is `unsafe` because env vars affect global
    // state visible to other threads. This test runs serially within
    // the test binary and the only consumer of `TRYBUILD` is trybuild
    // itself, invoked synchronously below.
    unsafe {
        std::env::set_var("TRYBUILD", "overwrite");
    }

    // Trybuild's actual compilation happens at `TestCases::drop` time.
    // Scope it so the .stderr files are populated before we read them.
    {
        let t = trybuild::TestCases::new();
        t.compile_fail(format!("{FIXTURE_DIR}/*.rs"));
    }

    // Walk each fixture and verify its `expect-error:` directives are
    // present in the captured stderr.
    let fixtures = list_fixtures(FIXTURE_DIR);
    assert!(
        !fixtures.is_empty(),
        "no compile-fail fixtures discovered under {FIXTURE_DIR}"
    );

    let mut failures: Vec<String> = Vec::new();
    for fixture in fixtures {
        let directives = parse_expect_directives(&fixture);
        assert!(
            !directives.is_empty(),
            "{}: every compile-fail fixture must declare at least one \
             `// expect-error: <substring>` directive in its header",
            fixture.display()
        );

        let stderr_path = fixture.with_extension("stderr");
        let stderr = match fs::read_to_string(&stderr_path) {
            Ok(s) => s,
            Err(e) => {
                failures.push(format!(
                    "{}: could not read {} — trybuild should have produced \
                     it (TRYBUILD=overwrite). I/O error: {e}",
                    fixture.display(),
                    stderr_path.display()
                ));
                continue;
            }
        };

        for needle in &directives {
            if !stderr.contains(needle) {
                failures.push(format!(
                    "{}: stderr is missing expected substring {needle:?}.\n\
                     Full stderr captured by trybuild:\n{stderr}",
                    fixture.display()
                ));
            }
        }
    }

    assert!(
        failures.is_empty(),
        "compile-fail substring check failed:\n\n{}",
        failures.join("\n----\n")
    );
}

fn list_fixtures(dir: &str) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let read = match fs::read_dir(dir) {
        Ok(r) => r,
        Err(_) => return out,
    };
    for entry in read.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
    out.sort();
    out
}

/// Parse `// expect-error: <substring>` lines from the fixture header.
/// Tolerates leading whitespace, `//!` and `//` styles.
fn parse_expect_directives(fixture: &Path) -> Vec<String> {
    let source = fs::read_to_string(fixture)
        .unwrap_or_else(|e| panic!("can't read {}: {e}", fixture.display()));

    source
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim_start();
            let body = trimmed
                .strip_prefix("//!")
                .or_else(|| trimmed.strip_prefix("//"))?;
            let body = body.trim_start();
            body.strip_prefix("expect-error:")
                .map(|s| s.trim().to_string())
        })
        .filter(|s| !s.is_empty())
        .collect()
}
