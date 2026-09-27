# Octave

**A Rust library for 4-of-8 relay-assisted key establishment, designed to use
ChaCha20-Poly1305-PSIV for authenticated encryption.** Octave shares a 32-byte root
among eight relays, reconstructs possible roots, and accepts exactly one confirmed
value. Its algebraic core also has a checked Lean specification.

## What is PSIV?

**ChaCha20-Poly1305-PSIV** is an authenticated-encryption construction designed for
nonce-misuse resistance and key commitment. It combines the ChaCha20 core and Poly1305
in a synthetic-IV mode. Tim Beyne, Yu Long Chen and Michiel Verbauwhede describe and
analyze it in [*A Robust Variant of ChaCha20-Poly1305*](https://eprint.iacr.org/2025/222).
Their security results apply under the paper's stated models and assumptions.

For each record, PSIV:

1. Hashes the plaintext and associated data with Poly1305, including padding and lengths.
2. Uses the key, nonce and hash to derive a synthetic authentication tag.
3. Encrypts with a ChaCha20-core stream determined by the key, nonce and tag, then
   returns `ciphertext || tag`.

Decryption checks the tag before releasing plaintext. Associated data is authenticated
without being encrypted. The nonce and associated data are supplied separately.
The [PSIV implementation and byte specification](https://github.com/moritayasuaki/psiv)
define the construction, including its state layout.

| PSIV parameter | Value |
| --- | --- |
| Key | 32 bytes |
| Nonce | 12 bytes |
| Authentication tag | 16 bytes |
| Record | Ciphertext followed by tag |
| Plaintext and associated data limits | Each at most 65,536 bytes |

Nonce-misuse resistance is protection against nonce-management failures, not a reason
to reuse nonces. Identical key, nonce, associated data and plaintext produce identical
records. Applications still need replay protection and a key lifecycle. PSIV has its
own state layout and wire format; ordinary ChaCha20-Poly1305 and XChaCha20-Poly1305
are not substitutes. See [PSIV and integration](docs/PSIV.md).

## How Octave uses PSIV

Octave handles the sharing and candidate-selection layer. A protocol integration uses
PSIV for authenticated relay traffic and verifies fixed, transcript-bound confirmation
evidence under a key derived from each candidate root.

- **Eight labelled shares:** each root byte is shared with an independent cubic over GF(257).
- **Four honest shares reconstruct the root:** at most three corrupt relays plus one
  additional honest outage leave at least four honest shares available.
- **At most 70 subsets:** reconstruct all available four-share subsets and deduplicate roots.
- **Exactly one confirmed root:** no matches, multiple matches or a verifier error fail closed.

**Integration status:** this crate provides the `psiv::Backend` contract and a
`psiv::Session` wrapper that validates lengths. It does not link a concrete PSIV implementation. Applications
must supply the backend, contextual KDF, authenticated relay transport and confirmation
exchange. Confirmation soundness is an explicit assumption in the Lean theorems;
the primitive's security analysis does not by itself prove the composed protocol.

## Use from Rust

```toml
[dependencies]
octave = { git = "https://github.com/moritayasuaki/octave.git" }
```

This local example exercises sharing and recovery with a **test oracle**. It is not a
network handshake; a real receiver verifies fixed confirmation evidence instead of
already knowing the root.

```rust
use core::convert::Infallible;
use octave::{ReceivedShares, RootSecret, SysRng, split};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut rng = SysRng;
    let root = RootSecret::random(&mut rng)?;
    let shares = split(&root, &mut rng)?;
    let mut received = ReceivedShares::new();
    for share in shares.into_iter().take(4) {
        // A network integration obtains this identity from authenticated link state.
        received.insert(share.relay_id(), share)?;
    }
    // Test oracle only: replace with transcript-bound PSIV/KDF verification.
    let recovered = received.recover(|candidate| Ok::<_, Infallible>(candidate == &root))?;
    assert_eq!(recovered, root);
    Ok(())
}
```

Run `cargo run --example roundtrip`. Default features provide the system RNG;
`default-features = false` enables `no_std + alloc` with caller-supplied randomness.
See the [API guide](docs/API.md) for encoding, receipt, recovery and error handling.
The crate is available as a Git dependency and is not published on crates.io.

## Specification and verification

- [Protocol specification](SPEC.md): normative `k=4, n=8, t=3, f=1` profile and assumptions.
- [Verification](docs/VERIFICATION.md): theorem index, test coverage and reproduction steps.
- [Contributing](.github/CONTRIBUTING.md): build setup and checks.
- [Security](SECURITY.md): supported scope and reporting.

```sh
cargo test --locked       # Rust tests; no Lean installation needed
sh scripts/check.sh rust # Formatting, tests, linting, documentation and example
```

**Experimental research software.** Lean checks algebraic and conditional protocol
properties. Rust-to-Lean equivalence, compiled constant-time behavior and end-to-end
cryptographic security are not established. This is not a production-ready protocol.

Licensed under the [MIT license](LICENSE).
