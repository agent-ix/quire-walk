---
id: SR-002
title: "Gap analysis of quire-walk PR #1 (FR-356-AC-1..4, TC-898, TC-899)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-walk@296b99313aa6e5d7b80e04dbae136dbf09c04182; PR #1 diff origin/main...HEAD; spec/functional/FR-356-walk-nested-structures-through-one-iterative-walker-toolkit.md; spec/test-cases/TC-898-walker-toolkit-walks-deep-trees-on-a-small-stack.md; spec/test-cases/TC-899-kani-verifies-the-walker-toolkit.md; tests/it/deep.rs; tests/it/leaf.rs; src/proofs.rs"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-walk/FR-356
    type: reviews
---
# Gap analysis of quire-walk PR #1

## Summary

Ticket: none (slug quire-walk-1). `quire coverage --scope .` reports 3/4 rows
backed: 6/6 Rust symbols bound, and FR-356-AC-4 "has no backing symbol".

- FR-356-AC-1: `leaf::quire_walk_has_no_features_and_no_dependency`
  (`#[trace("TC-898", "FR-356-AC-1")]`) checks features and the resolved
  dependency graph. The thumbv7em build half is `make build-no-std`, which
  runs in `make ci`. Backed.
- FR-356-AC-2: three deep.rs tests on a 512 KiB thread (100k chain, 100k
  mutually recursive chain, stop at 50k). Backed.
- FR-356-AC-3: `tc_898_a_100k_node_arena_computes_subtree_sizes_in_one_forward_loop`.
  Backed.
- FR-356-AC-4: ten Kani harnesses exist in src/proofs.rs, but no trace tag
  binds them, and `make kani` is outside `make ci`. I did not run Kani on
  this head.

## Verdict

Approve with findings: one medium, a small in-PR fix (FND-001), and one low.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-356-AC-4 has no backing symbol, so coverage is 3/4 while the FR says "Implemented". The harnesses are the evidence, so tag them in place. Do not add a pseudo-test, and do not add `ix-trace-rs` to `[dependencies]`, which would break AC-1. `quire coverage` reads comment forms (`rust-comment-id`, `rust-doc-comment-id`), so put a `// Trace: TC-899, FR-356-AC-4` line on `check_complete_walk`, `check_stops` and `check_arena` in src/proofs.rs. Then re-run `quire coverage --scope .` and confirm 4/4, since the scanner may skip `cfg(kani)` items. | src/proofs.rs:168,202,233; spec/functional/FR-356-walk-nested-structures-through-one-iterative-walker-toolkit.md:62 |
| FND-002 | low | `tc_898_a_foreign_id_reads_as_none_in_range_or_not` is tagged FR-356-AC-3, but AC-3 is the 100k forward-loop computation. The foreign-id behaviour of `Arena::get` and `Results`/`Computed` has no AC: only TC-899's Kani bound mentions it. Fix: add an FR-356 AC for "an id of another arena reads as `None`" and retag the test to it. | tests/it/deep.rs:297-345; src/arena.rs:154-160 |
