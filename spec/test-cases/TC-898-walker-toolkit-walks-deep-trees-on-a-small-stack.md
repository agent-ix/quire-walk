---
id: TC-898
title: "The walker toolkit walks deep trees on a small stack"
type: TC
relationships:
  - target: ix://agent-ix/filament-walk/FR-356
    type: verifies
---
# TC-898: The walker toolkit walks deep trees on a small stack

## Description

Verify that the walker toolkit `filament-walk` is a `no_std` shared leaf, that it enters and exits
every node of a 100,000-deep tree in order with typed frames, that a stop
ends the walk, and that arena order computes bottom-up with no stack.

Scope: FR-356-AC-1, FR-356-AC-2, FR-356-AC-3, FR-356-AC-5, FR-356-AC-6.

## Test Procedure

Run steps 2 to 6 on a thread spawned with a 512 KiB stack.

1. Build `filament-walk` for `thumbv7em-none-eabi`, and list its features
   and its dependency tree.
2. Walk a 100,000-deep chain, recording each enter and exit with the depth
   carried in its frame.
3. Walk a 100,000-level chain alternating an expression node and a type node,
   each with its own frame type, recording enters and exits.
4. Walk the chain of step 2 with an enter callback that stops at depth
   50,000 with a value.
5. Compute each node's subtree size over a 100,000-node arena in one forward
   loop.
6. Read an id of another arena, in range and past the end, through
   `Arena::get`, the results view and the returned results.
7. In a test binary with a counting allocator, walk a 41-node tree with
   `walk_reserved` and an unlimited reservation, recording each reservation
   request and each allocator request.
8. Walk it again with a reservation that admits exactly the total of step 7.
9. Walk it with every smaller total, and walk a 100,000-deep chain on a
   512 KiB thread with the exact total and with one byte less.

## Expected Results

- Step 1: the build succeeds, it has no features, and the tree holds only
  `core` and `alloc`.
- Steps 2 and 3: every node is entered once in pre-order and exited once in
  post-order, and each exit's frame equals its enter's frame.
- Step 4: the walk returns the value, and no callback runs after the stop.
- Step 5: the root's size is 100,000.
- Step 6: every foreign id reads as `None`.
- Step 7: the requests equal the allocator's requests, in order.
- Step 8: the walk completes with the visit order of `walk`.
- Step 9: each smaller total is refused with no visit after the denial, no
  extra allocator request and no live byte left; the chain completes with
  the exact total and is refused with one byte less.

## Status

Implemented. Step 1 is backed by `make build-no-std` and `tests/it/leaf.rs`; steps 2 to 6 by `tests/it/deep.rs`; steps 7 to 9 by `tests/reserved.rs`.
