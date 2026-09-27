# Octave Rust library API

The `octave` crate is a native Rust implementation of the specified sharing and recovery
layer. `src/` contains the implementation; the `lean/Relay/` modules provide
the mathematical reference. Rust is linked and used through Cargo; there is no Lean
runtime, generated-C dependency or C ABI in this crate.

## Roots and coefficient generation

`RootSecret` holds exactly 32 bytes. `from_bytes([u8;32])` is an explicit import;
`TryFrom<&[u8]>` rejects wrong lengths. `random(&mut rng)` fills all 32 bytes from
`rand_core` 0.10's `TryCryptoRng`. With the default `os-rng` feature, `SysRng` is the
`getrandom` system-randomness adapter.

`split(&root, &mut rng)` returns eight `Share` values. It generates 96 independent field
coefficients: byte positions 0..31, then powers 1, 2, 3. Each sample draws a little-endian
16-bit word, rejects 65,535, and reduces the accepted word modulo 257. Every field value
has exactly 255 accepted preimages, including 0 and 256. After 128 consecutive rejections
for one coefficient the call fails; no partially generated share array is returned.
RNG errors propagate through `RandomError::Source`.

The RNG trait is a review aid, not proof of a generator's security. Applications must
use a correctly seeded cryptographic source. `Coefficients::from_bytes` and
`split_with_coefficients` are advanced deterministic interfaces: correct encoding does
not establish fresh, independent, uniform sampling. Never reuse the 96 coefficients.
There is no requirement that a polynomial's leading coefficient be nonzero.

## Canonical byte formats

| Object | Encoding |
| --- | --- |
| Root | Exactly 32 bytes |
| Coefficients | Exactly 192 bytes: 96 canonical u16 little-endian values 0..=256 |
| Share payload | Exactly 64 bytes: 32 canonical u16 little-endian values 0..=256 |
| Labelled share convenience format | Exactly 65 bytes: label 1..=8, then 64-byte payload |

`Share::from_payload` takes a validated `RelayId`; `Share::from_bytes` also decodes the
one-byte label. Both reject wrong lengths or noncanonical elements rather than reducing
invalid inputs modulo 257. A field value 256 is valid share data but invalid in a recovered
byte-root candidate. Such candidates are rejected before confirmation callbacks run.

These encodings are payload formats, not authenticated transport envelopes. They contain
no session ID, peer identity, nonce, PSIV record or transcript. The application must
authenticate and validate those bindings before inserting a share.

## Receipt and recovery

`ReceivedShares::insert(authenticated_relay, share)` checks the claimed label against the
separate trusted identity. It rejects repeated labels, even if payloads match, and leaves
the existing slot unchanged. The caller must abort or handle such errors explicitly;
an ignored insertion error does not poison the collection. Each collection belongs to
one session, version, profile, ordered roster and pair of peers; the type cannot prove
the provenance of values supplied by an application.

`candidates()` snapshots the received slots. It visits each ascending four-tuple of
available labels once, uses Lagrange interpolation at zero, rejects non-byte roots, and
deduplicates equal roots. It exposes:

- `subset_count()`: exactly choose(m,4), at most 70.
- `invalid_byte_subsets()`: count of subsets producing a field value 256.
- `iter()` / `len()`: distinct canonical roots, which remain **unconfirmed**.

With fewer than four slots, `candidates()` is empty. `recover()` instead returns
`ConfirmationError::InsufficientShares`. `reconstruct_candidate(&[Share;4])` requires
four distinct labels and returns an unconfirmed root or decoding error; it provides no
authentication. There is no majority or agreement-count acceptance rule.

`CandidateSet::confirm` and `ReceivedShares::recover` accept a callback with this shape:

```rust
Fn(&RootSecret) -> Result<bool, VerificationError>
```

The callback should borrow fixed confirmation evidence and a fixed transcript. It must
derive the appropriate confirmation key and verify that evidence against each candidate.
Do not create fresh self-authenticating evidence under each candidate or accept because
interpolation succeeded. Bind the session, ordered peer identities/roles, version, suite,
4-of-8 profile, ordered relay roster, direction, and responder challenge. Use separate
KDF domains for directional confirmation and traffic keys.

Each distinct candidate is verified once. If any callback errors, recovery returns
`Verifier(error)` and no root. Otherwise all candidates are checked before success:
zero successes gives `NoConfirmedCandidate`, multiple distinct successes gives
`AmbiguousConfirmation`, and exactly one returns that root. Neither a timeout nor an
early success skips checking other candidates. A network adapter may collect evidence
asynchronously first, then use this synchronous fixed-evidence predicate.

The guarantee remains conditional: every confirmed candidate must equal Alice's real
root. Ordinary AEAD correctness or a 16-byte tag is not a proof of this property. The
local `roundtrip` example uses equality to a known test root and is not an authenticated
network protocol.

## PSIV adapter

`psiv::Backend` preserves `init/seal/open/clear` semantics of the existing PSIV dependency.
`psiv::Session<B>` caches one backend instance, calls `clear` on drop, validates input
limits before invoking it, checks output lengths, and returns opened plaintext in
zeroizing storage. The backend is required to verify authentication before returning
plaintext. The wrapper cannot establish cryptographic correctness of a supplied backend.

Types and constants follow the [PSIV contract](PSIV.md): key `[u8;32]`, nonce `[u8;12]`, tag `[u8;16]`, external nonce/AD, record
`ciphertext || tag`, and message/AD limits of 65,536 bytes. Backend errors are preserved.
Nonce allocation, replay handling, secure backend erasure and the contextual KDF remain
application/backend obligations. No primitive implementation or internal state packing
is introduced by the trait.

The PSIV boundary tests use an explicitly non-cryptographic contract mock. They validate
size/error/cleanup behavior only. The concrete PSIV package is not linked by this crate.

## Memory and platform scope

The crate forbids unsafe code in its own source. Root, coefficient, share and candidate
storage is zeroized on drop, and debug output redacts secrets. Explicit exports return
zeroizing arrays where possible. Copies created by the compiler, caller, stack moves,
operating system or backend are not guaranteed erased. This is not a constant-time or
memory-erasure proof; candidate enumeration, equality counts and allocation can vary.

Default features are `std` and `os-rng`. Disabling defaults gives `no_std + alloc` and
requires caller-provided randomness. Actual target support for `SysRng` follows
`getrandom`; no unsupported backend is silently substituted. `no_std` tests exercise the host target; they do not establish embedded or
WebAssembly support. See [verification](VERIFICATION.md) for coverage.

Dependency API references: [rand_core](https://docs.rs/rand_core/0.10.1/rand_core/),
[zeroize](https://docs.rs/zeroize/1.9.0/zeroize/),
[getrandom](https://docs.rs/getrandom/0.4.3/getrandom/).
