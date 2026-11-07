---
id: SR-202611071624002
title: "Gap analysis of quire-walk PR #4: IR-1404 reservation-charged walk"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-walk@b96557b4e7101e933d97ed8cfffa3d56ea3bcdd3; PR #4 diff origin/main...HEAD (spec FR-356 behaviour 5, FR-356-AC-6, TC-898 steps 7-9, tests/reserved.rs, src/reservation.rs, src/walker.rs)"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-walk/FR-356
    type: reviews
---
# Gap analysis of quire-walk PR #4

## Summary

Ticket: IR-1404. Plan completion: not assessed (planless).

FR-356 behaviour 5 and FR-356-AC-6 are new; TC-898 steps 7-9 trace them to
`tests/reserved.rs` (four `#[trace("TC-898", "FR-356-AC-6")]` tests). Checked
AC-6 clause by clause:

- ask once per growth with size times new capacity, sequence equals the
  allocator's sizes: `charged_bytes_are_the_bytes_requested_from_the_allocator`.
- exact total gives the visit order of `walk`:
  `an_exact_provision_admits_with_the_visit_order_of_walk`.
- every smaller total refused with `Refusal::Denied`, no visit after, only
  admitted requests reach the allocator, no live byte left:
  `every_smaller_provision_refuses_with_no_further_visit_and_no_heap_left`.
- 100,000-deep chain on 512 KiB: `a_deep_input_walks_and_refuses_on_a_small_stack`
  (see FND-001).

No new production code lacks an owning requirement: `Refusal::Allocation` and
the refusal-over-Break precedence are in the FR-356 behaviour text/rustdoc;
neither is an AC clause. No stubs.

## Verdict

One low finding. AC-6 is otherwise fully backed by tests that can fail (the
coder reports two red mutations).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | AC-6 "a 100,000-deep chain does the same" and TC-898 step 9 "with the exact total ... the chain completes with the exact total" are not what the test runs: the chain's complete walk uses `Budget::new(usize::MAX)`, not the exact total, so exact-provision admission on the deep chain is unasserted (only one-byte-short refusal is). Fix: run the chain a second time with `Budget::new(total)` and assert it completes with `remaining == 0`. | tests/reserved.rs:262-283; spec/test-cases/TC-898-walker-toolkit-walks-deep-trees-on-a-small-stack.md:39-40,54-56 |
