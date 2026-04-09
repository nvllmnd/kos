use core::{
    alloc::{GlobalAlloc, Layout},
    cell::UnsafeCell,
    ops::{Deref, Index},
    ptr::NonNull,
    sync::atomic::AtomicUsize,
};

use alloc::alloc::Allocator;

use crate::{
    basic::slot::{AllocSlot, SlotPtr, SlottedAllocator},
    clone_slice_into,
};

#[repr(C)]
#[derive(Debug)]
pub struct ArenaStaticSync<const SIZE: usize> {
    mem: UnsafeCell<[u8; SIZE]>,
    used: AtomicUsize,
}

impl<const S: usize> ArenaStaticSync<S> {
    pub const fn new() -> Self {
        Self {
            mem: UnsafeCell::new([0u8; S]),
            used: AtomicUsize::new(0),
        }
    }

    pub fn allocate_raw(&self, layout: core::alloc::Layout) -> *mut u8 {
        let used = self.used();
        // .used
        // .fetch_add(layout.size(), core::sync::atomic::Ordering::Relaxed);
        let mem = self.mem.get().cast::<u8>();
        let alloc_ptr = unsafe { mem.add(used) };
        let offset = mem.align_offset(layout.align());
        let res = unsafe { alloc_ptr.add(offset) };
        self.used_inc(offset + layout.size());
        res
    }

    pub const fn as_ptr(&self) -> NonNull<u8> {
        self.mem().cast::<u8>()
    }

    pub fn ptr_slice(&self) -> NonNull<[u8]> {
        let p = self.as_ptr();
        NonNull::slice_from_raw_parts(p, S)
    }

    pub const fn mem(&self) -> NonNull<[u8; S]> {
        unsafe { NonNull::new_unchecked(self.mem.get()) }
    }

    /// Simply sets our atomic used counter to 0 and does nothing else
    #[inline]
    pub unsafe fn clear(&self) {
        self.set_used(0);
    }

    /// Sets used counter back to 0 and sets all bytes of [Self::mem] to 0
    pub unsafe fn clear_zeroed(&self) {
        unsafe { self.clear() };

        {
            let mem = self.mem.get();
            let m = unsafe { mem.as_mut().unwrap() };
            m.fill(0);
        }
    }
    // pub fn lookup_tagged_ptr<T>(&self, ptr: TaggedPtr<T>) -> Option<NonNull<T>> {
    //     let p = self.lookup_ptr(ptr.tag())?;
    //     let p = p.cast::<T>();
    //     Some(p)
    // }

    // /// This method looks up the current pointer at the byte offset index in [crate::basic::const_sync::TaggedPtr]
    // /// and then compares its address to the [core::ptr::NonNull] ptr in [crate::basic::const_sync::TaggedPtr]
    // /// to verify that the tagged pointer is pointing to where we are expecting it to be pointing to.
    // ///
    // /// If for some reason this method ever returns false, then the [crate::basic::const_sync::TaggedPtr] passed in as a parameter
    // /// is most likely pointing to old memory that has since been [crate::basic::const_sync::ArenaStaticSync::clear]/[crate::basic::const_sync::ArenaStaticSync::clear_zeroed]jjjj
    // pub fn tag_is_valid<T>(&self, ptr: TaggedPtr<T>) -> bool {
    //     let Some(curr_ptr) = self.lookup_ptr(ptr.tag()) else {
    //         return false;
    //     };

    //     let tagged_ptr = ptr.val.cast::<u8>();
    //     curr_ptr.addr() == tagged_ptr.addr()
    // }

    #[inline]
    /// Same as [Self::used_fetch_add], but discards the return value
    pub fn used_inc(&self, n: usize) {
        let _ = self.used_fetch_add(n);
    }

    #[inline]
    /// gets current used value by calling [AtomicUsize::load] with [core::sync::atomic::Ordering::Relaxed]
    pub fn used(&self) -> usize {
        self.used.load(core::sync::atomic::Ordering::Relaxed)
    }

    #[inline]
    /// sets the current used field value to @param(new_value).
    /// Simply wraps a call to [AtomicUsize::store], passing in @param(new_value) with
    /// [core::sync::atomic::Ordering::Relaxed] ordering
    pub fn set_used(&self, new_value: usize) {
        self.used
            .store(new_value, core::sync::atomic::Ordering::Relaxed);
    }

    #[inline]
    /// adds @param(n) to used field and returns the previous old value
    /// Simply wraps [AtomicUsize::fetch_add]
    pub fn used_fetch_add(&self, n: usize) -> usize {
        self.used
            .fetch_add(n, core::sync::atomic::Ordering::Relaxed)
    }
}

unsafe impl<const S: usize> core::marker::Sync for ArenaStaticSync<S> {}

unsafe impl<const S: usize> Allocator for ArenaStaticSync<S> {
    fn allocate(
        &self,
        layout: core::alloc::Layout,
    ) -> Result<core::ptr::NonNull<[u8]>, alloc::alloc::AllocError> {
        let p = self.allocate_raw(layout);

        let Some(ptr) = NonNull::new(p) else {
            return Err(alloc::alloc::AllocError);
        };

        let sl = NonNull::slice_from_raw_parts(ptr, layout.size());
        Ok(sl)
    }

    unsafe fn deallocate(&self, ptr: core::ptr::NonNull<u8>, layout: core::alloc::Layout) {}
}

unsafe impl<const S: usize> GlobalAlloc for ArenaStaticSync<S> {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        self.allocate_raw(layout)
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: core::alloc::Layout) {}
}

impl<const S: usize> Default for ArenaStaticSync<S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const S: usize> SlottedAllocator for ArenaStaticSync<S> {
    fn lookup(&self, tag: AllocSlot) -> Option<NonNull<u8>> {
        self.lookup_ptr(tag)
    }
}

impl<const S: usize> SlottedAllocator for &ArenaStaticSync<S> {
    fn lookup(&self, tag: AllocSlot) -> Option<NonNull<u8>> {
        self.lookup_ptr(tag)
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;

    use super::*;

    #[global_allocator]
    static G: ArenaStaticSync<255> = ArenaStaticSync::<255>::new();

    #[test]
    fn arena_works_as_global_allocator() {
        let mut v = Vec::new();
        v.push(100);

        assert_eq!(v[0], 100);
    }
}
