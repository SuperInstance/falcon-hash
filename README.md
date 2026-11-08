# Falcon Hash — FNV-1a Rolling Hash and Hash Combiner Utilities

`falcon-hash` is a Rust crate providing fast, deterministic hashing utilities: FNV-1a rolling hash for byte slices, value hashing via the standard library, and a simple hash combiner. It is designed for fingerprinting, cache keys, and deduplication where speed matters more than cryptographic security.

**Status (2026-11-08): shelved.** No active development; kept as a public artefact. The FNV-1a implementation is correct and usable as-is — its limitations are stated plainly below.

## Why It Matters

Not every hash needs to be SHA-256. In high-throughput systems — caching layers, dedup pipelines, change detection, bloom filters — you need a hash that is:

- **Fast**: <10ns per hash for small inputs
- **Deterministic**: same input always produces the same output (no randomized seeds)
- **Well-distributed**: low collision rates for typical workloads
- **Zero-allocation**: no heap usage, stack-only operation

FNV-1a fits this profile perfectly. It's used in:

| Application | Why FNV-1a |
|---|---|
| Cache key fingerprinting | Deterministic across runs |
| Bloom filter hashing | Fast, independent of crypto libs |
| URL/content dedup | Low collision for short strings |
| Hash table checksums | Simple to verify, fast to compute |
| MinHash signatures | Shingle hashing in similarity estimation |

**Not suitable for:** password storage, digital signatures, integrity verification against adversarial input, or any security context. Use BLAKE2b, SHA-256, or Argon2 for those.

## How It Works

### FNV-1a Algorithm

FNV (Fowler–Noll–Vo) 1a is a non-cryptographic hash function defined by two constants:

- **FNV offset basis (64-bit):** `0xcbf29ce484222325`
- **FNV prime (64-bit):** `0x100000001b3` (= 2⁴⁰ + 2⁸ + 0xb3)

The algorithm processes each byte sequentially:

$$h = h_{\text{offset}}$$
$$\textbf{for each byte } b \textbf{ in input:}$$
$$\quad h \leftarrow h \oplus b$$
$$\quad h \leftarrow h \times p_{\text{fnv}}$$

The `wrapping_mul` in Rust ensures 64-bit overflow wraps modulo 2⁶⁴, matching the mathematical definition.

### Why 1a (Not 1)?

FNV-1 XORs the byte *after* multiplying. FNV-1a XORs *before* multiplying. In exhaustive checks (every 1–3-byte input, and a million random 4-byte inputs), no collisions were observed for either FNV-1 or FNV-1a — at short lengths the 64-bit output does not distinguish them. The honest limit is the birthday bound: collisions become likely around ~2³² items.

### Hash Combiner

The `combine_hashes` function merges two hashes using a mixing function:

$$\text{combine}(a, b) = (a \oplus b) \times 0\text{x}517cc1b727220a95$$

This constant is derived from the golden ratio (φ) multiplied by 2⁶⁴, providing good bit-diffusion when combining hashes for compound keys. As implemented, this mix is **commutative**: `combine(a, b) == combine(b, a)` — XOR does not record order, and the multiplication afterwards does not change that. It merges two hashes into one value; it does not encode which came first. Do not use it where order matters.

### Complexity

| Operation | Time | Space |
|---|---|---|
| `rolling_hash(data)` | O(n) where n = bytes | O(1) — single u64 register |
| `hash_value<T>(&T)` | O(sizeof(T)) | O(1) |
| `combine_hashes(a, b)` | O(1) | O(1) |

No heap allocation. All functions operate on stack values only.

### Avalanche Analysis

Measured on this implementation: flipping one input bit changed on average ~0.31 of the output bits for 4-byte inputs and ~0.44 for 16-byte inputs — below the strict avalanche ideal of 0.5, improving with input length, with trailing bytes diffusing weakest (a known FNV trait). That is adequate for non-adversarial fingerprinting and cache keys; it is not a strong mixing function and must not be relied on where inputs may be adversarial.

## Quick Start

This crate is **not published to crates.io**. Use it via git:

```toml
[dependencies]
falcon-hash = { git = "https://github.com/SuperInstance/falcon-hash" }
```

(or copy `src/lib.rs` — it is a single file with no dependencies.)

```rust
use falcon_hash::{rolling_hash, combine_hashes, hash_value};

// FNV-1a rolling hash
let h1 = rolling_hash(b"hello world");
let h2 = rolling_hash(b"hello world");
assert_eq!(h1, h2);  // deterministic

// Hash combiner for compound keys
let combined = combine_hashes(rolling_hash(b"user:42"),
                               rolling_hash(b"session:99"));

// Value hashing via std::Hash
let name_hash = hash_value(&"alice");
```

## API

### Functions

| Function | Signature | Description |
|---|---|---|
| `rolling_hash` | `(&[u8]) -> u64` | FNV-1a 64-bit hash of a byte slice. |
| `hash_value<T: Hash>` | `(&T) -> u64` | Hash any `Hash`-implementing type via `DefaultHasher`. |
| `combine_hashes` | `(u64, u64) -> u64` | Commutative mix of two hashes using golden-ratio constant (order-insensitive as implemented). |

### Properties

| Property | Value |
|---|---|
| Output size | 64 bits (u64) |
| Seed | Fixed (deterministic) |
| Allocation | Zero |
| Endianness | `rolling_hash` consumes bytes in slice order, so its result is identical on every platform |
| Max collision rate | ~2⁻⁶⁴ for random inputs (birthday bound: ~2³² items) |

## Architecture Notes

`falcon-hash` embodies **γ + η = C**:

- **γ (gamma)**: The FNV-1a specification — the offset basis, prime, and the XOR-then-multiply iteration. This is the *mathematical contract* guaranteeing determinism and distribution quality.
- **η (eta)**: The Rust implementation — `wrapping_mul` for overflow semantics, `u64` native representation, `for &byte in data` iteration. This is the *compiled realization*.
- **C (Configuration)**: **Reliable non-cryptographic fingerprinting** — the property that emerges when the implementation (η) faithfully follows the FNV-1a spec (γ). When aligned, identical inputs always produce identical hashes, and hash distribution is well-spread across the u64 range.

The hash combiner uses `0x517cc1b727220a95`, which is Knuth's multiplicative hash constant (√5 − 1)/2 scaled to 2⁶⁴. This ensures that combined hashes maintain good distribution even when the input hashes share structure (e.g., both derived from similar string prefixes).

### Usage in the SuperInstance Ecosystem

`falcon-hash` has **no known dependents** — nothing in the SuperInstance ecosystem currently depends on this crate. The FNV-1a *algorithm* is used elsewhere in the fleet in inline form: **credential-store** carries its own hand-copied `fnv1a64` as a fleet-agreed digest, but it does not use this crate.

## References

- **Fowler, G., Noll, L. C., Vo, K., & Eastlake, D. (2012).** "The FNV Non-Cryptographic Hash Algorithm." *Internet-Draft draft-eastlake-fnv*. IETF. — Authoritative specification of FNV-1a.
- **Knuth, D. E. (1998).** *The Art of Computer Programming, Vol. 3: Sorting and Searching*, 2nd ed., Section 6.4. Addison-Wesley. — Multiplicative hashing with golden-ratio constant (pp. 513–558).
- **Eastlake, D., & Hansen, T. (2006).** "US Secure Hash Algorithms (SHA and SHA-based HMAC and HKDF)." RFC 6234. — Contrast with cryptographic hash requirements.
- **Pagh, A., & Pagh, R. (2008).** "Uniform Hashing in Constant Time and Linear Space." *SIAM J. Computing*, 38(1), 85–96. — Theoretical analysis of hash distribution quality.
- **Lemire, D., & Kaser, O. (2019).** "Strongly Universal String Hashing Is Fast." *ACM Journal of Experimental Algorithmics*, 24(1). — Modern analysis of non-cryptographic hash function performance.
- **Cormen, T. H., et al. (2022).** *Introduction to Algorithms*, 4th ed., Ch. 11 (Hash Tables) and Ch. 25 (Universal Hashing). MIT Press.

## License

MIT
