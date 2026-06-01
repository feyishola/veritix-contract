## Description

<!-- Briefly describe what this PR does and why. -->

Closes #

## Type of Change

- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation
- [ ] CI / Tooling

## Changelog Reminder

<!--
If this PR introduces a change that integrators or contributors should know
about (new feature, bug fix, breaking ABI change, storage layout change, new
event, new function), please add or update an entry in `CHANGELOG.md` under
the `[Unreleased]` section.

If the change is internal (refactor, test, docs-only, config-only), no
changelog entry is needed.
-->

- [ ] I have updated `CHANGELOG.md` with a description of this change.

## Checklist

- [ ] `make test` passes with no failures
- [ ] `make fmt` has been run (no formatting diffs)
- [ ] New logic has at least one test covering the happy path
- [ ] Error and edge cases are tested where practical
- [ ] `cargo clippy --all -- -D warnings` is clean
- [ ] New/updated modules include `//!` module-level docs
- [ ] `storage_types.rs` updated if new storage keys were added
- [ ] `lib.rs` updated if a new module was declared
- [ ] `docs/abi-reference.md` updated if the public interface changed
