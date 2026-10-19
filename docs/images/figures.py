# SPDX-License-Identifier: AGPL-3.0-or-later
# Copyright (C) 2026 Agent-IX
"""quire-walk's documentation figures, rendered by the docs-figures skill.

Regenerate from the repository root with the agent-ix/dev-tools skill:

    python3 -m pip install fonttools
    python3 <dev-tools>/skills/docs-figures/scripts/render.py docs/images/figures.py

Add --check to confirm the committed SVGs are current. The step sequence and
stack sizes in the walk figure are the trace of the example in src/lib.rs.
"""

from docs_figures import FONTS, SMALL, TITLE, Figure  # noqa: F401

STYLE = "ix-docs"

# One role per kind of thing, fixed across the Quire repositories.
ROLES = [
    ("code", "authored source", "box"),
    ("logic", "compiler stage", "box"),
    ("model", "checked package / IR", "box"),
    ("evidence", "outcome", "box"),
    ("ink", "input", "box"),
]


# ---- figures -----------------------------------------------------------------


def hero():
    """README hero: native recursion overflows, the heap stack does not."""
    f = Figure(
        "hero",
        980,
        330,
        "Native recursion keeps one call frame per level of the input, so a 100,000-level tree "
        "overflows a 512 KiB stack. quire-walk keeps one typed frame per level on a heap stack "
        "and runs enter and exit callbacks from a loop, so native stack use stays constant.",
    )
    f.text(0, 12, "RECURSION · ONE CALL FRAME PER LEVEL", "muted", "mono", SMALL)
    f.text(540, 12, "WALK · ONE TYPED FRAME PER LEVEL, ON THE HEAP", "muted", "mono", SMALL)
    for i, depth in enumerate(["level 1", "level 2", "level 3"]):
        f.box(0, 28 + i * 58, 250, 50, "ink", "visit(node)", depth + " · on the call stack")
    f.text(125, 214, "…  100,000 levels", "muted", "mono", SMALL, anchor="middle")
    f.box(0, 226, 250, 50, "evidence", "stack overflow", "512 KiB thread")
    f.line([(250, 251), (290, 251)], "evidence", weight=2.4)
    f.text(298, 255, "the process aborts", "evidence", "mono", SMALL + 1)

    f.box(540, 28, 200, 62, "code", "your Walk impl", "enter(node, children)", "exit(frame)")
    f.line([(740, 59), (790, 59)], "code")
    f.box(793, 28, 187, 62, "logic", "walk(&mut w, root)", "a loop, no recursion")
    f.text(540, 124, "HEAP STACK OF TASKS", "muted", "mono", SMALL)
    f.box(540, 134, 120, 50, "model", "Enter(child)", "still to visit")
    f.box(672, 134, 120, 50, "model", "Enter(child)", "still to visit")
    f.box(804, 134, 176, 50, "model", "Exit(frame)", "your frame, kept")
    f.line([(886, 90), (886, 131)], "logic")
    f.box(540, 226, 440, 50, "logic", "native stack use: constant", "same at depth 10 or 100,000")
    f.line([(760, 184), (760, 223)], "logic", arrow=False)
    f.legend(0, 324, [ROLES[0], ROLES[1], ROLES[2], ROLES[3], ("ink", "native call stack", "box")])
    return f


def order():
    """The enter and exit order of the lib.rs example, with the stack size."""
    steps = [
        ("enter 0", "push 1, 2", 3),
        ("enter 1", "leaf", 3),
        ("exit 1", "frame back", 2),
        ("enter 2", "push 3", 3),
        ("enter 3", "leaf", 3),
        ("exit 3", "frame back", 2),
        ("exit 2", "frame back", 1),
        ("exit 0", "frame back", 0),
    ]
    f = Figure(
        "order",
        980,
        330,
        "A walk of the tree where node 0 has children 1 and 2 and node 2 has child 3 enters "
        "0, 1, 2, 3 and exits 1, 3, 2, 0: each node is entered before its children and exited "
        "after all of them. The heap stack holds at most three tasks.",
    )
    f.text(0, 12, "TREE: 0 → (1, 2) · 2 → (3)", "muted", "mono", SMALL)
    w, gap = 110, 12
    for i, (title, sub, _) in enumerate(steps):
        x = i * (w + gap)
        role = "logic" if title.startswith("enter") else "model"
        f.box(x, 28, w, 56, role, title, sub)
    f.text(0, 124, "TASKS ON THE HEAP STACK AFTER EACH STEP", "muted", "mono", SMALL)
    f.columns(
        0,
        134,
        980,
        180,
        [n for _, _, n in steps],
        labels=[t for t, _, _ in steps],
        ticks=[0, 1, 2, 3],
        role=["logic" if t.startswith("enter") else "model" for t, _, _ in steps],
    )
    return f


def arena():
    """An arena stores each node after its children; one forward loop computes."""
    nodes = [
        ("0", "leaf"),
        ("1", "leaf"),
        ("2", "add(0, 1)"),
        ("3", "leaf"),
        ("4", "add(2, 3)"),
    ]
    f = Figure(
        "arena",
        980,
        236,
        "An Arena stores each node after its children, so a node's children always have lower "
        "positions. Arena::bottom_up visits the nodes once, in order, and each node reads its "
        "children's finished results: no stack and no recursion.",
    )
    w = 150
    gap = (980 - 5 * w) / 4
    cx = lambda i: round(i * (w + gap) + w / 2)  # noqa: E731
    f.text(0, 12, "STORED IN PUSH ORDER · CHILDREN BEFORE PARENTS", "muted", "mono", SMALL)
    for i, (name, what) in enumerate(nodes):
        f.box(round(i * (w + gap)), 74, w, 56, "model", "node " + name, what)
    # The farther child runs higher and lands further right, so no edge crosses another.
    for parent, kids in [(2, (0, 1)), (4, (2, 3))]:
        for n, kid in enumerate(kids):
            y = 40 + n * 10
            start = cx(kid) + (30 if kid == 2 else 0)
            into = cx(parent) + (10 if n == 0 else -10)
            f.line([(start, 71), (start, y), (into, y), (into, 71)], "faint", 1.5)
    f.text(0, 170, "ONE FORWARD LOOP · bottom_up(compute)", "muted", "mono", SMALL)
    f.box(0, 180, 980, 50, "logic", "result[i] = compute(node i, results 0 … i-1)",
          "no stack: every child's result is already there")
    return f


REPOS = [
    (1, 0, "quire-driver", "the quire command", True),
    (2, 0, "quire-integration", "tests the whole chain", True),
    (3, 0, "quire-verification", "plans verification", True),
    (0, 1, "quire-specification", "the standard", True),
    (1, 1, "quire-spec-language", "compiler · evaluator", False),
    (2, 1, "quire-contract-ir", "contract model", False),
    (3, 1, "quire-contract-codegen", "code · tests · proofs", False),
    (4, 1, "quire-contract-runtime", "runs generated checks", False),
    (0, 2, "quire-observation", "qualified observations", False),
    (1, 2, "quire-protocol", "protocol conformance", True),
    (2, 2, "quire-analyze", "SMT consistency", False),
    (0, 3, "quire-mltl", "MLTL over observations", False),
]
FEEDS = [
    ("quire-specification", "quire-spec-language"),
    ("quire-spec-language", "quire-contract-ir"),
    ("quire-contract-ir", "quire-contract-codegen"),
    ("quire-contract-codegen", "quire-contract-runtime"),
    ("quire-spec-language", "quire-protocol"),
    ("quire-contract-ir", "quire-analyze"),
    ("quire-observation", "quire-protocol"),
    ("quire-observation", "quire-mltl"),
]
SHARED = ["quire-exact", "quire-semantic-value", "quire-walk", "quire-canonical",
          "quire-verification-contracts"]


def ecosystem(here="quire-spec-language"):
    """The Quire repositories: the main chain, its branches, and what runs and shares it."""
    f = Figure(
        "ecosystem",
        980,
        476,
        "The Quire repositories. The main chain runs from the standard, quire-specification, to "
        "the compiler, quire-spec-language, then quire-contract-ir, quire-contract-codegen and "
        "quire-contract-runtime. quire-spec-language also feeds quire-protocol, and "
        "quire-contract-ir feeds quire-analyze. quire-observation feeds quire-protocol and "
        "quire-mltl. Above the chain, quire-driver runs it, quire-integration tests it and "
        "quire-verification plans verification. Below it, shared libraries used throughout: "
        "quire-exact, quire-semantic-value, quire-walk, quire-canonical and "
        "quire-verification-contracts.",
    )
    w, h, step = 176, 50, 201
    rows = {0: 20, 1: 120, 2: 210, 3: 290}
    where = {}
    for col, row, name, sub, private in REPOS:
        x, y = col * step, rows[row]
        where[name] = (x, y)
        role = "logic" if name == here else "ink"
        f.box(x, y, w, h, role, name, sub, dashed=private)
    for a, b in FEEDS:
        (ax, ay), (bx, by) = where[a], where[b]
        if ay == by:
            f.line([(ax + w, ay + h / 2), (bx - 3, by + h / 2)], "model")
        else:
            f.line([(ax + w / 2, ay + h), (bx + w / 2, by - 3)], "model")
    f.text(0, 12, "ORCHESTRATION", "muted", "mono", SMALL)
    f.text(0, 112, "THE CHAIN", "muted", "mono", SMALL)
    f.text(0, 382, "SHARED LIBRARIES · USED THROUGHOUT", "muted", "mono", SMALL)
    gap = 10
    widths = [FONTS["title"].width(name, TITLE) + 24 for name in SHARED]
    spare = (980 - sum(widths) - gap * (len(SHARED) - 1)) / len(SHARED)
    x = 0
    for name, w0 in zip(SHARED, widths):
        w0 = round(w0 + spare)
        f.box(x, 390, w0, h, "logic" if name == here else "ink", name)
        x += w0 + gap
    f.legend(0, 472, [("logic", "this repository", "box"), ("ink", "private", "dash"),
                      ("model", "feeds", "fill")])
    return f


def ecosystem_here():
    """The ecosystem figure with this repository highlighted."""
    return ecosystem(here="quire-walk")


FIGURES = [hero, order, arena, ecosystem_here]
