---
id: SR-003
title: "Spec review of quire-walk PR #1: FR-356, TC-898, TC-899 standing alone"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-walk@296b99313aa6e5d7b80e04dbae136dbf09c04182; spec/functional/FR-356-walk-nested-structures-through-one-iterative-walker-toolkit.md; spec/test-cases/TC-898-walker-toolkit-walks-deep-trees-on-a-small-stack.md; spec/test-cases/TC-899-kani-verifies-the-walker-toolkit.md; compared against quire-spec-language's FR-356, TC-898, TC-899 and ADR-030 D-1"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-walk/FR-356
    type: reviews
  - target: ix://agent-ix/quire-walk/TC-898
    type: reviews
  - target: ix://agent-ix/quire-walk/TC-899
    type: reviews
---
# Spec review of quire-walk PR #1

## Summary

Ticket: none (slug quire-walk-1). `quire validate` passes all three files
(module warnings only). The text stands alone well:

- FR-356 dropped QSL's AC-5..7, arch-lint, `qsl-walk-grow`, the qualified
  core, US-027, FR-256/258 and the Linear ids. Its four ACs are all toolkit
  behaviour.
- TC-898/899 point at this repo's files and `make` targets.
- No private paths and no quire-research reference. The references to
  private QSpec FR-460 come from a private repo. QSL's ADRs are public.

## Verdict

Approve with findings: one medium (cross-repo spec copy, to be fixed in QSL)
and three low wording fixes.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-356-AC-1..4, TC-898 and TC-899 now exist in both repos. QSL's FR-356 still states AC-1..4 (AC-1 with extra arch-lint clauses), and QSL still has its own TC-898 and TC-899. The two copies will drift. Fix in the QSL follow-up: QSL's FR-356 drops AC-1..4 and behaviours 1 to 4 for a `depends_on` edge to ix://agent-ix/quire-walk/FR-356, keeps its own AC-5..7, and deletes or retargets TC-898/899. | spec/functional/FR-356-walk-nested-structures-through-one-iterative-walker-toolkit.md:1-75; ~/dev/quire-spec-language/spec/functional/FR-356-walk-nested-structures-through-one-iterative-walker-toolkit.md |
| FND-002 | low | Description and Dependencies cite "ADR-030 D-1 items 2, 6 and 7" and "2, 3, 6 and 7, and D-6". Items 6 and 7 and D-6 are QSL qualified-core rules: the core is iterative only, and every core entry point gets a deep test. They are not toolkit behaviour, and this FR dropped the behaviours that cited them. Fix: cite D-1 items 2 and 3 only. | spec/functional/FR-356-walk-nested-structures-through-one-iterative-walker-toolkit.md:17-18,70 |
| FND-003 | low | Behavior 2 says "a walk's memory is bounded by the nodes its stage charges (ADR-030 D-1 item 3)". "Its stage charges" is QSL limit vocabulary, which a standalone toolkit cannot check. Fix: state the toolkit's own bound, one frame per entered node not yet exited plus one entry per named child not yet entered, as walker.rs documents. | spec/functional/FR-356-walk-nested-structures-through-one-iterative-walker-toolkit.md:43-45 |
| FND-004 | low | AC-4's Verification cell "Kani proof" is not a catalogued verification method. `quire coverage` warns uncatalogued-verification-method, so nothing says what discharging it means. Fix: use a declared catalog method (for example Analysis or Test) and keep "Kani (TC-899)" in the text. | spec/functional/FR-356-walk-nested-structures-through-one-iterative-walker-toolkit.md:62 |

## Dispositions

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | deferred | Fixable only in quire-spec-language: origin/main still has its own TC-898, TC-899 and FR-356-AC-1..4. The QSL follow-up replaces them with a depends_on edge to ix://agent-ix/quire-walk/FR-356; it needs a ticket. |
| FND-002 | fixed | e8d5c82 |
| FND-003 | fixed | e8d5c82 |
| FND-004 | fixed | e8d5c82 |
