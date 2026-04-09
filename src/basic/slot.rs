use core::{alloc::Layout, ptr::NonNull};

use alloc::alloc::Allocator;

use crate::basic::const_sync::ArenaStaticSync;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct AllocSlot(u32);

/// pointer to an allocation made by [ArenaStaticSync], that is tagged
/// with the byte offset index to where the pointer SHOULD be pointing to.
/// you can use this to verify an allocation is currently valid and not dangling from an
/// inproper resetting of the Arena
pub trait SlottedAllocator: Allocator {
    fn lookup(&self, tag: AllocSlot) -> Option<NonNull<u8>>;
}

pub struct SlotPtr<A: SlottedAllocator> {
    slot: AllocSlot,
    parent: A,
}

impl<A> SlotPtr<A>
where
    A: SlottedAllocator,
{
    pub const fn new(slot: usize, parent: A) -> Self {
        Self {
            slot: AllocSlot(slot as u32),
            parent,
        }
    }

    #[inline]
    pub fn try_get(&self) -> Option<NonNull<u8>> {
        self.parent.lookup(self.slot)
    }

    #[inline]
    pub fn get(&self) -> NonNull<u8> {
        self.try_get().expect(
            "Error occured while looking up a slot index. Slot Index most likely out of range!",
        )
    }

    #[inline]
    pub fn try_get_cast<T>(&self) -> Option<NonNull<T>> {
        self.try_get().map(|x| x.cast::<T>())
    }

    #[inline]
    pub fn get_cast<T>(&self) -> NonNull<T> {
        self.try_get_cast()
            .expect("Error occured while looking up an allocation pointer with an AllocSlot")
    }

    pub const fn slot(&self) -> AllocSlot {
        self.slot
    }

    pub const fn allocator(&self) -> &A {
        &self.parent
    }
}

// Need this here so we can use the inner field of [AllocSlot]
impl<const S: usize> ArenaStaticSync<S> {
    pub fn lookup_ptr(&self, tag: AllocSlot) -> Option<NonNull<u8>> {
        let i = tag.0 as usize;
        if i >= S {
            return None;
        }
        let mem = self.as_ptr();
        let res = unsafe { mem.add(i) };
        Some(res)
    }

    pub fn allocate_slotted(&self, layout: Layout) -> SlotPtr<&Self> {
        let slot = self.used();
        let _ = self.allocate_raw(layout);
        let sl = SlotPtr {
            slot: AllocSlot(slot as u32),
            parent: self.by_ref(),
        };
        sl
    }
}
