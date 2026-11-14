// SPDX-License-Identifier: AGPL-3.0-or-later
//! The iterative traversal: [`Walk`], [`Children`] and [`walk`].

use alloc::vec::Vec;
use core::ops::ControlFlow;

use crate::reservation::{grow, Refusal, Reservation, Unlimited};

/// A top-down or mutually recursive walk, run by [`walk`] on an explicit
/// heap stack.
///
/// The walk declares its own node, frame and stop types. A walk over
/// several kinds of node (for example expressions that hold types, and
/// types that hold expressions) declares an enum for [`Walk::Node`] and one
/// for [`Walk::Frame`], with a variant per kind, so each level carries a
/// frame of its own kind.
pub trait Walk {
    /// The handle of one node to enter, usually a reference or an index.
    type Node;
    /// What the walk keeps for an entered node until it is exited.
    type Frame;
    /// The value a callback returns to end the walk early.
    type Stop;

    /// Enter `node`. Push the node's children onto `children`, in the
    /// order they are to be entered, and return the node's frame. The
    /// toolkit enters each child, and that child's whole subtree, before
    /// the next one, and passes the returned frame to [`Walk::exit`] after
    /// the last of them.
    ///
    /// Return [`ControlFlow::Break`] to end the walk: [`walk`] returns that
    /// value at once, and no callback runs after it. Children pushed before
    /// a break are discarded.
    fn enter(
        &mut self,
        node: Self::Node,
        children: &mut Children<'_, Self::Node>,
    ) -> ControlFlow<Self::Stop, Self::Frame>;

    /// Exit the node whose [`Walk::enter`] returned `frame`, once every
    /// child it pushed has been exited. Return [`ControlFlow::Break`] to end
    /// the walk as from [`Walk::enter`].
    fn exit(&mut self, frame: Self::Frame) -> ControlFlow<Self::Stop>;
}

/// The children [`Walk::enter`] names for the node it is entering.
///
/// The toolkit enters them in the order they are pushed. Under
/// [`walk_reserved`], naming a child that needs the pending list to grow
/// first asks the reservation; if it refuses, that child and every later
/// one in the same `enter` are dropped unnamed, and the walk ends with the
/// refusal once `enter` returns.
pub struct Children<'a, N> {
    pending: &'a mut Vec<N>,
    reservation: &'a mut dyn Reservation,
    /// The first refusal, which stops every later push.
    refusal: &'a mut Option<Refusal>,
}

impl<N> Children<'_, N> {
    /// Name `node` as the next child to enter.
    ///
    /// A refusal is recorded rather than returned; use [`Children::try_push`]
    /// or [`Children::refused`] to see it.
    pub fn push(&mut self, node: N) {
        // The refusal is already recorded for the walk to return.
        let _ = self.try_push(node);
    }

    /// Name `node` as the next child to enter, or return the refusal.
    ///
    /// The refusal is the one [`walk_reserved`] returns when this `enter`
    /// ends. Once any push is refused, every later push returns the same
    /// refusal and names nothing. An `enter` that does work of its own
    /// after naming its children uses this, or [`Children::refused`], to
    /// stop that work, because the walk cannot interrupt a running `enter`.
    ///
    /// # Errors
    ///
    /// The [`Refusal`] of this or an earlier push of the same `enter`.
    pub fn try_push(&mut self, node: N) -> Result<(), Refusal> {
        if let Some(refusal) = *self.refusal {
            return Err(refusal);
        }
        match grow(self.reservation, self.pending, 1) {
            Ok(()) => {
                self.pending.push(node);
                Ok(())
            }
            Err(refusal) => {
                *self.refusal = Some(refusal);
                Err(refusal)
            }
        }
    }

    /// Whether a push of this `enter` was refused, so that the walk will end
    /// with the refusal once `enter` returns.
    #[must_use]
    pub fn refused(&self) -> bool {
        self.refusal.is_some()
    }
}

impl<N> Extend<N> for Children<'_, N> {
    /// Name each node in turn, stopping at the first refusal without pulling
    /// another node from `nodes`. Each is a [`Children::try_push`] rather
    /// than a bulk reserve from the iterator's size hint, so the growth the
    /// reservation sees does not depend on the hint.
    fn extend<I: IntoIterator<Item = N>>(&mut self, nodes: I) {
        for node in nodes {
            if self.try_push(node).is_err() {
                break;
            }
        }
    }
}

/// One entry of the explicit stack: a node still to enter, or an entered
/// node's frame waiting for its exit.
enum Task<N, F> {
    Enter(N),
    Exit(F),
}

/// Run `walker` over the tree below `root`, depth first.
///
/// Every node is entered once, before any of its children, and exited
/// once, after all of them, with the frame its enter returned. Along every
/// root-to-leaf path the exits come in the reverse order of the enters.
/// The walk returns [`ControlFlow::Continue`] when every node has been
/// exited, or the first [`ControlFlow::Break`] a callback returns.
///
/// Native stack use is constant. The heap stack holds one frame per entered
/// node not yet exited and one entry per named child not yet entered, so its
/// size is bounded by the nodes the walk names.
///
/// This is [`walk_reserved`] with a reservation that never refuses, so the
/// two share one implementation.
///
/// # Panics
///
/// If the allocator cannot grow the heap stack.
pub fn walk<W: Walk + ?Sized>(walker: &mut W, root: W::Node) -> ControlFlow<W::Stop> {
    match walk_reserved(walker, root, &mut Unlimited) {
        Ok(flow) => flow,
        Err(refusal) => panic!("filament-walk: heap stack growth failed: {refusal:?}"),
    }
}

/// Run `walker` as [`walk`] does, asking `reservation` before each growth
/// of the walk's two heap lists.
///
/// The lists are the task stack (an entered node's frame plus the children
/// it named, together) and the pending list that [`Children`] fills. Before
/// a list grows, `reservation` is asked once for its full new capacity in
/// bytes (element size times new capacity), which is the exact amount
/// requested from the allocator; see [`Reservation`] for the policy. Growth
/// is by doubling from a minimum of four, and the calls come in the order
/// the growths happen.
///
/// The result is `Ok` with the walk's [`ControlFlow`] when no growth was
/// refused. When the reservation refuses (or the allocator does), the walk
/// returns that [`Refusal`] and does nothing further: no later callback
/// runs, the refused growth allocates nothing, and both lists are dropped.
/// A refusal found while `enter` runs takes precedence over the
/// [`ControlFlow`] that `enter` returns. The walk cannot interrupt a running
/// `enter`: it runs to completion, and an `enter` that charges or allocates
/// after naming children checks [`Children::refused`] (or uses
/// [`Children::try_push`]) to stop its own work.
///
/// With a reservation that admits exactly the total of a complete walk's
/// charges, the walk completes with the same visit order as [`walk`]; with
/// any smaller total, the walk is refused.
pub fn walk_reserved<W: Walk + ?Sized>(
    walker: &mut W,
    root: W::Node,
    reservation: &mut dyn Reservation,
) -> Result<ControlFlow<W::Stop>, Refusal> {
    let mut tasks: Vec<Task<W::Node, W::Frame>> = Vec::new();
    grow(reservation, &mut tasks, 1)?;
    tasks.push(Task::Enter(root));
    // Reused for every node: `enter` pushes its children here in entry
    // order. Popping them onto `tasks` reverses them, so the first child is
    // the next task popped.
    let mut pending: Vec<W::Node> = Vec::new();
    let mut refusal = None;
    while let Some(task) = tasks.pop() {
        match task {
            Task::Enter(node) => {
                let entered = walker.enter(
                    node,
                    &mut Children {
                        pending: &mut pending,
                        reservation: &mut *reservation,
                        refusal: &mut refusal,
                    },
                );
                if let Some(refusal) = refusal {
                    return Err(refusal);
                }
                let frame = match entered {
                    ControlFlow::Continue(frame) => frame,
                    ControlFlow::Break(stop) => return Ok(ControlFlow::Break(stop)),
                };
                // One growth for the frame and every named child, so a
                // refusal leaves `tasks` untouched.
                grow(reservation, &mut tasks, 1 + pending.len())?;
                tasks.push(Task::Exit(frame));
                while let Some(child) = pending.pop() {
                    tasks.push(Task::Enter(child));
                }
            }
            Task::Exit(frame) => {
                if let ControlFlow::Break(stop) = walker.exit(frame) {
                    return Ok(ControlFlow::Break(stop));
                }
            }
        }
    }
    Ok(ControlFlow::Continue(()))
}
