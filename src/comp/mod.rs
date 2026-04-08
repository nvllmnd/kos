//! This module contains types that are also generic over [alloc::alloc::Allocator],
//! allowing you to compose allocators and plug in your own implementations of [alloc::alloc::Allocator]
//!
//! [arena::Arena]
//!
//! is what you would consider your standard bump/arena allocator. It has a [arena::Arena::scoped] method
//! on it that returns a [arena::ScopedArena], which safely resets (but does not drop! at least not yet, i may add that feature in the future...)
//! the memory allocated during the [arena::ScopedArena]'s lifetime. This provides a safe way to reset the arena and set the bump index back to 0, effectively clearing the memory.
//!
//! CAUTION: If you allocate something with [arena::ScopedArena] that implements [core::ops::Drop], IT WILL NOT BE RAN WHEN [arena::ScopedArena] GOES OUT OF SCOPE. THIS COULD LEAD
//! TO VERY BAD THINGS SO BE CAREFUL, OR MANUALLY DROP EVERYTHING YOU NEED TO DROP BEFORE ALLOWING [arena::ScopedArena] TO LEAVE SCOPE
//!
//!

pub mod arena;
