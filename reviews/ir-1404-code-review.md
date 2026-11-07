---
id: SR-202611071624001
title: "Code review of quire-walk PR #4: IR-1404 reservation-charged walk"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-walk@b96557b4e7101e933d97ed8cfffa3d56ea3bcdd3; PR #4 diff origin/main...HEAD (Cargo.toml, README.md, src/lib.rs, src/reservation.rs, src/walker.rs, tests/reserved.rs, spec FR-356 and TC-898); rust-review lane folded in"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-walk/FR-356
    type: reviews
---
# Code review of quire-walk PR #4

## Summary

Ticket: IR-1404. Contract: quire-contract-ir PR #948 FR-038 "Owner
prerequisites" (quire-walk caller-reservation variant of `walk`, asking before
each growth of the task and pending stacks at exactly the full new capacity,
returning the denial instead of walking further). Rust-review lane folded in.

Checked:

- `grow` (src/reservation.rs) matches quire-canonical main `Provision::admit`:
  checked `len + additional`, no ask when it fits, target = max(needed,
  2*capacity, 4), charge `size * target`, then `try_reserve_exact`. ZSTs never
  grow and are never charged. Same `reserve(&mut self, bytes) -> Result<(),
  Denied>` shape and unit.
- Task stack: one growth per enter for `1 + pending.len()` before any push, so
  a refusal leaves `tasks` untouched; the initial root growth is charged too.
- `Children::extend` as a per-node `push` loop: still amortized O(1) per child
  (doubling), and visit order is unchanged (pending is drained by pop onto
  tasks exactly as before). No complexity regression.
- `walk` = `walk_reserved(.., &mut Unlimited)`: one implementation. Allocation
  failure used to reach `handle_alloc_error` (abort under std); it now panics
  with the refusal. Capacity overflow already panicked. Documented under
  `# Panics`; not a defect for existing callers.
- Lint: `unsafe_code = "deny"` in Cargo plus `#![forbid(unsafe_code)]` at the
  library root keeps the library (and the cfg(kani) proofs module) at forbid;
  only test/bench targets can opt in with `#[allow]`. Acceptable.
- `cargo test --test reserved`: 4 passed. `cargo kani`: see Verdict.
- A probe walker (deleted after) showed the in-flight-enter behaviour of
  FND-001: budget log `[(64, ok), (32, ok), (64, ok), (128, denied), (8, ok)]`,
  the last request made by `enter` after the walk's denial.

## Verdict

One medium finding, no high. The reservation accounting, refusal of the
growth, list drops and visit order are correct and tested. FND-001 is a real
gap against "returns the denial with no partial side effects": the `enter`
call in flight when `Children::push` is refused cannot observe the refusal and
keeps running. Fix inside this PR (a non-breaking `Children::refused()` query
or a fallible `try_push`, and stop driving the `extend` iterator after a
refusal), with doc wording that says the in-flight `enter` completes.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | A refused pending-list growth inside `enter` is invisible to `enter`: `Children::push` returns `()` and silently drops the node, and `extend` keeps pulling the iterator. The rest of the in-flight `enter` body runs after the denial, can charge and allocate against the same budget (probe: 128 denied, then 8 admitted), and its Break is discarded. The README/rustdoc "no later callback runs" and FR-356 "no callback SHALL run after the refusal" hold only at callback granularity; a consumer such as the FR-038 read (no request after a denied charge) cannot stop its own work. Fix: expose the refusal to `enter` (`Children::refused() -> bool` or `try_push -> Result<(), Refusal>`), make `extend` stop iterating once refused, and state the in-flight behaviour in docs/FR-356. | src/walker.rs:62-80; src/walker.rs:126-134; README.md:103-113; spec/functional/FR-356-walk-nested-structures-through-one-iterative-walker-toolkit.md:54-62 |
