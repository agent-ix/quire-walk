# quire-walk

Walk a deeply nested structure without using the call stack, and compute over
one without any stack at all. `quire-walk` is a small `no_std` Rust toolkit that
keeps its own stack of frames on the heap.

A tree walk is usually written as a function that calls itself for each child.
That is one native stack frame per level, so an input that nests deeply enough
(a rule with 100,000 chained operators, say) overflows the stack and aborts the
process. Quire's parsers, checkers and evaluators read input they did not write,
so none of them can afford that. `quire-walk` is the shared piece that removes
the problem: you describe what to do when a node is entered and when it is left,
and the toolkit runs the traversal from a loop. Native stack use is constant at
any depth.

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/images/hero-dark.svg">
  <img alt="Native recursion keeps one call frame per level of the input, so a 100,000-level tree overflows a 512 KiB stack. quire-walk keeps one typed frame per level on a heap stack and runs enter and exit callbacks from a loop, so native stack use stays constant." src="docs/images/hero-light.svg">
</picture>

> **Status: public prerelease.** The API is still changing and the crate is not
> published (`publish = false`); depend on it by git. Expect breaking changes.

## A first walk

The smallest real use is the doc-test in [`src/lib.rs`](src/lib.rs). It counts
the nodes of a tree held as lists of child indexes, then stops early at a named
node:

```rust
use core::ops::ControlFlow;
use quire_walk::{walk, Children, Walk};

struct Tree(Vec<Vec<usize>>);

struct Count<'a> {
    tree: &'a Tree,
    entered: usize,
    stop_at: Option<usize>,
}

impl Walk for Count<'_> {
    type Node = usize;
    type Frame = usize;
    type Stop = &'static str;

    fn enter(
        &mut self,
        node: usize,
        children: &mut Children<'_, usize>,
    ) -> ControlFlow<&'static str, usize> {
        if self.stop_at == Some(node) {
            return ControlFlow::Break("found");
        }
        self.entered += 1;
        children.extend(self.tree.0[node].iter().copied());
        ControlFlow::Continue(node)
    }

    fn exit(&mut self, _frame: usize) -> ControlFlow<&'static str> {
        ControlFlow::Continue(())
    }
}

let tree = Tree(vec![vec![1, 2], vec![], vec![3], vec![]]);
let mut count = Count { tree: &tree, entered: 0, stop_at: None };
assert_eq!(walk(&mut count, 0), ControlFlow::Continue(()));
assert_eq!(count.entered, 4);

let mut find = Count { tree: &tree, entered: 0, stop_at: Some(2) };
assert_eq!(walk(&mut find, 0), ControlFlow::Break("found"));
```

## How a walk runs

You implement the `Walk` trait, which declares three types and two callbacks:

- `Node` is the handle of a node to visit (a reference or an index), `Frame` is
  what you keep for a node between entering and leaving it, and `Stop` is the
  value you return to end the walk early.
- `enter` is called once per node, before any of its children. It names the
  children to visit by pushing them onto `children`, and returns the node's
  frame.
- `exit` is called once per node, after every child has been exited, and gets
  that same frame back.

`walk` visits each child and its whole subtree before the next child. On any
path from the root to a leaf, exits come in the reverse order of the enters.
Returning `ControlFlow::Break` from either callback ends the walk at once:
`walk` returns that value, no callback runs after it, and children already named
are dropped.

The figure shows the walk from the example above. The heap stack holds one
frame for each node entered but not yet exited, plus one entry for each child
named but not yet entered, so its size follows the nodes the walk names and
never the call depth.

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/images/order-dark.svg">
  <img alt="A walk of the tree where node 0 has children 1 and 2 and node 2 has child 3 enters 0, 1, 2, 3 and exits 1, 3, 2, 0: each node is entered before its children and exited after all of them. The heap stack holds at most three tasks." src="docs/images/order-light.svg">
</picture>

A walk over several kinds of node (expressions that hold types, and types that
hold expressions) declares an enum for `Node` and one for `Frame`, with a
variant per kind, so each level carries a frame of its own kind.

## Bottom-up with an arena

When a result depends only on a node's children, no stack is needed. An
`Arena<T>` is one vector in which every node is stored after its children.
`Arena::push` returns a typed `Id<T>`, and a parent can only be built from ids
that already exist, so children always sit at lower positions. `Arena::bottom_up`
then computes one result per node in a single forward loop; each node reads its
children's finished results through `Results::get`.

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/images/arena-dark.svg">
  <img alt="An Arena stores each node after its children, so a node's children always have lower positions. Arena::bottom_up visits the nodes once, in order, and each node reads its children's finished results: no stack and no recursion." src="docs/images/arena-light.svg">
</picture>

Two properties keep the arena safe to share:

- `Clone`, `PartialEq`, `Debug` and drop all run over the flat vector, so none
  of them recurse in proportion to depth.
- Each arena has its own token and every `Id` carries it. `Arena::get`,
  `Results::get` and `Computed::get` return `None` for an id from another
  arena, a clone included, so a foreign id never reads some other node.

## Reference

| Item | What it is |
| --- | --- |
| `Walk` | The trait you implement: `Node`, `Frame`, `Stop`, `enter`, `exit`. |
| `walk(&mut walker, root)` | Runs a walk depth first; returns `ControlFlow<Stop>`. |
| `Children` | Passed to `enter`; `push` or `extend` to name children in visiting order. |
| `Arena<T>` | Children-first node storage: `push`, `get`, `len`, `is_empty`, `iter`, `bottom_up`. |
| `Id<T>` | Typed, arena-bound handle; `Copy`, `Eq`, `Ord`, `Hash` whatever `T` is. |
| `Results`, `Computed` | The results seen during `bottom_up`, and the finished set (`get`, `len`, `into_vec`). |

The crate has no dependencies beyond `core` and `alloc`, no cargo features, and
forbids `unsafe` code, so any `no_std` crate can depend on it without feature
unification pulling anything else in. The tests in [`tests/it`](tests/it) walk
100,000-deep chains and build 100,000-node arenas on a thread with a 512 KiB
stack, and check that the crate really has no features and no dependencies.
[`src/proofs.rs`](src/proofs.rs) holds bounded-model-checking harnesses for
[Kani](https://github.com/model-checking/kani).

## The Quire ecosystem

`quire-walk` is a foundation library: it sits at the bottom of the Quire
repositories and depends on no other. [quire-spec-language](https://github.com/agent-ix/quire-spec-language)
uses it for its deep traversals.

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/images/ecosystem-dark.svg">
  <img alt="The Quire repositories in four bands. Foundations: quire-exact, quire-semantic-value, quire-walk, quire-canonical, quire-verification-contracts, quire-observation. Language: quire-specification, quire-spec-language, quire-contract-ir. Backends: quire-contract-codegen, quire-contract-runtime, quire-analyze, quire-protocol, quire-mltl. Orchestration: quire-verification, quire-driver, quire-integration." src="docs/images/ecosystem-light.svg">
</picture>

## Building and contributing

The Makefile wraps the usual gates:

| Command | What it does |
| --- | --- |
| `make test` | `cargo test`, including the 100,000-deep stack-safety tests. |
| `make lint` | Clippy with warnings denied, also for a bare-metal target. |
| `make build-no-std` | Builds for `thumbv7em-none-eabi` to prove nothing from `std` is pulled in. |
| `make kani` | Runs the Kani proof harnesses; needs `cargo-kani`, not part of `ci`. |
| `make ci` | Format check, lint, tests, no-std build, `cargo deny` and docs. |

Run `make ci` locally before opening a pull request. Contributions need the
[CLA](CLA.md); see [CONTRIBUTING.md](CONTRIBUTING.md) and [AGENTS.md](AGENTS.md).

## License

[AGPL-3.0-or-later](LICENSE).
