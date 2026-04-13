use core::{
    alloc::Layout,
    fmt::Write,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

use alloc::alloc::Allocator;
use anyhow::Context;

use crate::basic::slot::{AllocSlot, SlotLookup, SlotMap};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct SlotMapPtr<Val, A: Allocator> {
    slot: AllocSlot,
    sm: NonNull<SlotMap<A>>,
    _pd: PhantomData<NonNull<Val>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct SlotMapRef<'val, Val, A: Allocator> {
    slot: AllocSlot,
    sm: NonNull<SlotMap<A>>,
    _pd: PhantomData<&'val Val>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct SlotMapMut<'val, Val, A: Allocator> {
    slot: AllocSlot,
    sm: NonNull<SlotMap<A>>,
    _pd: PhantomData<&'val mut Val>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct SlotMapAnyPtr<A: Allocator> {
    slot: AllocSlot,
    sm: NonNull<SlotMap<A>>,
    _pd: PhantomData<NonNull<u8>>,
}

impl<'val, V, A> SlotMapRef<'val, V, A>
where
    A: Allocator,
{
    pub const fn from_parts(index: usize, layout: Layout, sm: &SlotMap<A>) -> Self {
        Self::with_slot(AllocSlot::new(index, layout), sm)
    }

    pub const fn with_slot(slot: AllocSlot, sm: &SlotMap<A>) -> Self {
        Self {
            slot,
            sm: NonNull::from_ref(sm),
            _pd: PhantomData,
        }
    }

    pub fn as_ref(&self) -> &'val V {
        let p = self.as_ptr();
        unsafe { p.as_ref() }
    }

    #[inline]
    pub fn as_ptr(&self) -> NonNull<V> {
        self.slot_cell()
            .expect("Failed to lookup Slot Cell from dereferencing a SlotMapRef!")
    }

    pub const fn to_map_ptr(self) -> SlotMapPtr<V, A> {
        SlotMapPtr {
            slot: self.slot,
            sm: self.sm,
            _pd: PhantomData,
        }
    }

    pub const fn to_map_any(self) -> SlotMapAnyPtr<A> {
        SlotMapAnyPtr {
            slot: self.slot,
            sm: self.sm,
            _pd: PhantomData,
        }
    }
}

impl<'val, V, A> SlotMapMut<'val, V, A>
where
    A: Allocator,
{
    pub const fn from_parts(index: usize, layout: Layout, sm: &SlotMap<A>) -> Self {
        Self::with_slot(AllocSlot::new(index, layout), sm)
    }

    pub const fn with_slot(slot: AllocSlot, sm: &SlotMap<A>) -> Self {
        Self {
            slot,
            sm: NonNull::from_ref(sm),
            _pd: PhantomData,
        }
    }

    #[inline]
    pub fn as_ptr(&self) -> NonNull<V> {
        self.slot_cell()
            .expect("Failed to lookup slot cell when dereferencing a SlotMapMut!")
    }

    pub fn as_ref(&self) -> &'val V {
        let ptr = self.as_ptr();
        unsafe { ptr.as_ref() }
    }

    pub fn as_mut(&mut self) -> &'val mut V {
        let mut ptr = self.as_ptr();
        unsafe { ptr.as_mut() }
    }
}

impl<A> SlotMapAnyPtr<A>
where
    A: Allocator,
{
    pub const fn from_parts(index: usize, layout: Layout, sm: &SlotMap<A>) -> Self {
        Self::with_slot(AllocSlot::new(index, layout), sm)
    }

    pub const fn with_slot(slot: AllocSlot, sm: &SlotMap<A>) -> Self {
        Self {
            slot,
            sm: NonNull::from_ref(sm),
            _pd: PhantomData,
        }
    }

    pub const fn cast<V>(self) -> SlotMapPtr<V, A> {
        SlotMapPtr {
            slot: self.slot,
            sm: self.sm,
            _pd: PhantomData,
        }
    }

    pub fn as_ptr(&self) -> NonNull<[u8]> {
        self.slot_cell()
            .expect("Failed to lookup slot cell while dereferencing a SlotMapAnyPtr!")
    }

    pub fn as_bytes(&self) -> &[u8] {
        let ptr = self.as_ptr();
        unsafe { core::slice::from_raw_parts(ptr.cast::<u8>().as_ptr(), ptr.len()) }
    }

    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        let ptr = self.as_ptr();
        unsafe { core::slice::from_raw_parts_mut(ptr.cast::<u8>().as_ptr(), ptr.len()) }
    }
}

impl<V, A> SlotMapPtr<V, A>
where
    A: Allocator,
{
    pub const fn from_parts(index: usize, layout: Layout, sm: &SlotMap<A>) -> Self {
        Self::with_slot(AllocSlot::new(index, layout), sm)
    }

    pub const fn with_slot(slot: AllocSlot, sm: &SlotMap<A>) -> Self {
        let sm = NonNull::from_ref(sm);
        Self {
            slot,
            sm,
            _pd: PhantomData,
        }
    }

    #[inline]
    pub fn try_as_ptr(&self) -> Option<NonNull<V>> {
        self.slot_cell()
    }

    pub fn as_ptr(&self) -> NonNull<V> {
        self.slot_cell()
            .expect("Failed to lookup slot cell when dereferencing SlotMapPtr!")
    }

    pub fn write_value(&self, val: V) -> anyhow::Result<()> {
        let p = self
            .try_as_ptr()
            .context("Failed call to SlotMapPtr::slot_cell!")?;

        if core::mem::needs_drop::<V>() {
            unsafe { *p.as_ptr() = val };
        } else {
            unsafe { NonNull::write(p, val) };
        }
        Ok(())
    }

    pub unsafe fn as_inner_ref(&self) -> &V {
        let ptr = self.as_ptr();
        unsafe { ptr.as_ref() }
        // .expect("Failed to lookup slot cell while dereferencing a SlotMapPtr!");
    }

    pub unsafe fn as_inner_mut(&mut self) -> &mut V {
        let mut ptr = self.as_ptr();
        unsafe { ptr.as_mut() }
    }

    pub unsafe fn as_bytes(&self) -> &[u8] {
        let ptr = self.as_ptr();
        let ptr = ptr.cast::<u8>();
        unsafe { core::slice::from_raw_parts(ptr.as_ptr() as *const _, core::mem::size_of::<V>()) }
    }

    pub unsafe fn as_bytes_mut(&mut self) -> &mut [u8] {
        let ptr = self.as_ptr();
        let ptr = ptr.cast::<u8>();
        unsafe {
            core::slice::from_raw_parts_mut(ptr.as_ptr() as *mut _, core::mem::size_of::<V>())
        }
    }

    /// Transforms [SlotMapPtr] into a [SlotMapRef].
    ///
    /// This creates a new lifetime, similar to creating
    /// a borrowed reference from a raw pointer, similar to ex: [NonNull::as_ref]
    ///
    /// # Safety
    ///
    /// When calling this method, you have to ensure that the pointer
    /// is convertable to a reference
    ///
    ///
    pub const unsafe fn as_ref(&self) -> SlotMapRef<'_, V, A> {
        SlotMapRef {
            slot: self.slot,
            sm: self.sm,
            _pd: PhantomData,
        }
    }

    /// Transforms a [SlotMapPtr] into a [SlotMapMut].
    ///
    /// This creates a new lifetime, similar to creating a (mutable!) borrowed
    /// reference from a raw pointer, similar to ex: [NonNull::as_mut]
    ///
    /// # Safety
    ///
    /// When calling this method, you have to ensure that the pointer
    /// is convertable to a reference
    ///
    pub const unsafe fn as_mut<'a>(self) -> SlotMapMut<'a, V, A> {
        SlotMapMut {
            slot: self.slot,
            sm: self.sm,
            _pd: PhantomData,
        }
    }

    /// Transforms a [SlotMapPtr] of type [V] into a [SlotMapPtr] of type [To]
    ///
    pub const fn cast<To>(self) -> SlotMapPtr<To, A> {
        SlotMapPtr {
            slot: self.slot,
            sm: self.sm,
            _pd: PhantomData,
        }
    }

    /// Transforms a [SlotMapPtr] into a [SlotMapAnyPtr],
    /// thus erasing and forgetting the type this pointer-like type points to
    pub const fn erase_type(self) -> SlotMapAnyPtr<A> {
        SlotMapAnyPtr {
            slot: self.slot,
            sm: self.sm,
            _pd: PhantomData,
        }
    }
}

/// General Trait for derefing [SlotMapPtr], [SlotMapRef], [SlotMapMut], and [SlotMapAnyPtr]
///  This is significant, i think, as it provides a general interface for which to lookup
/// allocated memory purely by index, on every call to [Deref::deref], thus allowing for
/// our [SlotMap] to resize safely without causing any UB (hopefully! lol we shall see!)
pub trait SlotCell<Pointee: ?Sized = [u8]> {
    /// Looks up memory allocated by [SlotMap] by index, instead of pointer or reference
    fn slot_cell(&self) -> Option<NonNull<Pointee>>;
}

impl<V, A> SlotCell<V> for SlotMapPtr<V, A>
where
    A: Allocator,
{
    fn slot_cell(&self) -> Option<NonNull<V>> {
        let sm = unsafe { self.sm.as_ref() };
        sm.lookup_slot(self.slot).map(|p| p.cast::<V>())
    }
}

impl<'val, V, A> SlotCell<V> for SlotMapRef<'val, V, A>
where
    A: Allocator,
{
    fn slot_cell(&self) -> Option<NonNull<V>> {
        let sm = unsafe { self.sm.as_ref() };
        sm.lookup_slot(self.slot).map(|p| p.cast::<V>())
    }
}

impl<'val, V, A> SlotCell<V> for SlotMapMut<'val, V, A>
where
    A: Allocator,
{
    fn slot_cell(&self) -> Option<NonNull<V>> {
        let sm = unsafe { self.sm.as_ref() };
        sm.lookup_slot(self.slot).map(|p| p.cast::<V>())
    }
}

impl<A> SlotCell<[u8]> for SlotMapAnyPtr<A>
where
    A: Allocator,
{
    fn slot_cell(&self) -> Option<NonNull<[u8]>> {
        let sm = unsafe { self.sm.as_ref() };
        sm.lookup_slot(self.slot)
    }
}

impl<A> Deref for SlotMapAnyPtr<A>
where
    A: Allocator,
{
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.as_bytes()
    }
}

impl<A> DerefMut for SlotMapAnyPtr<A>
where
    A: Allocator,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_bytes_mut()
    }
}

impl<'val, V, A> Deref for SlotMapRef<'val, V, A>
where
    A: Allocator,
{
    type Target = V;

    fn deref(&self) -> &'val Self::Target {
        self.as_ref()
    }
}

impl<'val, V, A> Deref for SlotMapMut<'val, V, A>
where
    A: Allocator,
{
    type Target = V;

    fn deref(&self) -> &'val Self::Target {
        self.as_ref()
    }
}

impl<'val, V, A> DerefMut for SlotMapMut<'val, V, A>
where
    A: Allocator,
{
    fn deref_mut(&mut self) -> &'val mut Self::Target {
        self.as_mut()
    }
}

impl<V, A> Deref for SlotMapPtr<V, A>
where
    A: Allocator,
{
    type Target = V;

    fn deref(&self) -> &Self::Target {
        unsafe { self.as_inner_ref() }
    }
}

impl<V, A> DerefMut for SlotMapPtr<V, A>
where
    A: Allocator,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.as_inner_mut() }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
//     #[repr(C)]
//     struct Point {
//         x: i32,
//         y: i32,
//     }

//     // #[test]
//     // fn slot_map_works() {
//     //     // let sm = SlotMap::new();
//     //     // let v = sm.push_value(500);
//     //     // assert_eq!(*v, 500);

//     //     // let v2 = sm.push_value(Point { x: 50, y: 50 });
//     //     // assert_eq!(*v2, Point { x: 50, y: 50 });
//     // }
// }
