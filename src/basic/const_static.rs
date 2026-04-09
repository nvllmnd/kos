use core::{
    alloc::{GlobalAlloc, Layout},
    cell::{Cell, UnsafeCell},
    ptr::NonNull,
};

use alloc::alloc::Allocator;

#[repr(C, align(4096))]
#[derive(Debug)]
pub struct ArenaStatic<const SIZE: usize = 4096> {
    storage: UnsafeCell<[u8; SIZE]>,
    used: Cell<u32>,
}
impl<const S: usize> ArenaStatic<S> {
    pub const fn new() -> Self {
        Self {
            storage: UnsafeCell::new([0; S]),
            used: Cell::new(0),
        }
    }

    pub fn allocate_raw(&self, layout: core::alloc::Layout) -> *mut u8 {
        if self.is_full() || !self.has_space_for(layout) {
            return core::ptr::null_mut();
        }

        let begin =
            NonNull::new(self.storage.get()).expect("Storage pointer should always be non-null!");
        let used = self.used.get();

        let next_ptr = unsafe { begin.byte_offset(used as isize) };
        let offset = next_ptr.align_offset(layout.align());
        let next_ptr = unsafe { next_ptr.add(offset) };

        let o = offset as u32;
        let s = layout.size() as u32;

        self.used.set(used + o + s);
        next_ptr.as_ptr() as *mut _
    }

    pub const fn has_space_for(&self, layout: Layout) -> bool {
        let used = self.used.get() as usize;
        let size = layout.size();
        used + size < S
    }

    pub const fn is_full(&self) -> bool {
        self.used.get() > S as u32
    }

    pub const fn has_space(&self) -> bool {
        !self.is_full()
    }

    /// # Safety
    ///
    /// this call invalidates all pointers currently used for any allocation that might still be
    /// alive! this call simply resets the field used to track our bump-style allocations back to 0
    /// Be sure that you dont refer to anything that was allocated so far after this method is
    /// called!
    ///
    #[inline]
    pub unsafe fn reset(&self) {
        self.used.set(0)
    }

    #[inline]
    pub fn clear(self) -> Self {
        drop(self);
        Self::new()
    }
}

impl<const S: usize> Default for ArenaStatic<S> {
    fn default() -> Self {
        Self::new()
    }
}

unsafe impl GlobalAlloc for ArenaStatic {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        self.allocate_raw(layout)
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: core::alloc::Layout) {}
}

unsafe impl<const S: usize> Allocator for ArenaStatic<S> {
    fn allocate(
        &self,
        layout: core::alloc::Layout,
    ) -> Result<NonNull<[u8]>, alloc::alloc::AllocError> {
        let Some(ptr) = NonNull::new(self.allocate_raw(layout)) else {
            // only get here if this arena is out of available memory, or if this next
            // allocation does not fit in our avaialable free space.
            return Result::Err(alloc::alloc::AllocError);
        };
        // allocate_raw checks only returns a non-null pointer that is valid, so this slice is
        // safe and legit
        let ptr = NonNull::slice_from_raw_parts(ptr, layout.size());

        Result::Ok(ptr)
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: core::alloc::Layout) {}
}

impl<const S: usize> Clone for ArenaStatic<S> {
    fn clone(&self) -> Self {
        let storage = UnsafeCell::new([0; S]);
        let src = self.storage.get();
        let dst = storage.get();
        unsafe { core::ptr::copy_nonoverlapping(src, dst, S) };
        Self {
            storage,
            used: self.used.clone(),
        }
    }
}
