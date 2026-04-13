//! This module provides some functions to calculate the FNV-1a hash for
//! 32 and 64 bit numbers intended to be used by [crate] crate internally

pub mod fnv {
    use core::hash::Hasher;

    pub const PRIME32: u32 = 0x010001930;
    pub const OFFSET32: u32 = 0x811c9dc5;

    pub const PRIME64: u64 = 0x00000100000001b3;
    pub const OFFSET64: u64 = 0xcbf29ce484222325;

    pub fn hash_string(s: &str) -> u64 {
        hash_bytes(s.as_bytes())
    }

    const fn hasher(acc: u64, c: &u8) -> u64 {
        let v = acc ^ (*c) as u64;
        v.wrapping_mul(PRIME64)
    }
    /// hashes a &[str] into a u64. for 32 bit version, use [hash32_string]
    /// for a non-const version that uses iterators, use [hash_string]
    pub const fn hash_string_const(s: &str) -> u64 {
        let bs = s.as_bytes();
        hash_bytes_const(bs)
    }

    pub fn hash_bytes(bytes: &[u8]) -> u64 {
        if bytes.is_empty() {
            return 0;
        }

        bytes.iter().fold(OFFSET64, hasher)
    }

    /// same as [hash_bytes], but const
    pub const fn hash_bytes_const(bytes: &[u8]) -> u64 {
        if bytes.is_empty() {
            return 0;
        }
        let mut hash = OFFSET64;
        let len = bytes.len();

        // NOTE: we use a bare loop here so we can hash strings in a const context at compile time
        let mut i = 0;
        loop {
            if i < len {
                let c = bytes[i] as u64;
                hash = hash ^ c;
                hash = hash.wrapping_mul(PRIME64);
                i += 1;
            } else {
                break hash;
            }
        }
    }

    struct FnvHash(u64);

    impl FnvHash {
        pub const fn new() -> Self {
            Self(OFFSET64)
        }

        pub fn hash_bytes(&mut self, bytes: &[u8]) {
            self.0 = bytes.iter().fold(self.0, hasher);
        }

        #[inline]
        pub fn hash_str(&mut self, s: &str) {
            self.hash_bytes(s.as_bytes());
        }
    }

    impl Hasher for FnvHash {
        fn finish(&self) -> u64 {
            self.0
        }

        fn write(&mut self, bytes: &[u8]) {
            self.hash_bytes(bytes);
        }
    }
}
