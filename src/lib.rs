#![no_std]
#![feature(allocator_api)]

use alloc::{alloc::Allocator, vec::Vec};

extern crate alloc;

pub mod basic;
pub mod buf;
pub mod comp;
pub(crate) mod hash;

#[inline]
pub fn clamp<T>(lower: T, val: T, higher: T) -> T
where
    T: Ord,
{
    core::cmp::max(lower, core::cmp::min(val, higher))
}

pub fn append_byte_slice<A>(dst: &mut Vec<u8, A>, src: &[u8])
where
    A: Allocator,
{
    let i = dst.len() - 1;
    let end = i + src.len();
    dst.resize(dst.len() + src.len(), 0);
    assert!(end < dst.len());
    let dst = &mut dst[i..=end];

    copy_slice_into(src, dst);
}

#[inline]
pub fn copy_slice_into<T>(src: &[T], dst: &mut [T])
where
    T: Copy,
{
    let n = core::cmp::min(dst.len(), src.len());
    dst[..n].copy_from_slice(&src[..n])
}

#[inline]
pub fn clone_slice_into<T>(src: &[T], dst: &mut [T])
where
    T: Clone,
{
    let n = core::cmp::min(dst.len(), src.len());
    dst[..n].clone_from_slice(&src[..n])
}

pub unsafe fn array_from_raw<const S: usize, T>(ptr: *const T) -> [T; S]
where
    T: Default + Copy,
{
    let mut i = 0;
    let mut result = [T::default(); S];
    while i < S {
        let elem = unsafe { ptr.add(i).as_ref().expect("null ptr deref!!!") };
        result[i] = *elem;
        i += 1;
    }
    result
}

#[inline]
pub fn array_from_slice<const S: usize, T>(sl: &[T]) -> [T; S]
where
    T: Default + Copy,
{
    array_from_slice_with(sl, T::default())
}

pub fn array_from_slice_with<const S: usize, T>(sl: &[T], default_val: T) -> [T; S]
where
    T: Copy,
{
    let mut arr = [default_val; S];
    copy_slice_into(sl, &mut arr);
    arr
}
