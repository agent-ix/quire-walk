// SPDX-License-Identifier: AGPL-3.0-or-later
//! TC-898 steps 7 to 9 (FR-356-AC-6): `walk_reserved` charges exactly the
//! bytes it requests from the allocator, admits an exact provision with
//! the visit order of `walk`, and refuses every smaller one without a
//! further visit or a leaked byte.
//!
//! This is its own test binary because it installs a counting global
//! allocator. The counters are per thread, so tests running in parallel do
//! not see each other's allocations.

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::ops::ControlFlow;

use filament_walk::{walk, walk_reserved, Children, Denied, Refusal, Reservation, Walk};
use ix_trace_rs::trace;

/// The most allocator requests one measured walk may make.
const MAX_REQUESTS: usize = 64;

thread_local! {
    /// The bytes each allocation or growth requested, in order.
    static REQUESTS: [Cell<usize>; MAX_REQUESTS] = const { [const { Cell::new(0) }; MAX_REQUESTS] };
    static REQUEST_COUNT: Cell<usize> = const { Cell::new(0) };
    /// Bytes allocated and not yet freed on this thread.
    static LIVE: Cell<isize> = const { Cell::new(0) };
}

struct Counting;

fn note_request(bytes: usize) {
    let count = REQUEST_COUNT.with(Cell::get);
    // Slots past the last are not kept; `measure` asserts none were lost.
    if count < MAX_REQUESTS {
        REQUESTS.with(|requests| requests[count].set(bytes));
    }
    REQUEST_COUNT.with(|n| n.set(count + 1));
}

// SAFETY: every method forwards to `System` unchanged and only updates
// const-initialised, destructor-free thread-local cells.
#[allow(unsafe_code)]
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        note_request(layout.size());
        LIVE.with(|live| live.set(live.get() + layout.size() as isize));
        System.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        LIVE.with(|live| live.set(live.get() - layout.size() as isize));
        System.dealloc(ptr, layout);
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        note_request(new_size);
        LIVE.with(|live| live.set(live.get() + new_size as isize - layout.size() as isize));
        System.realloc(ptr, layout, new_size)
    }
}

#[global_allocator]
static ALLOCATOR: Counting = Counting;

/// The allocator requests made while `run` executes, and the change in
/// live bytes across it.
fn measure<T>(run: impl FnOnce() -> T) -> (T, Vec<usize>, isize) {
    let live_before = LIVE.with(Cell::get);
    REQUEST_COUNT.with(|n| n.set(0));
    let result = run();
    let live_after = LIVE.with(Cell::get);
    let count_after = REQUEST_COUNT.with(Cell::get);
    assert!(count_after <= MAX_REQUESTS, "raise MAX_REQUESTS");
    let requests = REQUESTS.with(|r| r[..count_after].iter().map(Cell::get).collect());
    (result, requests, live_after - live_before)
}

/// A reservation that records every request and admits while a budget
/// lasts. The records are reserved up front so keeping them allocates
/// nothing during a measured walk.
struct Budget {
    remaining: usize,
    asked: Vec<usize>,
    /// How many visits had happened when a request was refused.
    visits: &'static std::thread::LocalKey<Cell<usize>>,
    visits_at_denial: Option<usize>,
}

impl Budget {
    fn new(remaining: usize) -> Self {
        Self {
            remaining,
            asked: Vec::with_capacity(MAX_REQUESTS),
            visits: &VISITS,
            visits_at_denial: None,
        }
    }
}

impl Reservation for Budget {
    fn reserve(&mut self, bytes: usize) -> Result<(), Denied> {
        if bytes > self.remaining {
            self.visits_at_denial = Some(self.visits.with(Cell::get));
            return Err(Denied);
        }
        self.remaining -= bytes;
        self.asked.push(bytes);
        Ok(())
    }
}

thread_local! {
    static VISITS: Cell<usize> = const { Cell::new(0) };
}

/// A tree of 41 nodes: node 0 has ten children (1 to 10) and each of them
/// three more (11 to 40), so the pending list grows past its minimum and
/// the task stack grows more than once.
fn children_of(node: usize) -> std::ops::Range<usize> {
    match node {
        0 => 1..11,
        1..=10 => 3 * node + 8..3 * node + 11,
        _ => 0..0,
    }
}

/// Records every enter and exit into lists reserved up front.
struct Recorder {
    order: Vec<(bool, usize)>,
}

impl Recorder {
    fn new() -> Self {
        Self {
            order: Vec::with_capacity(128),
        }
    }
}

impl Walk for Recorder {
    type Node = usize;
    type Frame = usize;
    type Stop = ();

    fn enter(&mut self, node: usize, children: &mut Children<'_, usize>) -> ControlFlow<(), usize> {
        VISITS.with(|v| v.set(v.get() + 1));
        self.order.push((true, node));
        children.extend(children_of(node));
        ControlFlow::Continue(node)
    }

    fn exit(&mut self, frame: usize) -> ControlFlow<()> {
        VISITS.with(|v| v.set(v.get() + 1));
        self.order.push((false, frame));
        ControlFlow::Continue(())
    }
}

/// The visit order of `walk` and the charges of a complete reserved walk.
fn reference() -> (Vec<(bool, usize)>, Vec<usize>) {
    let mut plain = Recorder::new();
    assert_eq!(walk(&mut plain, 0), ControlFlow::Continue(()));
    let mut budget = Budget::new(usize::MAX);
    let mut reserved = Recorder::new();
    VISITS.with(|v| v.set(0));
    let _ = walk_reserved(&mut reserved, 0, &mut budget).expect("an unlimited budget admits");
    assert_eq!(reserved.order, plain.order);
    (plain.order, budget.asked)
}

#[trace("TC-898", "FR-356-AC-6")]
#[test]
fn charged_bytes_are_the_bytes_requested_from_the_allocator() {
    let (_, charges) = reference();
    let mut budget = Budget::new(usize::MAX);
    let mut recorder = Recorder::new();
    let (result, requests, _) = measure(|| walk_reserved(&mut recorder, 0, &mut budget));
    assert_eq!(result, Ok(ControlFlow::Continue(())));
    assert_eq!(budget.asked, charges);
    // Every charge is one allocator request of the same size, in order.
    assert_eq!(requests, budget.asked);
    // Both lists grew past one allocation, by doubling from four.
    assert!(budget.asked.len() >= 4, "{:?}", budget.asked);
}

#[trace("TC-898", "FR-356-AC-6")]
#[test]
fn an_exact_provision_admits_with_the_visit_order_of_walk() {
    let (order, charges) = reference();
    let total = charges.iter().sum();
    let mut budget = Budget::new(total);
    let mut recorder = Recorder::new();
    let result = walk_reserved(&mut recorder, 0, &mut budget);
    assert_eq!(result, Ok(ControlFlow::Continue(())));
    assert_eq!(recorder.order, order);
    assert_eq!(budget.remaining, 0);
}

#[trace("TC-898", "FR-356-AC-6")]
#[test]
fn every_smaller_provision_refuses_with_no_further_visit_and_no_heap_left() {
    let (order, charges) = reference();
    let total: usize = charges.iter().sum();
    for provision in 0..total {
        let mut budget = Budget::new(provision);
        let mut recorder = Recorder::new();
        VISITS.with(|v| v.set(0));
        let (result, requests, live) = measure(|| walk_reserved(&mut recorder, 0, &mut budget));
        let Err(Refusal::Denied { requested }) = result else {
            panic!("provision {provision} of {total} must be refused, got {result:?}");
        };
        let denied_at = budget
            .visits_at_denial
            .expect("the reservation saw the denial");
        // The refused growth allocated nothing: the allocator saw exactly
        // the admitted charges.
        assert_eq!(requests, budget.asked, "provision {provision}");
        assert!(requested > budget.remaining);
        // No callback ran after the denial, and what ran was a prefix of
        // the full order.
        assert_eq!(recorder.order.len(), denied_at, "provision {provision}");
        assert_eq!(recorder.order, order[..denied_at], "provision {provision}");
        // Both lists were dropped: no live byte remains from the walk.
        assert_eq!(live, 0, "provision {provision}");
    }
}

/// A chain `0 -> 1 -> ... -> len - 1` that records nothing, so its heap use
/// is the walk's alone.
struct Chain {
    len: usize,
    entered: usize,
    exited: usize,
}

impl Walk for Chain {
    type Node = usize;
    type Frame = ();
    type Stop = ();

    fn enter(&mut self, node: usize, children: &mut Children<'_, usize>) -> ControlFlow<(), ()> {
        self.entered += 1;
        if node + 1 < self.len {
            children.push(node + 1);
        }
        ControlFlow::Continue(())
    }

    fn exit(&mut self, _frame: ()) -> ControlFlow<()> {
        self.exited += 1;
        ControlFlow::Continue(())
    }
}

#[trace("TC-898", "FR-356-AC-6")]
#[test]
fn a_deep_input_walks_and_refuses_on_a_small_stack() {
    const DEPTH: usize = 100_000;
    std::thread::Builder::new()
        .stack_size(512 * 1024)
        .spawn(|| {
            let mut full = Budget::new(usize::MAX);
            let mut chain = Chain {
                len: DEPTH,
                entered: 0,
                exited: 0,
            };
            let (result, requests, live) = measure(|| walk_reserved(&mut chain, 0, &mut full));
            assert_eq!(result, Ok(ControlFlow::Continue(())));
            assert_eq!((chain.entered, chain.exited), (DEPTH, DEPTH));
            assert_eq!(requests, full.asked);
            assert_eq!(live, 0);

            // The exact total completes with nothing left over.
            let total: usize = full.asked.iter().sum();
            let mut exact = Budget::new(total);
            let mut chain = Chain {
                len: DEPTH,
                entered: 0,
                exited: 0,
            };
            assert_eq!(
                walk_reserved(&mut chain, 0, &mut exact),
                Ok(ControlFlow::Continue(()))
            );
            assert_eq!(exact.remaining, 0);

            // One byte short of the whole walk refuses at its last growth.
            let mut short = Budget::new(total - 1);
            let mut chain = Chain {
                len: DEPTH,
                entered: 0,
                exited: 0,
            };
            let (result, _, live) = measure(|| walk_reserved(&mut chain, 0, &mut short));
            assert!(matches!(result, Err(Refusal::Denied { .. })));
            assert_eq!(chain.exited, 0, "the refusal came before any exit");
            assert_eq!(live, 0);
        })
        .expect("spawn a 512 KiB thread")
        .join()
        .expect("the reserved walk must not overflow a 512 KiB stack");
}

/// Draws from a pool its walker also draws from.
struct Pool<'p>(&'p Cell<usize>);

impl Reservation for Pool<'_> {
    fn reserve(&mut self, bytes: usize) -> Result<(), Denied> {
        let left = self.0.get().checked_sub(bytes).ok_or(Denied)?;
        self.0.set(left);
        Ok(())
    }
}

/// Names ten children, then charges `WORK` of its own from the pool unless
/// the push was refused.
struct Worker<'p> {
    pool: &'p Cell<usize>,
    pulled: usize,
    worked: usize,
}

const WORK: usize = 1;

impl Walk for Worker<'_> {
    type Node = usize;
    type Frame = ();
    type Stop = ();

    fn enter(&mut self, node: usize, children: &mut Children<'_, usize>) -> ControlFlow<(), ()> {
        if node == 0 {
            let pulled = &mut self.pulled;
            children.extend((1..=10).inspect(|_| *pulled += 1));
            if !children.refused() {
                self.pool.set(self.pool.get() - WORK);
                self.worked += 1;
            }
        }
        ControlFlow::Continue(())
    }

    fn exit(&mut self, _frame: ()) -> ControlFlow<()> {
        ControlFlow::Continue(())
    }
}

#[trace("TC-898", "FR-356-AC-6")]
#[test]
fn an_enter_sees_the_refusal_and_stops_its_own_charge() {
    let word = core::mem::size_of::<usize>();
    // The task stack takes its first growth (4 tasks), then the pending
    // list grows to 4 and 8; the third growth (16, at the ninth child) is
    // refused while `WORK` bytes remain.
    let task = core::mem::size_of::<(usize, usize)>();
    let pool = Cell::new(4 * task + 4 * word + 8 * word + WORK);
    let mut worker = Worker {
        pool: &pool,
        pulled: 0,
        worked: 0,
    };
    let result = walk_reserved(&mut worker, 0, &mut Pool(&pool));
    assert_eq!(
        result,
        Err(Refusal::Denied {
            requested: 16 * word
        })
    );
    assert_eq!(
        worker.worked, 0,
        "the enter made no charge after the refusal"
    );
    assert_eq!(pool.get(), WORK, "the pool was not drawn on");
    assert_eq!(worker.pulled, 9, "extend pulled no node after the refusal");
}
