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
    ("ORCHESTRATION", [("quire-verification", True), ("quire-driver", True), ("quire-integration", True)]),
    ("BACKENDS", [("quire-contract-codegen", False), ("quire-contract-runtime", False),
                  ("quire-analyze", False), ("quire-protocol", True), ("quire-mltl", False)]),
    ("LANGUAGE", [("quire-specification", True), ("quire-spec-language", False), ("quire-contract-ir", False)]),
    ("FOUNDATIONS", [("quire-exact", False), ("quire-semantic-value", False), ("quire-walk", False),
                     ("quire-canonical", False), ("quire-verification-contracts", False),
                     ("quire-observation", False)]),
]


def ecosystem(here="quire-spec-language"):
    """The Quire repositories in bands; each band builds on the ones below."""
    f = Figure(
        "ecosystem",
        980,
        364,
        "The Quire repositories in four bands. Foundations: quire-exact, quire-semantic-value, "
        "quire-walk, quire-canonical, quire-verification-contracts, quire-observation. Language: "
        "quire-specification, quire-spec-language, quire-contract-ir. Backends: "
        "quire-contract-codegen, quire-contract-runtime, quire-analyze, quire-protocol, "
        "quire-mltl. Orchestration: quire-verification, quire-driver, quire-integration.",
    )
    y = 0
    gap = 10
    for band, repos in REPOS:
        f.text(0, y + 12, band, "muted", "mono", SMALL)
        widths = [FONTS["title"].width(name, TITLE) + 24 for name, _ in repos]
        spare = (980 - sum(widths) - gap * (len(repos) - 1)) / len(repos)
        x = 0
        for (name, private), w in zip(repos, widths):
            w = round(w + spare)
            role = "logic" if name == here else "ink"
            f.box(x, y + 20, w, 50, role, name, "private" if private else "public", dashed=private)
            x += w + gap
        y += 88
    f.legend(0, 360, [("logic", "this repository", "box"), ("ink", "private", "dash")])
    return f


def ecosystem_here():
    """The ecosystem figure with this repository highlighted."""
    return ecosystem(here="quire-walk")


FIGURES = [hero, order, arena, ecosystem_here]
