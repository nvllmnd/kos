use core::cell::Cell;
use core::cell::UnsafeCell;
use core::ptr::NonNull;

#[repr(C)]
pub struct Inline<const SIZE: usize> {
    storage: UnsafeCell<[u8; SIZE]>,
    used: Cell<u32>,
}

pub struct Block {
    ptr: NonNull<u8>,
}
