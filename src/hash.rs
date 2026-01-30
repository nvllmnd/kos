pub mod fnv {

    pub const PRIME32: u32 = 0x010001930;
    pub const OFFSET32: u32 = 0x811c9dc5;

    pub const PRIME64: u64 = 0x00000100000001b3;
    pub const OFFSET64: u64 = 0xcbf29ce484222325;

    pub fn hash_string(s: &str) -> u64 {
        hash_bytes(s.as_bytes())
    }

    /// hashes a &[str] into a u64. for 32 bit version, use [hash32_string]
    /// for a non-const version that uses iterators, use [hash_string]
    pub const fn hash_string_ct(s: &str) -> u64 {
        let bs = s.as_bytes();
        hash_bytes_ct(bs)
    }

    pub fn hash_bytes(bytes: &[u8]) -> u64 {
        const fn hasher(acc: u64, c: &u8) -> u64 {
            let v = acc ^ (*c) as u64;
            v * PRIME64
        }

        if bytes.is_empty() {
            return 0;
        }

        bytes.iter().fold(OFFSET64, hasher)
    }

    /// same as [hash_bytes], but calculates hash at compile time
    pub const fn hash_bytes_ct(bytes: &[u8]) -> u64 {
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
                hash = hash * PRIME64;
                i = i + 1;
            } else {
                break hash;
            }
        }
    }
}
