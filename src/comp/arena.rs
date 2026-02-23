use core::{
    alloc::{GlobalAlloc, Layout},
    cell::Cell,
    error::Error,
    fmt::Display,
    marker::PhantomData,
    ops::Deref,
    ptr::NonNull,
};

use alloc::alloc::Allocator;
use anyhow::bail;

use crate::basic::const_static::ArenaStatic;

#[derive(Debug, Clone)]
pub struct Arena<A: Allocator> {
    inner: A,
    used: Cell<usize>,
    size: Cell<usize>,
    buf: NonNull<u8>,
}

impl<A: Allocator> Arena<A> {
    pub fn new(parent: A, cap: usize) -> anyhow::Result<Self> {
        let layout = Layout::array::<u8>(cap).unwrap();
        let Ok(buf) = parent.allocate(layout) else {
            bail!(
                "Backing Allocation Error! Most likely OOM or not enough memory in backing allocator in Arena for the allocation to succeed"
            );
        };
        let size = buf.len();

        let buf = buf.as_ptr() as *mut u8;
        let buf = NonNull::new(buf).unwrap();

        let s = Self {
            inner: parent,
            used: Cell::new(0),
            size: Cell::new(size),
            buf,
        };

        Ok(s)
    }

    pub const fn used(&self) -> usize {
        self.used.get()
    }

    pub const fn size(&self) -> usize {
        self.size.get()
    }

    pub const fn layout(&self) -> Layout {
        let size = self.size.get() as usize;
        match Layout::array::<u8>(size) {
            Ok(l) => l,
            Err(_) => panic!("Could no create Layout::array::<u8>"),
        }
    }

    pub unsafe fn reset(&self) {
        self.used.set(0);
    }

    const fn top(&self) -> NonNull<u8> {
        let used = self.used();

        unsafe { self.buf.add(used) }
    }

    /// This requires a mutable reference to self so that there is only ever 1 [ScopedArena] per scope
    /// (we abuse borrow semantics a little bit to enforce this, but i think it pays off, as this makes calls to [Arena::reset]  essentially safe)
    pub const fn scoped<'a>(&'a mut self) -> ScopedArena<'a, A> {
        ScopedArena(self)
    }
}

unsafe impl<A: Allocator> Allocator for Arena<A> {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, alloc::alloc::AllocError> {
        let used = self.used();
        let size = self.size();

        let alloc_size = layout.size();
        if alloc_size + used > size {
            return Result::Err(alloc::alloc::AllocError);
        }
        let top = self.top();
        let offset = top.align_offset(layout.align());
        let top = unsafe { top.add(offset) };
        self.used.set(used + offset + alloc_size);

        let sl = NonNull::slice_from_raw_parts(top, alloc_size);
        Result::Ok(sl)
    }

    unsafe fn deallocate(&self, _ptr: NonNull<u8>, _layout: Layout) {}
}

impl<A> Drop for Arena<A>
where
    A: Allocator,
{
    fn drop(&mut self) {
        let layout = self.layout();
        let ptr = self.buf;
        unsafe { self.inner.deallocate(ptr, layout) };
        self.used.set(0);
        self.size.set(0);
    }
}

#[repr(transparent)]
#[derive(Debug)]
pub struct ScopedArena<'a, A: Allocator>(&'a Arena<A>);

unsafe impl<'a, A> Allocator for ScopedArena<'a, A>
where
    A: Allocator,
{
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, alloc::alloc::AllocError> {
        self.0.allocate(layout)
    }

    unsafe fn deallocate(&self, _ptr: NonNull<u8>, _layout: Layout) {}
}

impl<'a, A> Deref for ScopedArena<'a, A>
where
    A: Allocator,
{
    type Target = Arena<A>;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<'a, A> Drop for ScopedArena<'a, A>
where
    A: Allocator,
{
    fn drop(&mut self) {
        unsafe { self.0.reset() };
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ConversionError;

impl Display for ConversionError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "ConversionError")
    }
}

impl Error for ConversionError {}

impl<const S: usize> TryFrom<ArenaStatic<S>> for Arena<ArenaStatic<S>> {
    type Error = ConversionError;
    fn try_from(value: ArenaStatic<S>) -> Result<Self, Self::Error> {
        Self::new(value, S - 1).map_err(|_| ConversionError)
    }
}

impl<'a, const S: usize> TryFrom<&'a ArenaStatic<S>> for Arena<&'a ArenaStatic<S>> {
    type Error = ConversionError;

    fn try_from(value: &'a ArenaStatic<S>) -> Result<Self, Self::Error> {
        Self::new(value, S - 1).map_err(|_| ConversionError)
    }
}

mod tests {
    use core::cell::RefCell;

    use alloc::{alloc::Global, boxed::Box, rc::Rc, vec::Vec};

    use super::*;
    use crate::{basic::const_static::ArenaStatic, buf::kstring::KString};

    #[test]
    fn can_allocate_static() -> anyhow::Result<()> {
        struct Buff([u8; 100]);

        let parent = ArenaStatic::<255>::new();
        let child = Arena::try_from(parent.by_ref())?;
        let mut p = Box::new_in(50, child.by_ref());
        let x = *p;
        assert_eq!(x, 50);
        *p = 100;
        assert_eq!(*p, 100);

        let rc = Rc::new_in(RefCell::new(Buff([0; 100])), child.by_ref());
        {
            let mut buf = rc.as_ref().borrow_mut();
            let mut src = [0; 100];
            for (i, x) in src.iter_mut().enumerate() {
                *x = i as u8;
            }
            *buf = Buff(src);
        }

        for (i, x) in rc.as_ref().borrow().0.iter().enumerate() {
            assert_eq!(*x, i as u8);
        }

        Ok(())
    }

    #[test]
    fn can_allocate_global() -> anyhow::Result<()> {
        let arena = Arena::new(Global, 4096 * 8)?;

        let mut buckets = Vec::with_capacity_in(8, arena.by_ref());

        for i in 0..8 {
            let mut buf = Box::new_in([0usize; 255], arena.by_ref());

            for (i, x) in buf.iter_mut().enumerate() {
                *x = i * i;
            }

            buckets.push(buf);
        }

        for buf in buckets.iter() {
            for (i, x) in buf.iter().enumerate() {
                assert_eq!(*x, i * i);
            }
        }

        Ok(())
    }

    #[test]
    fn can_reset() -> anyhow::Result<()> {
        let arena = Arena::new(Global, 4096 * 8)?;
        {
            let mut buckets = Vec::with_capacity_in(8, arena.by_ref());

            for i in 0..8 {
                let mut buf = Box::new_in([0usize; 255], arena.by_ref());

                for (i, x) in buf.iter_mut().enumerate() {
                    *x = i * i;
                }

                buckets.push(buf);
            }

            for buf in buckets.iter() {
                for (i, x) in buf.iter().enumerate() {
                    assert_eq!(*x, i * i);
                }
            }

            // SAFETY: We know this is safe as all allocations made thus far will not out live past this scope
            unsafe { arena.reset() }
        }

        assert_eq!(arena.used(), 0);

        let mut s = KString::new_in(arena.by_ref());
        s.push("ayye lmao");
        assert_eq!(s.as_str(), "ayye lmao");

        Ok(())
    }

    #[test]
    fn scoped_arena() -> anyhow::Result<()> {
        let mut arena = Arena::new(Global, 4096)?;

        {
            let scoped = arena.scoped();

            let mut s = KString::new_in(scoped.by_ref());
            s.push("ayye lmao");
            assert_eq!(s.as_str(), "ayye lmao");
        }

        let mut s = KString::new_in(arena.by_ref());
        s.push("ayye lmao");
        assert_eq!(s.as_str(), "ayye lmao");
        Ok(())
    }
}
