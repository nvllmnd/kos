use core::{intrinsics::copy_nonoverlapping, marker::PhantomData, ops::Deref, ptr::NonNull};

use alloc::{
    alloc::{Allocator, Global},
    vec::{self, Vec},
};

use crate::{buf::kstring::KString, copy_slice_into};

const INLINE_KEY_LEN: usize = 23;

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
struct InlineKey {
    len: u8,
    buf: [u8; INLINE_KEY_LEN],
}

impl InlineKey {
    pub const fn new() -> Self {
        Self {
            len: 0,
            buf: [0u8; INLINE_KEY_LEN],
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        if s.len() > INLINE_KEY_LEN {
            None
        } else {
            let mut buf = [0u8; INLINE_KEY_LEN];
            copy_slice_into(&mut buf, s.as_bytes());
            let s = Self {
                len: s.len() as u8,
                buf,
            };
            Some(s)
        }
    }

    pub const fn len(&self) -> usize {
        self.len as usize
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        core::str::from_utf8(&self.buf[..self.len()])
            .expect("u8 buf should be utf8 compatible to read as &str")
    }
}

impl Deref for InlineKey {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

#[derive(Debug, Clone, Default)]
enum Key<A: Allocator> {
    #[default]
    Empty,
    Inline(InlineKey),
    Heap(KString<A>),
}

impl<A> Key<A>
where
    A: Allocator,
{
    pub const fn new() -> Self {
        Self::Empty
    }

    pub const fn new_in(alloc: A) -> Self {
        Self::Heap(KString::new_in(alloc))
    }

    pub const fn is_empty(&self) -> bool {
        match self {
            Self::Empty => true,
            Self::Inline(inline_key) => inline_key.len() == 0,
            Self::Heap(kstring) => kstring.len() == 0,
        }
    }

    #[inline]
    pub fn hash64(&self) -> u64 {
        crate::hash::fnv::hash_string_ct(self.as_str())
    }

    #[inline]
    pub fn hash_index(&self, size: usize) -> usize {
        self.hash64() as usize % size
    }

    pub fn as_str(&self) -> &str {
        match self {
            Key::Empty => "",
            Key::Inline(inline_key) => inline_key.as_str(),
            Key::Heap(kstring) => kstring.as_str(),
        }
    }

    pub fn from_str_in(s: &str, alloc: A) -> Self {
        if s.len() > INLINE_KEY_LEN {
            Self::Heap(KString::from_str_in(s, alloc))
        } else {
            Self::Inline(InlineKey::from_str(s).unwrap())
        }
    }
}

impl Key<Global> {
    pub fn from_str(s: &str) -> Self {
        Self::from_str_in(s, Global)
    }
}

impl<A> Deref for Key<A>
where
    A: Allocator,
{
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

#[derive(Debug, Clone)]
struct KeyValue<V, A: Allocator> {
    key: Key<A>,
    val: V,
}

impl<V, A> KeyValue<V, A>
where
    A: Allocator,
{
    pub const fn new(key: Key<A>, val: V) -> Self {
        Self { key, val }
    }

    pub const fn is_empty(&self) -> bool {
        self.key.is_empty()
    }

    #[inline]
    pub fn key_hash(&self) -> u64 {
        self.key.hash64()
    }

    #[inline]
    pub fn key_str(&self) -> &str {
        self.key.as_str()
    }

    pub const fn value(&self) -> &V {
        &self.val
    }
}

#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct FnvHashMap<V, A: Allocator> {
    buf: Vec<Option<KeyValue<V, A>>, A>,
}

impl<V> Default for FnvHashMap<V, Global> {
    fn default() -> Self {
        Self { buf: Vec::new() }
    }
}

impl<V, A> FnvHashMap<V, A>
where
    A: Allocator,
{
    pub const fn new_in(alloc: A) -> Self {
        Self {
            buf: Vec::new_in(alloc),
        }
    }

    pub const fn len(&self) -> usize {
        self.buf.len()
    }

    pub fn with_size_in(size: usize, alloc: A) -> Self {
        let mut buf = Vec::with_capacity_in(size, alloc);
        buf.resize_with(size, || None);
        Self { buf }
    }

    pub fn insert(&mut self, key: &str, value: V) {
        todo!("Implement FnvHashMap::insert method")
    }

    pub fn delete(&mut self, key: &str) -> V {
        todo!("Implement FnvHashMap::delete method")
    }

    pub fn try_get(&self, key: &str) -> Option<&V> {
        todo!("Implement FnvHashMap::try_get")
    }

    pub fn get(&self, key: &str) -> &V {
        self.try_get(key)
            .expect("Key passed to FnvHashMap::get should exist in FnvHashMap, if you are not sure a key exists in this map, call FnvHashMap::try_get")
    }

    pub fn try_get_mut(&mut self, key: &str) -> Option<&mut V> {
        todo!("Implement FnvHashMap::try_get_mut")
    }

    pub fn get_mut(&mut self, key: &str) -> &mut V {
        self.try_get_mut(key)
            .expect("Key passed to FnvHashMap::get_mut should exist in FnvHashMap, if you are not sure a key exists in this map, call FnvHashMap::try_get_mut")
    }

    pub fn has(&self, key: &str) -> bool {
        todo!("Impement FnvHashMap::has method")
    }

    pub const fn key_iter<'a>(&'a self) -> KeyIter<'a, A> {
        let buf_ptr = NonNull::new(self.buf.as_ptr() as *mut _).unwrap();
        let buf = NonNull::slice_from_raw_parts(buf_ptr, self.buf.len());
        KeyIter {
            buf,
            bi: 0,
            _pd: PhantomData,
        }
    }

    pub const fn value_iter<'a>(&'a self) -> ValueIter<'a, V, A> {
        let buf_ptr = NonNull::new(self.buf.as_ptr() as *mut _).unwrap();
        let buf = NonNull::slice_from_raw_parts(buf_ptr, self.buf.len());
        ValueIter {
            buf,
            bi: 0,
            _pd: PhantomData,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct KeyIter<'a, A: Allocator> {
    buf: NonNull<[Option<KeyValue<(), A>>]>,
    bi: usize,
    _pd: PhantomData<&'a [&'a str]>,
}

impl<'a, A> Iterator for KeyIter<'a, A>
where
    A: Allocator + 'a,
{
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        let buf = self.buf.cast::<Option<KeyValue<(), A>>>();
        // skip through empty elements
        let kv = loop {
            if self.bi >= self.buf.len() {
                return None;
            }
            let ptr = unsafe { buf.add(self.bi) };

            self.bi += 1;

            let Some(kv) = (unsafe { ptr.as_ref() }) else {
                continue;
            };

            break kv;
        };
        Some(kv.key.as_str())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ValueIter<'a, V, A: Allocator> {
    buf: NonNull<[Option<KeyValue<V, A>>]>,
    bi: usize,
    _pd: PhantomData<&'a [&'a V]>,
}

impl<'a, V, A> Iterator for ValueIter<'a, V, A>
where
    A: Allocator + 'a,
{
    type Item = &'a V;

    fn next(&mut self) -> Option<Self::Item> {
        let len = self.buf.len();

        let buf = self.buf.cast::<Option<KeyValue<V, A>>>();
        // skip past empty elements
        let kv = loop {
            if self.bi >= len {
                return None;
            }

            let ptr = unsafe { buf.add(self.bi) };

            self.bi += 1;
            let Some(kv) = (unsafe { ptr.as_ref() }) else {
                continue;
            };

            break kv;
        };

        Some(&kv.val)
    }
}

impl<V> FnvHashMap<V, Global> {
    pub const fn new() -> Self {
        Self { buf: Vec::new() }
    }
}
