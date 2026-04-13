use core::{
    alloc::GlobalAlloc,
    sync::atomic::{AtomicPtr, AtomicUsize},
};

use alloc::alloc::{Allocator, Global};

#[derive(Debug)]
pub struct ArenaSync<A: Allocator> {
    inner: A,
    buf: AtomicPtr<u8>,
    used: AtomicUsize,
    size: AtomicUsize,
}

impl<A> ArenaSync<A>
where
    A: Allocator,
{
    pub fn new_in(alloc: A, init_size: usize) -> Self {
        todo!()
    }
}

impl ArenaSync<Global> {
    #[inline]
    pub fn new(init_size: usize) -> Self {
        Self::new_in(Global, init_size)
    }
}

unsafe impl<A> core::marker::Sync for ArenaSync<A> where A: Allocator {}

unsafe impl<A> GlobalAlloc for ArenaSync<A>
where
    A: Allocator,
{
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        todo!()
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        todo!()
    }
}

unsafe impl<A> Allocator for ArenaSync<A>
where
    A: Allocator,
{
    fn allocate(
        &self,
        layout: core::alloc::Layout,
    ) -> Result<core::ptr::NonNull<[u8]>, alloc::alloc::AllocError> {
        todo!()
    }

    unsafe fn deallocate(&self, ptr: core::ptr::NonNull<u8>, layout: core::alloc::Layout) {
        todo!()
    }
}
