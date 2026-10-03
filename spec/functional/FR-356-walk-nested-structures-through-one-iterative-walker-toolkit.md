---
id: FR-356
title: "Walk nested structures through one iterative walker toolkit"
type: FR
relationships:
  - target: ix://agent-ix/quire-spec-language/ADR-030
    type: depends_on
  - target: ix://agent-ix/quire-spec-language/ADR-011
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-460
    type: depends_on
---
# FR-356: Walk nested structures through one iterative walker toolkit

## Description

`quire-walk` SHALL let a walk over a nested structure run in arena order or
on one shared `no_std` walker toolkit, so that no walk's native stack use
grows with the input's depth (ADR-030 D-1 item 2; QSpec FR-460).
The behaviours that make QSL's own walks use the toolkit stay in the
quire-spec-language repository under the same id; this requirement is the
toolkit's own behaviour.

## Use case

A QSL stage, or an IR, CG or RT stage, walks a 100,000-deep tree. It runs the
walk on the toolkit's heap stack, so the walk finishes with constant native
stack use. An assessor who qualifies a core reads one Kani-verified
traversal instead of a hand-written stack in each walk.

## Behavior

1. **Arena order.** A tree stored as an arena SHALL hold each node after its
   children, and a bottom-up computation over it SHALL be one forward loop
   over the arena, with no stack.
2. **The walker toolkit.** A top-down or mutually recursive walk SHALL run on
   the walker toolkit. The toolkit SHALL keep an explicit heap stack of
   frames, each frame a value of a type the walk declares, and SHALL call
   the walk's enter callback when it pushes a node's frame and its exit
   callback with that frame when it pops it. An enter callback SHALL name
   the children to push, possibly of another node type, and MAY stop the
   walk with a value, which the toolkit returns with no further callback.
   The toolkit SHALL grow its heap stack by one frame per entered node
   not yet exited plus one entry per named child not yet entered, so the
   stack's size is bounded by the nodes the walk names (ADR-030 D-1 item 3).
3. **One shared leaf crate.** The toolkit SHALL be the crate `quire-walk`,
   a `#![no_std]` shared leaf that depends only on `core` and `alloc` and
   has no features (ADR-011 §6.1 layer W, FB-05).
4. **Kani-verified.** The toolkit SHALL carry Kani harnesses proving, for
   every tree within each harness's bound, that every node is entered and
   exited exactly once, that each exit receives the frame its enter pushed,
   that exits come in the reverse order of their enters along each path,
   that a stop ends the walk at once, and that no traversal panics.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-356-AC-1 | `quire-walk` builds for `thumbv7em-none-eabi` with `#![no_std]`, and it has no features and a dependency tree of only `core` and `alloc`. | Test (TC-898) |
| FR-356-AC-2 | On a thread with a 512 KiB stack, a toolkit walk over a 100,000-deep chain enters every node once in pre-order and exits every node once in post-order, each exit receiving the frame its enter pushed. A mutually recursive walk over a 100,000-level chain that alternates an expression node and a type node does the same, each level's frame of its own type. An enter callback that stops at depth 50,000 returns its value, and no callback runs after it. | Test (TC-898) |
| FR-356-AC-3 | A bottom-up computation over a 100,000-node arena is one forward loop over the arena and completes on a thread with a 512 KiB stack. | Test (TC-898) |
| FR-356-AC-4 | `cargo kani` on the toolkit crate verifies every harness: each node entered and exited exactly once, each exit receiving its enter's frame, exits in reverse enter order along each path, a stop ending the walk at once, and no panic, for every tree within the harness bound. | Analysis (Kani, TC-899) |
| FR-356-AC-5 | An id of another arena of the same node type reads as `None` from `Arena::get`, from the results view during `bottom_up` and from the results it returns, whether its position is in range or past the end, and so does an id of the arena a clone was made from. | Test (TC-898) |

## Status

Implemented.

## Dependencies

- QSL ADR-030 D-1 items 2 and 3.
- QSL ADR-011 §6.1 layer W and FB-05's shared-leaf class.

## References

- QSpec FR-460, the ecosystem depth rule.
