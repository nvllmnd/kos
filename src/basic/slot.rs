use core::{
    alloc::Layout,
    cell::UnsafeCell,
    marker::PhantomData,
    ops::{Deref, DerefMut, Index},
    ptr::NonNull,
    sync::atomic::AtomicUsize,
};

use alloc::{
    alloc::{Allocator, Global},
    vec::Vec,
};

use crate::basic::{
    slot_ptr::{SlotMapAnyPtr, SlotMapPtr},
    static_sync::ArenaStaticSync,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct AllocSlot {
    i: u32,
    layout: Layout,
}

impl AllocSlot {
    pub const fn no_size(slot: usize) -> Self {
        Self {
            i: slot as u32,
            layout: Layout::new::<()>(),
        }
    }

    pub const fn new(slot: usize, layout: Layout) -> Self {
        Self {
            i: slot as u32,
            layout,
        }
    }

    pub const fn from_type<V>(slot: usize) -> Self {
        Self {
            i: slot as u32,
            layout: Layout::new::<V>(),
        }
    }

    pub(crate) const fn slot(&self) -> usize {
        self.i as usize
    }

    pub const fn layout(&self) -> Layout {
        self.layout
    }
}

/// pointer to an allocation made by [ArenaStaticSync], that is tagged
/// with the byte offset index to where the pointer SHOULD be pointing to.
/// you can use this to verify an allocation is currently valid and not dangling from an
/// inproper resetting of the Arena
pub trait SlottedAllocator: Allocator {
    fn lookup(&self, tag: AllocSlot) -> Option<NonNull<u8>>;
}

pub trait SlotLookup {
    fn lookup_slot(&self, slot: AllocSlot) -> Option<NonNull<[u8]>>;
}

pub struct SlotPtr<A: SlottedAllocator> {
    slot: AllocSlot,
    parent: A,
}

impl<A> SlotPtr<A>
where
    A: SlottedAllocator,
{
    pub const fn new(slot: usize, layout: Layout, parent: A) -> Self {
        Self {
            slot: AllocSlot::new(slot, layout),
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
    pub fn lookup_ptr(&self, AllocSlot { i, layout }: AllocSlot) -> Option<NonNull<u8>> {
        let i = i as usize;
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
            slot: AllocSlot::new(slot, layout),
            parent: self.by_ref(),
        };
        sl
    }
}

#[derive(Debug)]
pub struct SlotMap<A: Allocator> {
    buf: UnsafeCell<NonNull<[u8]>>, // buf: Vec<u8, A>,
    used: AtomicUsize,
    alloc: A,
}

impl SlotMap<Global> {
    pub const fn new() -> Self {
        let buf: NonNull<[u8]> = NonNull::slice_from_raw_parts(NonNull::dangling(), 0);
        let buf = UnsafeCell::new(buf);
        let used = AtomicUsize::new(0);
        let alloc = alloc::alloc::Global;
        Self { buf, used, alloc }
    }

    pub fn with_capacity(cap: usize) -> Self {
        let alloc = alloc::alloc::Global;
        let buf = alloc
            .allocate(Layout::array::<u8>(cap).unwrap())
            .expect("Allocation Failed! Global Allocator most likely out of memory!");
        let buf = UnsafeCell::new(buf);
        let used = AtomicUsize::new(0);
        Self { buf, used, alloc }
    }
}

impl<A> SlotMap<A>
where
    A: Allocator,
{
    pub fn lookup_any_ptr(&self, slot: AllocSlot) -> Option<SlotMapAnyPtr<A>> {
        let ptr = self.lookup_slot(slot)?;
        let res = SlotMapAnyPtr::with_slot(slot, self);
        Some(res)
    }

    pub const fn ptr_from_type<V>(&self, index: usize) -> SlotMapPtr<V, A> {
        let slot = AllocSlot::new(index, Layout::new::<V>());
        SlotMapPtr::with_slot(slot, self)
    }

    pub const fn new_in(alloc: A) -> Self {
        let buf = NonNull::slice_from_raw_parts(NonNull::dangling(), 0);
        let buf = UnsafeCell::new(buf);
        let used = AtomicUsize::new(0);
        Self { buf, used, alloc }
    }

    pub fn with_capacity_in(cap: usize, alloc: A) -> Self {
        let buf = alloc
            .allocate(Layout::array::<u8>(cap).unwrap())
            .expect("Allocation Failed! Global Allocator most likely out of memory!");
        let buf = UnsafeCell::new(buf);
        let used = AtomicUsize::new(0);
        Self { buf, used, alloc }
    }

    pub const fn buf(&self) -> NonNull<[u8]> {
        let nn = unsafe {
            self.buf.get().as_ref().expect(
                "Inner NonNull<[u8]> is somehow null when being access through an UnsafeCell!",
            )
        };
        *nn
    }

    // pub const fn as_bytes(&self) -> &[u8] {
    //     let buf = self.buf();
    //     unsafe { buf.as_ref() }
    // }

    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        if self.is_empty() {
            &[]
        } else {
            let buf = self.buf();
            unsafe { buf.as_ref() }
        }
    }

    pub fn as_bytes_mut(&self) -> &mut [u8] {
        if self.is_empty() {
            &mut []
        } else {
            let mut buf = self.buf();
            unsafe { buf.as_mut() }
        }
    }

    /// [SlotMap]'s version of malloc. Returns an [AllocSlot]
    /// that you can use to either crate some [SlotMapPtr]s, [SlotMapRef]s, or [SlotMapMut], (if you know the type)
    /// or simply use [SlotLookup::lookup_slot] to get the raw [NonNull] u8 pointer
    /// that points to the memory to be used for an allocated slot
    #[inline]
    pub fn malloc_slot(&self, layout: Layout) -> AllocSlot {
        self.prealloc_slot(layout)
    }

    pub fn push_value<V>(&self, val: V) -> SlotMapPtr<V, A> {
        let slot = self.malloc_slot(Layout::new::<V>());
        SlotMapPtr::with_slot(slot, self)
    }

    /// Linearly scans through currently allocated memory for an address that matches the
    /// address given from @param(ptr). To do the same thing, but with a constant borrow, see [Self::find_index_from_ref]
    pub fn find_index_from_ptr(&self, ptr: NonNull<u8>) -> Option<AllocSlot> {
        if self.is_empty() {
            None
        } else {
            let buf = self.buf();
            let len = buf.len();

            let mut p = buf.cast::<u8>();
            for i in 0..len {
                if p.addr() == ptr.addr() {
                    return Some(AllocSlot::no_size(i));
                }
                p = unsafe { p.add(1) };
            }

            None
        }
        // for x in buf.() {}
    }

    /// Forwards call to [Self::find_index_from_ptr] after changing given &V to a [NonNull] u8
    #[inline]
    pub fn find_index_from_ref<V>(&self, val: &V) -> Option<AllocSlot> {
        let p = NonNull::from_ref(val);
        self.find_index_from_ptr(p.cast::<u8>())
    }

    /// Here we check against the value returned by [Self::used], since there
    /// is a possiblity that an instance of [SlotMap] was created with a [NonNull::dangling],
    /// so we want to avoid reading/writing through our buf pointer until we know
    /// forsure that it is pointing to actual valid memory currently
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.used() == 0
    }

    /// Same thing as calling [Self::is_empty], but with a more clear direction
    /// of intention
    #[inline]
    pub fn is_dangling(&self) -> bool {
        self.is_empty()
    }

    /// Light wrapper around [AtomicUsize::load], or essentially
    /// fetches the used value with [core::sync::atomic::Ordering::Relaxed] atomic ordering
    #[inline]
    pub fn used(&self) -> usize {
        self.used.load(core::sync::atomic::Ordering::Relaxed)
    }

    /// Get a reference to [SlotMap]'s parent allocator
    pub const fn allocator(&self) -> &A {
        &self.alloc
    }

    /// Gets currently allocated memory capactiy
    pub fn capacity(&self) -> usize {
        if self.is_empty() {
            return 0;
        }

        let buf = self.buf.get();
        let buf = unsafe {
            buf.as_ref()
                .expect("Inner NonNull<[u8]> is somehow null when accessing through UnsafeCell!")
        };
        buf.len()
    }

    fn init_memory(&self, init_size: usize) {
        // memory is already initialized, bail
        if self.used() > 0 {
            return;
        }

        let layout = Layout::array::<u8>(init_size).unwrap();
        let mem = self.allocator().allocate(layout).expect(
            "SlotMap Inner Allocator returned an error from a call to Allocator::allocate!",
        );

        self.set_buf(mem);
    }

    fn set_buf(&self, new_buf: NonNull<[u8]>) {
        let cur_buf = self.buf.get();
        unsafe { core::ptr::write(cur_buf, new_buf) };
    }

    /// Grows owned memory by @param(n) bytes, returns
    /// the size of the new allocation (which as of now is just double the last size)
    fn grow_memory(&self, n: usize) -> usize {
        if self.is_dangling() {
            self.init_memory(n);
            n
        } else {
            let old = self.buf();
            let old_layout = Layout::array::<u8>(old.len()).unwrap();
            let layout = Layout::array::<u8>(old.len() * 2).unwrap();
            let new_mem = self.allocator().allocate(layout).expect(
                "SlotMap Inner Allocator Return an error when trying to call Allocator::allocate!",
            );

            let dst = new_mem.cast::<u8>();
            let src = old.cast::<u8>();
            let size = old.len();

            // Copy memory from old allocation into our new allocation, and then deallocate the old pointer
            unsafe {
                NonNull::copy_from_nonoverlapping(dst, src, size);

                self.allocator().deallocate(old.cast::<u8>(), old_layout);
            };
            self.set_buf(new_mem);
            new_mem.len()
        }
    }

    /// Light wrapper around [AtomicUsize::store]
    #[inline]
    fn store_used(&self, new_value: usize) {
        self.used
            .store(new_value, core::sync::atomic::Ordering::Relaxed);
    }

    /// Light wrapper around [Self::used_fetch_add], but drops/ignores the return value
    /// and justa adds @param(n) to current value of [Self::used]
    #[inline]
    fn inc_used(&self, n: usize) {
        let _ = self.used_fetch_add(n);
    }

    /// Light wrapper around [AtomicUsize::fetch_add]
    #[inline]
    fn used_fetch_add(&self, addened: usize) -> usize {
        self.used
            .fetch_add(addened, core::sync::atomic::Ordering::Relaxed)
    }

    /// Gets the offset pointer to where the next allocation may lie (does not take into consideration align_offset!)
    #[inline]
    pub fn used_ptr(&self) -> NonNull<u8> {
        unsafe { self.buf().cast::<u8>().add(self.used()) }
    }

    /// Gets the align_offset value, taken from a call to [NonNull::align_offset] on the pointer returned by [Self::used_ptr]
    /// using the @param(layout) [Layout::align]
    #[inline]
    pub fn prealloc_align_offset(&self, layout: Layout) -> usize {
        let up = self.used_ptr();
        up.align_offset(layout.align())
    }

    /// Gets the slot index for the next allocation. Use this to grab
    /// an [AllocSlot] before an allocation is made (through standard [alloc::alloc::Allocator] trait interface)
    /// Takes into consideration the align_offset (as just grabbing [Self::used] might not be entirely accurate)
    pub fn prealloc_slot(&self, layout: Layout) -> AllocSlot {
        let used = self.used();
        let offset = self.prealloc_align_offset(layout);
        let slot = (used + offset) as u32;
        AllocSlot::new(slot as usize, layout)
    }
}

unsafe impl<A> Allocator for SlotMap<A>
where
    A: Allocator,
{
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, alloc::alloc::AllocError> {
        let slot = self.malloc_slot(layout);
        let ptr = self.lookup_slot(slot).ok_or(alloc::alloc::AllocError)?;
        Ok(ptr)
    }

    unsafe fn deallocate(&self, _ptr: NonNull<u8>, _layout: Layout) {
        // NOTE: For now do nothing, i may write a mechanism to deallocate slots, where
        // i can keep a separate buffer/vec of [AllocSlot]s that are free to use for
        // new allocations, this buf would also hold the [Layout] of the slot so that
        // we can be sure new allocation will fit into empty slots that were previously deallocated.
        // This is my solution to memory fragmentation when using this data structure [SlotMap]
    }
}

impl<A> SlotLookup for SlotMap<A>
where
    A: Allocator,
{
    fn lookup_slot(&self, slot: AllocSlot) -> Option<NonNull<[u8]>> {
        let AllocSlot { i, .. } = slot;
        let i = i as usize;
        if i < self.capacity() {
            let buf = self.buf().cast::<u8>();
            let ptr = unsafe { buf.add(i) };
            let sl = NonNull::slice_from_raw_parts(ptr, slot.layout().size());
            Some(sl)
        } else {
            None
        }
    }
}
