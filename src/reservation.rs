// SPDX-License-Identifier: AGPL-3.0-or-later
//! A caller's say over the heap a walk grows: [`Reservation`],
//! [`Denied`] and [`Refusal`].

use alloc::vec::Vec;

/// The smallest capacity a walk-owned list grows to.
const MIN_CAPACITY: usize = 4;

/// A caller's decision on heap growth, asked before each growth.
///
/// Every growth of a list that [`crate::walk_reserved`] owns calls
/// [`Reservation::reserve`] once with the full new capacity in bytes
/// (element size times new capacity). The list then grows to exactly that
/// capacity with `try_reserve_exact`, so the bytes asked are the bytes
/// requested from the allocator. A growth that moves holds the old and the
/// new block at once, and the full-capacity charge covers that moment;
/// nothing is credited back when the old block goes.
///
/// This is the same shape, unit and growth policy (doubling from a minimum
/// of four, exact reservation) as `quire-canonical`'s `Reservation`, so one
/// caller type can implement both. It is declared here rather than imported
/// because `filament-walk` is a shared leaf with no dependency.
pub trait Reservation {
    /// Admit or refuse `bytes` of heap growth.
    ///
    /// # Errors
    ///
    /// [`Denied`] to refuse. Whatever else the caller needs to know about
    /// the denial (which budget, what remained) stays with the caller.
    fn reserve(&mut self, bytes: usize) -> Result<(), Denied>;
}

/// A [`Reservation`] refused a growth.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Denied;

/// Why a walk stopped growing its heap lists.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Refusal {
    /// The caller's [`Reservation`] refused `requested` bytes.
    Denied {
        /// The bytes the reservation was asked for.
        requested: usize,
    },
    /// The allocator could not provide `requested` bytes.
    Allocation {
        /// The bytes the growth asked the allocator for.
        requested: usize,
    },
}

/// The reservation [`crate::walk`] uses: it admits every growth.
pub(crate) struct Unlimited;

impl Reservation for Unlimited {
    fn reserve(&mut self, _bytes: usize) -> Result<(), Denied> {
        Ok(())
    }
}

/// Make room in `list` for `additional` more items, asking `reservation`
/// first. A list that already has room is not grown and nothing is asked.
pub(crate) fn grow<T>(
    reservation: &mut dyn Reservation,
    list: &mut Vec<T>,
    additional: usize,
) -> Result<(), Refusal> {
    let size = core::mem::size_of::<T>();
    let Some(needed) = list.len().checked_add(additional) else {
        return Err(Refusal::Allocation {
            requested: size.saturating_mul(additional),
        });
    };
    let capacity = list.capacity();
    if needed <= capacity {
        return Ok(());
    }
    let target = needed.max(capacity.saturating_mul(2)).max(MIN_CAPACITY);
    let bytes = size.saturating_mul(target);
    if reservation.reserve(bytes).is_err() {
        return Err(Refusal::Denied { requested: bytes });
    }
    list.try_reserve_exact(target - list.len())
        .map_err(|_| Refusal::Allocation { requested: bytes })
}
