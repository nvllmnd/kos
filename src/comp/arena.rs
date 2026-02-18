use core::{
    alloc::{GlobalAlloc, Layout},
    cell::Cell,
    error::Error,
    fmt::Display,
    ptr::NonNull,
};

use alloc::alloc::Allocator;

use crate::basic::const_static::ArenaStatic;

#[derive(Debug, Clone)]
pub struct Arena<A: Allocator> {
    inner: A,
    used: Cell<u32>,
    size: Cell<u32>,
    buf: NonNull<u8>,
}

impl<A: Allocator> Arena<A> {
    pub fn new(parent: A, cap: usize) -> anyhow::Result<Self> {
        let buf = parent.allocate(Layout::array::<u8>(cap).unwrap())?;
        let size = buf.len();
        let buf = buf.as_ptr() as *mut u8;
        let buf = NonNull::new(buf).unwrap();
        let s = Self {
            inner: parent,
            used: Cell::new(0),
            size: Cell::new(size as u32),
            buf,
        };
        Ok(s)
    }

    pub const fn layout(&self) -> Layout {
        let size = self.size.get() as usize;
        match Layout::array::<u8>(size) {
            Ok(l) => l,
            Err(_) => panic!("Could no create Layout::array::<u8>"),
        }
    }

    pub unsafe fn clear(&self) {
        self.used.set(0);
    }

    fn top(&self) -> NonNull<u8> {
        let used = self.used.get();

        unsafe { self.buf.add(used as usize) }
    }
}

unsafe impl<A: Allocator> Allocator for Arena<A> {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, alloc::alloc::AllocError> {
        let used = self.used.get() as usize;
        let size = self.size.get() as usize;

        let alloc_size = layout.size();
        if alloc_size + used > size {
            return Result::Err(alloc::alloc::AllocError);
        }
        let top = self.top();
        let offset = top.align_offset(layout.align());
        let top = unsafe { top.add(offset) };
        self.used.set((offset + alloc_size) as u32);

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

    use alloc::{boxed::Box, rc::Rc};

    use super::*;
    use crate::basic::const_static::ArenaStatic;

    #[test]
    fn can_allocate() -> anyhow::Result<()> {
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
}
