//!
//! This module provides container types that are generic over the [alloc::alloc::Allocator] trait
//!
//! So far this is just [kstring::KString] and [ustring::UString],
//!
//! ### [kstring::KString]
//!
//! is a drop-in replacement for [alloc::string::String] that is generic over the [alloc::alloc::Allocator] trait
//! there are a potentially few
//!
//! #### Differences from [alloc::string::String], as of 04/08/2026:
//!
//! - FIXME: [kstring::KString] not having its reference taken
//! automatically like [alloc::string::String] does, ex:
//!
//! ```rust
//!
//! fn format_kstring() {
//!
//!    let s = kos::buf::kstring::KString::from("ello m8!");
//!    assert_eq!(&s, "ello m8!"); // borrow must be taken here
//! }
//!
//! fn format_string() {
//!     let s = String::from("ello m8!");
//!
//!     // compiler knows to borrow (or it impls a From or Deref trait that tells the
//!     // compiler to deref/borrow it automatically, which one it is I forget lol :D)
//!     assert_eq!(s, "ello m8!");
//! }
//!
//! ```
//!
//!
//! ### [ustring::UString]
//!
//! is a small-buffer optimized string.
//! So [ustring::UString]s created/appended to, making it a string of total length shorter than 23 bytes/characters long
//! is not allocated on the heap at all, and is instead kept on the stack. Since [ustring::UString] has exclusive ownership
//! of the the string it contains, this is all well and good. If a short string less than 23 bytes long is appeneded to, or a string longer than 23 bytes is created,
//! then that string data is allocated onto the heap, just like this crate's [kstring::KString] or [alloc::string::String]
//!
//!
//!
pub mod kstring;
pub mod ustring;
