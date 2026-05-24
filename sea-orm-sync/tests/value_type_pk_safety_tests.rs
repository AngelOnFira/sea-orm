//! Trybuild harness pinning the compile-time safety contract for
//! `Id<E, T>`-wrapped primary keys.
//!
//! Each fixture under `tests/value_type_pk_compile_fail/` must fail to
//! compile. Two layers of assertion:
//!
//! 1. **Trybuild exact-match**: the captured stderr must match the
//!    committed `.stderr` snapshot character-for-character. This
//!    catches subtle changes in how rustc presents the error
//!    (formatting tweaks, new hint lines, reordered notes, etc.).
//!    Snapshots are generated against CI's pinned stable rustc; when
//!    CI updates to a newer rustc and the snapshots drift, the test
//!    fails loudly and contributors regenerate them with
//!    `TRYBUILD=overwrite cargo test --test value_type_pk_safety_tests`.
//!
//! 2. **Substring check**: each fixture declares
//!    `// expect-error: <substring>` directives in its header; the
//!    harness asserts each substring appears in the captured stderr.
//!    Substrings are our own prose (the
//!    `#[diagnostic::on_unimplemented]` messages) or our own type
//!    names — both stable across rustc upgrades. This layer catches
//!    "must-fail invariant held but for the wrong reason" — e.g. the
//!    fixture is failing on a syntax error rather than the trait-bound
//!    error we wanted.
//!
//! Local development tip: if a snapshot diff looks cosmetic and your
//! `rustc --version` differs from CI's stable, the snapshots may be
//! stale relative to your toolchain. Either install CI's stable
//! (`rustup install <ci-version>` + `rustup override set <ci-version>`
//! in this repo dir) or regenerate with `TRYBUILD=overwrite` and let
//! CI confirm.

use std::fs;
use std::path::{Path, PathBuf};

const FIXTURE_DIR: &str = "tests/value_type_pk_compile_fail";

#[test]
fn pk_safety() {
    // Trybuild's actual compilation happens at `TestCases::drop` time.
    // Scope it so the .stderr files are populated before we read them
    // for the substring layer below. Trybuild will panic in drop if a
    // fixture compiles when it shouldn't, OR if a fixture's captured
    // stderr drifts from the committed snapshot — both load-bearing.
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
