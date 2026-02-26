//! A simple string type, wrapping Vec<u8, A> where A: Allocator
//! in order to wrap our MiMallocator and Allocator impls in this crate
//!

use core::{
    fmt::{Display, Write},
    hash::Hash,
    ops::{Add, AddAssign, Deref, DerefMut},
    str::Utf8Error,
};

use alloc::{
    alloc::{Allocator, Global},
    string::String,
    vec::Vec,
};

/// A simple String type that works as a drop-in replacement for std::String that is
/// Generic over the Allocator trait.
///
///
///
///
#[repr(transparent)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KString<A: Allocator = Global> {
    buf: Vec<u8, A>,
}

impl From<&str> for KString<Global> {
    fn from(value: &str) -> Self {
        Self::from_vec(value.as_bytes().to_vec())
    }
}

impl<A> KString<A>
where
    A: Allocator,
{
    /// See [Vec::reserve] @param(1) should be in bytes
    pub fn reserve(&mut self, additional: usize) {
        self.buf.reserve(additional);
    }

    pub const fn len(&self) -> usize {
        self.buf.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }

    pub const fn new_in(alloc: A) -> Self {
        Self {
            buf: Vec::new_in(alloc),
        }
    }

    pub fn allocator(&self) -> &A {
        self.buf.allocator()
    }

    pub fn concat(lhs: Self, rhs: Self) -> Self
    where
        A: Clone,
    {
        let alloc = lhs.allocator();

        let mut res = Self::with_capacity_in(lhs.len() + rhs.len(), alloc.clone());
        res.push(lhs.utf8());
        res.push(rhs.utf8());
        res
    }

    pub fn from_str_in(s: &str, alloc: A) -> Self {
        let buf = s.as_bytes().to_vec_in(alloc);

        Self { buf }
    }

    #[inline]
    pub fn from_bytes_in(bytes: &[u8], alloc: A) -> Self {
        Self::from_vec(bytes.to_vec_in(alloc))
    }

    pub const fn from_vec(vec: Vec<u8, A>) -> Self {
        Self { buf: vec }
    }

    #[inline(always)]
    pub fn with_capacity_in(cap: usize, alloc: A) -> Self {
        Self {
            buf: Vec::with_capacity_in(cap, alloc),
        }
    }

    pub const fn try_utf8(&self) -> Result<&str, Utf8Error> {
        core::str::from_utf8(self.as_bytes())
    }

    #[inline]
    pub const fn utf8(&self) -> &str {
        let Ok(res) = self.try_utf8() else {
            panic!("bytes in KString should be utf8 compatible in order to call KString::utf8()")
        };
        res
    }

    pub const fn try_utf8_mut(&mut self) -> Result<&mut str, Utf8Error> {
        core::str::from_utf8_mut(self.as_bytes_mut())
    }

    pub const fn utf8_mut(&mut self) -> &mut str {
        let Ok(res) = self.try_utf8_mut() else {
            panic!(
                "bytes in KString should be utf8 compatible in order to call KString::utf8_mut()"
            )
        };
        res
    }

    pub const fn as_bytes(&self) -> &[u8] {
        self.buf.as_slice()
    }

    pub const fn as_bytes_mut(&mut self) -> &mut [u8] {
        self.buf.as_mut_slice()
    }

    #[inline]
    pub fn push_char(&mut self, c: char) {
        self.buf.push(c as u8)
    }

    /// Intended to behave exactly as standard library method: [String::push_str]
    pub fn push(&mut self, string: &str) {
        let bytes = string.as_bytes();
        self.buf.extend_from_slice(bytes);
    }

    /// Same as [KString::push], but also appends parameter delim immediately after
    /// pushing string
    pub fn push_delim(&mut self, string: &str, delim: char) {
        self.push(string);
        self.push_char(delim);
    }

    #[inline]
    pub fn append_string(&mut self, other: KString) {
        self.push(other.as_str());
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        core::str::from_utf8(self.buf.as_ref()).expect("Strings must be UTF-8!")
    }

    #[inline]
    pub fn as_str_mut(&mut self) -> &mut str {
        core::str::from_utf8_mut(self.buf.as_mut()).expect("Strings should be UTF-8!")
    }

    #[inline]
    pub fn append(&mut self, other: Self) {
        self.push(other.utf8());
    }
}

impl KString {
    #[inline]
    pub fn from_string(string: KString) -> Self {
        Self {
            buf: Vec::from(string.as_bytes()),
        }
    }
}

impl KString {
    pub const fn new() -> Self {
        Self::new_in(Global)
    }

    #[inline(always)]
    pub fn with_capacity(cap: usize) -> Self {
        Self::with_capacity_in(cap, Global)
    }
}

impl Default for KString {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}

impl<A> Display for KString<A>
where
    A: Allocator,
{
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.utf8())
    }
}

impl<A> Deref for KString<A>
where
    A: Allocator,
{
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.utf8()
    }
}

impl<A> DerefMut for KString<A>
where
    A: Allocator,
{
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.utf8_mut()
    }
}

impl<A> PartialEq<str> for KString<A>
where
    A: Allocator,
{
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self.utf8() == other
    }
}

impl<A> PartialOrd<str> for KString<A>
where
    A: Allocator,
{
    #[inline]
    fn partial_cmp(&self, other: &str) -> Option<core::cmp::Ordering> {
        let left: &str = self.as_ref();
        left.partial_cmp(other)
    }
}

impl<A> Add for KString<A>
where
    A: Allocator + Clone,
{
    type Output = KString<A>;

    fn add(self, rhs: Self) -> Self::Output {
        Self::concat(self, rhs)
    }
}

impl<A> AddAssign for KString<A>
where
    A: Allocator,
{
    fn add_assign(&mut self, rhs: Self) {
        self.append(rhs);
    }
}

impl<A> Write for KString<A>
where
    A: Allocator,
{
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.push(s);
        Ok(())
    }
}

impl From<KString<Global>> for String {
    fn from(value: KString<Global>) -> Self {
        String::from_utf8(value.buf)
            .expect("core::string::String should be created from a utf8-compatible KString!!")
    }
}

impl From<String> for KString<Global> {
    fn from(value: String) -> Self {
        Self {
            buf: value.into_bytes(),
        }
    }
}
