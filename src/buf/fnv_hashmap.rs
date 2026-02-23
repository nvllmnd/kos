use alloc::{alloc::Allocator, vec::Vec};

#[derive(Debug, Clone)]
pub struct FnvHashMap<V, A: Allocator> {
    keys: Vec<Key<A>, A>,
    vals: Vec<V, A>,
    len: usize,
}
