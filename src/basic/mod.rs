//!
//! This module contains basic allocators that can be used as a parent allocator to
//! any allocators in the [kos::comp] module, which contains Compositional Allocators, which can be initialized with
//! a child allocator to defer all allocations it makes with
//!
//!
//! TODO: Add a Page Allocator that abstracts either my own mmap/(whatever atrocities Windows uses to call for system memory),
//! delegate to some other rust crate that wraps mmap and provide some kind of interface over that (or not, just re-export it)
//! or at least add some other allocators that can be used as a Global allocator (must impl [core::marker::Sync]).
//!
//!
//! [const_static::ArenaStatic]
//!
//! is a Bump-Style Arena Allocator, where the buffer/memory it allocates into
//! is a statically determined size. You cannot resize this type. Im working on an Arena Type that is resizeable, but that
//! is still a WIP anyway you can use [const_static::ArenaStatic] as a parent allocator for others, or as a locally scoped allocator for
//! [core]/[kos]/your own types. [const_static::ArenaStatic] cannot be declared as a [global_allocator], as it does not impl [core::marker::Sync].
//!
//! TODO: Create a 'ArenaStaticSync' type that does impl [core::marker::Sync] and can therefore be used as a [global_allocator]
//!
//!
//! [sync::ArenaSync]
//!
//! A version of [const_static::ArenaStatic] that is safe to share borrowed references to it across threads and can
//! be declared as [global_allocator] to it implementing [core::marker::Sync]
//!

pub mod arena_static;
pub mod slot;
pub mod slot_ptr;
pub mod static_sync;
pub mod sync;
