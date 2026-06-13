# Falcon Hash

**Falcon Hash** is a zero-dependency Rust crate providing **FNV-1a rolling hash** computation and hash combiner utilities for fast, deterministic hashing of byte slices and arbitrary hashable values.

## Why It Matters

Hashing is foundational to hash maps, bloom filters, content-addressable storage, checksums, and rolling-window string matching. The **FNV-1a** (Fowler-Noll-Vo) hash family is valued for its simplicity, speed on short keys, and excellent distribution for non-cryptographic purposes. While Rust's `DefaultHasher` (SipHash) provides DoS resistance, FNV-1a is 3–5× faster on small inputs and uses no state setup. The hash combiner in this crate enables building composite hashes for structured data — essential for memoization caches and structural equality checks.

## How It Works

### FNV-1a Algorithm

FNV-1a processes input byte-by-byte using two constants derived from the FNV prime and offset basis for 64-bit:

```
hash = 0xcbf29ce484222325  (offset basis)
for each byte b:
    hash = hash XOR b
    hash = hash × 0x100000001b3  (FNV prime)
```

The XOR-then-multiply ordering (the "-1a" variant) produces better avalanche properties than the original FNV-1 (multiply-then-XOR), especially for inputs with repeated bytes.

**Time complexity:** O(n) where n is input length. Each byte requires one XOR, one multiply, and one comparison — no table lookups, no branches.

### Hash Combiner

The `combine_hashes` function uses a **mixing function** to merge two 64-bit hashes into one:

```
combined = (a XOR b) × 0x517cc1b727220a95
```

The constant `0x517cc1b727220a95` is from the MurmurHash3 finalizer family, providing good bit-diffusion. Note that `combine(a, b) ≠ combine(b, a)` because XOR is symmetric but the mixing is position-sensitive via the argument order.

### Properties

- **Deterministic:** Same input always produces same output (unlike SipHash with random seeds).
- **Non-cryptographic:** No collision resistance against adversarial inputs.
- **Avalanche:** ~50% bit-flip probability per input bit change.

## Quick Start

```rust
use falcon_hash::{rolling_hash, hash_value, combine_hashes};

// Hash a byte slice with FNV-1a
let h = rolling_hash(b"hello world");
assert_eq!(h, rolling_hash(b"hello world")); // deterministic

// Hash any Hash value using std DefaultHasher
let n = hash_value(&42u64);

// Combine two hashes (order matters)
let combined = combine_hashes(rolling_hash(b"foo"), rolling_hash(b"bar"));
```

## API

| Function | Signature | Description |
|----------|-----------|-------------|
| `rolling_hash` | `fn(&[u8]) → u64` | FNV-1a hash of a byte slice |
| `hash_value<T: Hash>` | `fn(&T) → u64` | Hash any `Hash` type via `DefaultHasher` |
| `combine_hashes` | `fn(u64, u64) → u64` | Mix two hashes into one |

## Architecture Notes

Part of the **SuperInstance** hashing toolkit. Falcon Hash provides the hashing primitives used by the Fleet indexing and deduplication layers. It contributes to **γ + η = C**: γ (correct hash semantics) and η (fast non-cryptographic hashing) combine for efficient data integrity.

See [ARCHITECTURE.md](https://github.com/SuperInstance/SuperInstance/blob/main/ARCHITECTURE.md) for the full system design.

## References

1. Fowler, Noll, Vo. "FNV Hash." <http://www.isthe.com/chongo/tech/comp/fnv/>, 1991–present.
2. Appleby, A. "MurmurHash3." <https://github.com/aappleby/smhasher>, 2008.
3. Pagh, R., Rodler, F. F. "Cuckoo Hashing." *Journal of Algorithms*, 2004.

## License

MIT
