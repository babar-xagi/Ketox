# Ketox development roadmap

The project goal is to make Rust libraries natural and safe to call from Kotlin. The current implementation targets Kotlin/JVM and establishes the first functions-based development milestone, version `0.0.1`.

The [full design and roadmap](KETOX_FULL_PROJECT_DESIGN_AND_ROADMAP.md) remains the reference for future direction. Its phase numbers and proposed release versions are planning targets, not completed features or release commitments.

## Current milestone: Phase 0 contracts and initial Phase 1 functions

- [x] Rename the project and package namespace to Ketox.
- [x] Set up the Cargo workspace and user-facing `ketox` crate.
- [x] Document architecture, type mapping, naming, ownership, exceptions, loading, and compatibility rules.
- [x] Define metadata schema version 1 and shared validation.
- [x] Implement `#[kotlin_export]` validation and per-export metadata constants.
- [x] Implement single-source-file discovery and deterministic Kotlin/JNI/JSON generation.
- [x] Add a Cargo build helper and a native hello-world example.
- [x] Add `ketox generate` and `ketox inspect` CLI commands.
- [x] Support Phase 1 primitive values, copied strings, borrowed string inputs, and Unit returns.
- [x] Add panic containment and explicit null/UTF-16/string-size error handling.
- [x] Add Rust unit/compile tests and real JVM integration checks.
- [x] Add Windows and Unix test runners and a three-OS CI configuration.
- [ ] Confirm the complete local Rust and JVM checks pass.
- [ ] Run hosted CI and establish the supported platform/architecture matrix.
- [ ] Complete Phase 1 review before considering a `0.1.0` release.

## Validation status

| Environment | Status |
| --- | --- |
| Local Windows Rust checks | 31 tests passed; final formatting/lint/locked checks pending |
| Local Windows JVM checks | Pending the initial validation run |
| GitHub Actions Windows/Linux/macOS | Configured; not yet run |
| Rust 1.88 minimum-version check | Configured in CI; not yet run |
| Android and Kotlin/Native | Not implemented or tested |

The first CI matrix uses the architectures supplied by the selected hosted runners. It does not yet establish both macOS x64 and ARM64 coverage or cross-compilation support.

## Next work

1. Finish the first milestone's verification and review diagnostics, generated API ergonomics, and JNI behavior.
2. Define Phase 2 contracts for `Option`, `Result`, and collections before extending conversion code.
3. Design explicit native ownership and stale-handle protection before introducing exported classes.
4. Add richer models, callbacks, and coroutine integration as separate, tested milestones.
5. Build Android distribution and Gradle integration, then investigate a separate Kotlin/Native backend.

No package publication, release, or licensing decision is part of this initial implementation.
