use core::{
    fmt::{Display, Write},
    ops::{Deref, DerefMut},
};

use alloc::alloc::{Allocator, Global};

use crate::{buf::kstring::KString, copy_slice_into};

const INLINE_LEN: usize = 23;

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
struct Inline {
    len: u8,
    buf: [u8; INLINE_LEN],
}

impl Inline {
    pub const fn empty() -> Self {
        Self {
            len: 0,
            buf: [0u8; INLINE_LEN],
        }
    }

    pub fn new(s: &str) -> Self {
        // clamp at [INLINE_LEN], so if input s.len() >= [INLINE_LEN], then only [INLINE_LEN]
        // characters ill get initialized into this structure, if you want to fail if input string
        // is larger than [INLINE_LEN], try [Inline::try_new]
        let len = core::cmp::min(s.len(), INLINE_LEN);
        let src = &s[..len];
        let mut buf = [0u8; INLINE_LEN];
        buf.copy_from_slice(src.as_bytes());
        Self {
            len: len as u8,
            buf,
        }
    }

    /// same as [Inline::new], but returns None if input string is larger than [INLINE_LEN]
    pub fn try_new(s: &str) -> Option<Self> {
        if s.len() >= INLINE_LEN {
            None
        } else {
            Some(Self::new(s))
        }
    }

    pub const fn len(&self) -> usize {
        self.len as usize
    }

    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &self.buf[..self.len()]
    }

    #[inline]
    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        let len = self.len();
        &mut self.buf[..len]
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        core::str::from_utf8(&self.buf[..self.len()])
            .expect("Inline::buf should be utf8 compatible to read as string!")
    }

    #[inline]
    pub fn as_str_mut(&mut self) -> &mut str {
        core::str::from_utf8_mut(self.as_bytes_mut())
            .expect("Inline::buf should be utf8 compatible to read as string!")
    }
}

impl Deref for Inline {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl DerefMut for Inline {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_str_mut()
    }
}

#[derive(Debug, Clone)]
enum Inner<A: Allocator> {
    Inline(Inline),
    Heap(KString<A>),
}

impl<A> Inner<A>
where
    A: Allocator,
{
    pub const fn empty() -> Self {
        Self::Inline(Inline::empty())
    }

    pub fn new_in(s: &str, alloc: A) -> Self {
        if let Some(inl) = Inline::try_new(s) {
            Self::Inline(inl)
        } else {
            Self::Heap(KString::from_str_in(s, alloc))
        }
    }

    #[inline]
    pub fn allocator(&self) -> Option<&A> {
        match self {
            Inner::Inline(_) => None,
            Inner::Heap(kstring) => Some(kstring.allocator()),
        }
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        match self {
            Inner::Inline(inline) => inline.as_str(),
            Inner::Heap(kstring) => kstring.as_str(),
        }
    }

    #[inline]
    pub fn as_str_mut(&mut self) -> &mut str {
        match self {
            Inner::Inline(inline) => inline.as_str_mut(),
            Inner::Heap(kstring) => kstring.as_str_mut(),
        }
    }

    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Inner::Inline(inline) => inline.as_bytes(),
            Inner::Heap(kstring) => kstring.as_bytes(),
        }
    }

    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        match self {
            Inner::Inline(inline) => inline.as_bytes_mut(),
            Inner::Heap(kstring) => kstring.as_bytes_mut(),
        }
    }
}

impl Inner<Global> {
    pub fn new(s: &str) -> Self {
        if let Some(inl) = Inline::try_new(s) {
            Self::Inline(inl)
        } else {
            Self::Heap(KString::from(s))
        }
    }
}

impl<A> Deref for Inner<A>
where
    A: Allocator,
{
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl<A> DerefMut for Inner<A>
where
    A: Allocator,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_str_mut()
    }
}

impl<A> Default for Inner<A>
where
    A: Allocator,
{
    fn default() -> Self {
        Self::empty()
    }
}

#[derive(Debug, Clone, Default)]
#[repr(transparent)]
pub struct UString<A: Allocator>(Inner<A>);

impl<A> UString<A>
where
    A: Allocator,
{
    pub const fn empty() -> Self {
        Self(Inner::empty())
    }

    #[inline]
    pub fn new_in(s: &str, alloc: A) -> Self {
        Self(Inner::new_in(s, alloc))
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        Inner::<A>::as_str(&self.0)
    }

    #[inline]
    pub fn as_str_mut(&mut self) -> &mut str {
        Inner::<A>::as_str_mut(&mut self.0)
    }

    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        Inner::<A>::as_bytes(&self.0)
    }

    #[inline]
    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        Inner::<A>::as_bytes_mut(&mut self.0)
    }
}

impl UString<Global> {
    pub fn new(s: &str) -> Self {
        Self(Inner::new(s))
    }
}

impl<A> Deref for UString<A>
where
    A: Allocator,
{
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0.as_str()
    }
}

impl<A> DerefMut for UString<A>
where
    A: Allocator,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.as_str_mut()
    }
}

impl<A> Display for UString<A>
where
    A: Allocator,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
