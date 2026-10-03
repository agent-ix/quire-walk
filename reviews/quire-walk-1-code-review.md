---
id: SR-001
title: "Code review of quire-walk PR #1: import quire-walk from quire-spec-language"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-walk@296b99313aa6e5d7b80e04dbae136dbf09c04182; PR #1 diff origin/main...HEAD (Cargo.toml, Cargo.lock, Makefile, clippy.toml, deny.toml, rust-toolchain.toml, CLAUDE.md, src/lib.rs, src/walker.rs, src/arena.rs, src/proofs.rs, tests/it/main.rs, tests/it/deep.rs, tests/it/leaf.rs, removed tests/integration.rs) plus the scaffold commit b294532 (.gitignore, .github/workflows/ci.yml, scripts/check_unsafe_comments.sh, scripts/unsafe_comment_baseline.txt, AGENTS.md, README.md); rust-review lane folded in"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-walk/FR-356
    type: reviews
---
# Code review of quire-walk PR #1

## Summary

Ticket: none in the branch or title (`task/import-quire-walk`); review slug
quire-walk-1. Rust-review lane folded in.

Checked:

- `diff -r src ~/dev/quire-spec-language/quire-walk/src` is empty: src/ is
  byte-identical to QSL's local checkout (c83be7111).
- `tests/it/deep.rs` is identical to QSL's `quire-walk/tests/it/deep.rs`.
  `tests/it/leaf.rs` differs from QSL's `tests/it/quire_walk_leaf.rs` only in
  comments and assert messages (the workspace wording was dropped).
- `walk` in src/walker.rs: explicit `Vec<Task>` stack, `?` on both callbacks,
  `pending` reused and dropped on a break. No panic path, no recursion.
  src/arena.rs: `get` checks the arena token before indexing with
  `Vec::get`. No `unwrap`, `expect` or `as` casts in library code.
  `unsafe_code = "forbid"`, `#![no_std]`, empty `[dependencies]`.
- `.gitignore` has unanchored `target/`, `*-target/`, `target-*/` and
  `.worktrees/`. `git ls-files` holds no build artifacts.
- `make ci` runs `build-no-std` (`cargo build --locked --target
  thumbv7em-none-eabi`) and a no_std clippy pass. The log at
  ~/dev/worktrees/logs/quire-walk-import-make-ci.log shows both green, exit 0.
- No quire-research reference, no local path, no Linear id in the tree.

## Verdict

Approve with findings: no high. FND-001 is the cross-repo copy this
extraction creates. It cannot be fixed in this PR; it must be ticketed and
closed in QSL. FND-002 to FND-004 are low.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The extraction leaves two copies of the crate. QSL still carries `quire-walk/src` (byte-identical, 711 lines), `quire-walk/tests/it/deep.rs` and `tests/it/quire_walk_leaf.rs`. Until QSL deletes its `quire-walk/` crate and depends on agent-ix/quire-walk by git, the two copies drift. Fix: a QSL follow-up ticket that switches QSL to the git dependency and deletes the copy. The copy is gone only when the QSL path 404s. | src/; ~/dev/quire-spec-language/quire-walk/src; ~/dev/quire-spec-language/tests/it/quire_walk_leaf.rs |
| FND-002 | low | `.github/workflows/ci.yml` does not match `make ci`. It skips `build-no-std` and the thumbv7em clippy pass, and it runs `cargo deny check licenses`, while the Makefile runs a full `cargo deny check`, so the `allow-git` source rule is never exercised in the workflow. The no_std build is covered only by local `make ci`. Fix: have the workflow run `make ci`. Workflow edits need Peter's clearance first. | .github/workflows/ci.yml:24-57; Makefile:38-56,97 |
| FND-003 | low | The unsafe-comment audit guards nothing. `[lints.rust] unsafe_code = "forbid"` already refuses every `unsafe` block, so `scripts/check_unsafe_comments.sh`, the empty `scripts/unsafe_comment_baseline.txt`, `make audit-unsafe`, the CI "Unsafe audit" step and the CLAUDE.md paragraph about them are scaffold ceremony. Fix: delete them. | Cargo.toml:30; scripts/check_unsafe_comments.sh; scripts/unsafe_comment_baseline.txt; Makefile; CLAUDE.md:25 |
| FND-004 | low | QSL-specific wording came along into the standalone crate. The Cargo.toml comment says feature unification cannot bring "`std` nor `stacker`" into it (`stacker` is QSL's `qsl-walk-grow` concern). The walker.rs doc says the stack is bounded by nodes "the calling stage charges (ADR-030 D-1 item 3)". The proofs.rs doc says `cargo kani -p quire-walk` (QSL workspace form). The package `description` leads with "ADR-011 §6.1 layer W". Fix: state the toolkit's own bound (one frame per entered node not yet exited, plus one entry per named child not yet entered) and drop the QSL-internal labels. | Cargo.toml:6,16-18; src/walker.rs:79-82; src/proofs.rs:2 |
