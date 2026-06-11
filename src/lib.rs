//! Falcon-hash: a simple hash utility crate
//!
//! Provides a basic hash combiner and utility functions.

use std::hash::{Hash, Hasher};

/// Combine two u64 hashes using a mixing function.
pub fn combine_hashes(a: u64, b: u64) -> u64 {
    let mixed = a ^ b;
    mixed.wrapping_mul(0x517cc1b727220a95)
}

/// Hash a single value using the standard library hasher.
pub fn hash_value<T: Hash>(value: &T) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

/// A simple rolling hash for byte slices (FNV-1a style).
pub fn rolling_hash(data: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for &byte in data {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rolling_hash_deterministic() {
        let h1 = rolling_hash(b"hello");
        let h2 = rolling_hash(b"hello");
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_combine_different() {
        let a = rolling_hash(b"foo");
        let b = rolling_hash(b"bar");
        assert_ne!(combine_hashes(a, b), combine_hashes(b, a));
    }
}
