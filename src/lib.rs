//!
//! This crate contains types and functions for use as custom allocators that all
//! at least implement the [alloc::alloc::Allocator] trait.
//! (some also implement the [alloc::alloc::GlobalAlloc] trait as well, but that is only helpful if that type also implements [core::marker::Sync],
//! which is something I need to tidy up and make consistent)
//!
//! [alloc::string::String] is not generic over [alloc::alloc::Allocator], so you can
//! use [crate::buf::kstring::KString] for creating strings that use an
//! allocator different from the [alloc::alloc::Global] allocator ([global_allocator])
//!
//! The [crate::comp] module contains allocators that can be composed with other allocators, allowing you
//! to plug in your own implementations of the [alloc::alloc::Allocator] trait
//!
//!
//!
//! #### On why I did not write my own HashMap/HashSet types
//!
//! - Here we are re-exporting hashmaps from the [hashbrown] crate, as that
//! is what the rust [std] library uses internally anyway (as of right now on 04/08/2026 in nightly Rust).
//!
//! - The [hashbrown] crate developers were so very nice as to make all their associative array types generic over
//! the [alloc::alloc::Allocator] trait, so I would really be doing a lot of double work by implementing my own
//! hashmap and hashset types on my own.
//!
//! - Finally, from a breif, surface level look at the hashbrown crate, it looks like it lets you plug in
//! your own custom hashers and all that. It provides a low-level [hashbrown::HashTable] interface, which im guessing one could do
//! a lot with to tweak the behavior of the hash-map/set//!
//!
//!
//! All that being said, I may eventually write my own as a fun-ish exercise, but I will probably end up
//! writing it in C++, C, Zig or something, as its apparently not so trivial to do so efficiently in Rust, because of lifetimes and all
//! that good stuff. emulating a [alloc::string::String] that is generic over [alloc::alloc::Allocator] is hard enough! =P
//!
#![no_std]
#![feature(allocator_api)]

use alloc::{alloc::Allocator, vec::Vec};

extern crate alloc;

pub mod basic;
pub mod buf;
pub mod comp;
pub(crate) mod hash;

//
#[cfg(feature = "hashbrown_hashmaps")]
pub use hashbrown::HashMap as Map;

#[cfg(feature = "hashbrown_hashmaps")]
pub use hashbrown::HashSet as Set;

#[cfg(feature = "hashbrown_hashmaps")]
pub use hashbrown::HashTable as Table;

#[cfg(feature = "hashbrown_hashmaps")]
pub use hashbrown::*;

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
