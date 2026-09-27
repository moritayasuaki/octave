# Octave v0.3 — protocol specification

Octave is a 4-of-8 relay-assisted key-establishment library with a native Rust
implementation and a checked Lean mathematical core. Its intended authenticated-
encryption dependency is [ChaCha20-Poly1305-PSIV](docs/PSIV.md).

## 1. Scope

This specification defines sharing, labelled receipt, candidate enumeration,
reconstruction and unique-confirmed-value selection. The Rust crate provides canonical
share payload codecs, cryptographic RNG integration and a typed PSIV backend boundary.
The Lean core proves algebraic facts and conditional protocol theorems.

A concrete PSIV backend, networking, link provisioning, replay/nonce state machines,
canonical transcript encoding, KDF and confirmation exchange must be supplied by the
integration. PSIV's construction and internal state layout are defined by its own
specification and remain unchanged by Octave.

## 2. Normative parameters and trust boundary

| Parameter | Value | Meaning |
| --- | ---: | --- |
| n | 8 | Fixed, distinct relay identities in the session roster |
| k | 4 | Distinct valid shares needed for interpolation |
| t | 3 | Maximum colluding/corrupt relay identities |
| f | 1 | Additional honest relay allowed to be unavailable |
| Maximum subsets | 70 | `choose(8,4)` before filtering missing slots |
| Root | 32 bytes | Uniformly random bootstrap material, prior to session KDF |
| Sharing field | GF(257) | 32 independent coordinate polynomials |

The inequalities are `t < k` and `k + t + f = n`. For corrupt set C and unavailable
set O, let H be the roster minus `C ∪ O`. If `|C| ≤ 3` and `|O| ≤ 1`, then `|H| ≥ 4`.
The proof also permits overlap between C and O. A corrupt relay that drops its share is
already covered by C; the extra outage budget applies to otherwise honest delivery.

Honest delivery means every slot in H contains exactly Alice's generated share for the
fixed session. This is an explicit hypothesis, not something deduced from a slot's presence.
Up to three corrupt slots can contain arbitrary values or be absent. The adversary may
deny confirmation, in which case this protocol can fail. No unconditional liveness claim is made.

Alice and Bob have separately provisioned authenticated symmetric links to each relay.
An honest relay obtains `origin = Alice` from its authenticated incoming link state, never
from an unauthenticated claimed-origin field. Bob assigns slots using the authenticated
relay identity and fixed roster. Different claimed labels do not create additional identities.

## 3. PSIV dependency contract

| Item | Required existing interface |
| --- | --- |
| Key | 32 bytes |
| External nonce | 12 bytes |
| Authentication tag | 16 bytes |
| Encoded record | Ciphertext followed by tag |
| Plaintext maximum | 65,536 bytes per record |
| Associated data maximum | 65,536 bytes per record |
| Session operations | Initialize cached key setup, seal, open, clear |

The nonce and associated data are passed separately, not embedded in the record.
The context caches the dependency's key setup and is reusable across records.
Decryption releases plaintext only after authentication succeeds. Authentication failure
must not expose candidate plaintext through the API.

`lean/Relay/PSIV.lean` records these sizes and a semantic session API.
`src/psiv.rs` exposes the corresponding `Backend` trait and `Session` wrapper. These
are adapter contracts; they do not supply a cipher implementation or an FFI binding.

The [PSIV byte specification](https://github.com/moritayasuaki/psiv/blob/970d15e40add32c041a7dd8ffc6681a3b899ddaf/docs/SPEC.md)
is authoritative for state packing, domain constants, padding and tag-counter layout.
Use the specified 12-byte nonce and construction. Ordinary ChaCha20-Poly1305 and
XChaCha20-Poly1305 are not compatible substitutes.

## 4. Secret sharing

Let F = Z/257Z. Lean proves 257 prime; therefore this is a field. Its small size is suitable
because the entire root is a vector of 32 independently shared coordinates. It does not
restrict the root to 257 possible values.

For each byte position b in 0..31, encode the root byte as `s_b ∈ {0,...,255} ⊂ F`.
Sample `a_b,1`, `a_b,2`, `a_b,3` independently and uniformly from **all 257 field values**.
All 96 samples are mutually independent, independent of the root, and fresh per session.
Do not reuse coefficients across byte positions, roots or sessions. Do not require the
leading coefficient to be nonzero: the intended degree bound is at most three.

Define

```
p_b(X) = s_b + a_b,1 X + a_b,2 X² + a_b,3 X³.
x_i = i + 1, for internal label i ∈ Fin 8.
share_i[b] = p_b(x_i).
```

The eight points 1..8 are nonzero and distinct in F. The external labels are 1..8;
Lean labels are 0..7. The mapping is fixed by the profile, not supplied by packets.

`share` takes the randomness explicitly. The Lean core does not implement a random source. The Rust `split` API samples
coefficients from `rand_core::TryCryptoRng`, with an optional system RNG adapter.
An RNG adapter must use unbiased sampling over 0..256 (for example, reject 16-bit samples
≥65,535, then reduce modulo 257). Uniform 8-bit samples are insufficient. All deterministic
coefficients in the executable are public test fixtures.

## 5. Labelled representation and encoding boundary

```
Secret     = Fin 32 → F             -- includes raw, potentially invalid candidates
RootBytes  = Fin 32 → Fin 256       -- exactly 32 bytes
Randomness = Fin 32 → Fin 3 → F
Shares     = Fin 8 → Secret
Received   = Fin 8 → Option Secret
```

There is exactly one slot per relay identity. `none` means unavailable. All slots passed
to recovery must belong to the same immutable session, peers, version, profile and roster.
An ingress adapter must authenticate those bindings before filling a slot. A duplicate
delivery never supplies an additional point; conflicting deliveries for one slot must
cause a session error or make that slot unavailable under an explicitly chosen ingress
policy. Rust `ReceivedShares::insert` checks the supplied authenticated label against the share
and rejects duplicate insertions without changing the existing slot. Authenticating the
link or session itself is not implemented or proved here.

For the share-payload codec, each of the 32 field elements is a canonical unsigned
16-bit little-endian value 0..256, in byte-position order: 64 bytes of share data.
Reject other lengths and element encodings 257..65,535; do not reduce malformed inputs
modulo 257. Label and session headers are authenticated separately. The Rust library implements the
64-byte payload codec and a 65-byte convenience form (one label byte followed by the
payload). These formats are not authenticated transport envelopes.

Root encoding is injective into F³². On reconstruction, a field value 256 in any coordinate
is an invalid byte-root candidate. `decodeRoot` rejects it. `recoverRoot`, the public
byte-root boundary, rejects invalid roots before invoking the caller's confirmation
predicate. There is no truncation to eight bits and no modulo-256 conversion.

## 6. Enumeration and reconstruction

For a fixed snapshot of received slots, enumerate every four-element subset of available
labels exactly once in a deterministic order. No unavailable slot can belong to a candidate
subset. With m available labels, exactly `choose(m,4)` subsets are considered, at most 70.
With fewer than four available labels the list is empty and recovery fails.

For each subset U and byte coordinate b, compute

```
c_U[b] = sum over i in U of
           received_i[b] * product over j in U, j ≠ i of
             (0 - x_j) / (x_i - x_j).
```

Distinct labels ensure nonzero denominators. No division-by-zero convention is relied on.
The scalar implementation uses inverse times numerator, which is the same field expression.

Every subset gives one raw candidate, even if some of its shares are corrupt. Interpolation
success, agreement counts, majority voting, and being the first candidate are not acceptance
conditions. Several subsets can reconstruct the same secret. Candidate uniqueness is
uniqueness of the **secret value**, not of the subset or polynomial.

The Lean implementation enumerates a fixed 70-element list and filters by availability,
then reconstructs coordinatewise. Rust enumerates ascending four-tuples of available
labels; the subset set is the same. Candidate order is local to each implementation and
not a protocol acceptance condition. Lean proves that list equals the finite set of all 4-subsets,
has length 70 and has no duplicates. It proves the actual reconstructed candidate list
contains the true root under the honest-delivery and fault-bound hypotheses.

## 7. Unique-confirmed-candidate semantics

Freeze one transcript and its confirmation evidence for the selection call. Let
`confirms : Candidate → Bool` be a pure deterministic predicate for that evidence.
For the raw core, define `V = {c in candidates | confirms(c) = true}` as a set of values.

```
V empty                    → failure
V = {s}                    → accept s
V has ≥2 distinct values   → failure
```

`selectUnique` filters the list, then checks that every accepted occurrence equals the
first one. It does not return early on the first successful cryptographic verification.
`recoverRoot` additionally applies canonical byte-root validation as described above.
A timeout, missing evidence or failed verification supplies no confirmation. Rust verifies
each distinct canonical root once; verifier errors fail closed and return no root.
All successful predicate results are considered before returning a unique root.

Confirmation must be bound through a canonical encoding to protocol version, PSIV suite,
this 4-of-8 field profile, fresh session identifier, ordered peer identities and roles,
ordered relay roster, direction, and responder challenge. Separate KDF labels derive
confirmation A→B, confirmation B→A, traffic A→B and traffic B→A keys. Derived PSIV keys
are 32 bytes. The root is input material for the existing protocol's contextual KDF, not
an instruction to alter PSIV's internal setup or add a per-record KDF inside PSIV.

The candidate-dependent KDF and actual confirmation exchange are deliberately abstract.
An adapter must bind a fixed transcript before candidate checks, enforce record/nonce
limits, and avoid a circular design in which each candidate is permitted to manufacture
its own apparently valid confirmation. Directional confirmations must not reflect into
one another. The opaque predicate and typed context alone do not implement these checks.

### Conditional guarantees

For true root S, the explicit soundness hypothesis is

```
for every c in this candidate list, confirms(c) = true implies c = S.
```

Lean proves that any returned value equals S under this hypothesis. Separately, if S is
in the candidate list and `confirms(S) = true`, soundness implies successful selection of S.
With ≤3 corrupt relays, ≤1 additional outage and honest delivery, S is in the list.

These are deterministic implications. Proving that the PSIV/KDF confirmation adapter
satisfies the hypothesis with negligible failure probability is a separate cryptographic
reduction. Ordinary AEAD correctness or ciphertext integrity alone does not establish
cross-candidate, cross-key or transcript-bound confirmation soundness. This release gives
no unconditional `70 / 2^128` bound, no proof of PSIV key commitment, and no post-quantum
security theorem. A probabilistic bound requires the actual adapter and its security model.

Local successful selection is necessary for eventual session establishment. Full mutual
confirmation and the protocol's replay/nonce state machines still have to be implemented
before treating a networking session as established.

## 8. Algebra and privacy boundary

Over any field, two polynomials of degree at most three agreeing at four distinct points
are equal. The executable Lagrange expression equals interpolation evaluated at zero.
Consequently any all-honest four-subset reconstructs every coordinate of the original root.

For at most three observed nonzero points, Lean constructs a degree-at-most-three mask M
with `M(0)=1` and `M(x_i)=0` at every observed point. Translating a compatible polynomial by
`(s_1 - s_0) M` gives a bijection between the polynomials compatible with the same view and
secret s_0 or s_1. These fibers are finite when the field is finite and have equal cardinality.
This is the proved privacy threshold structure; it covers sets of size 0, 1, 2 and 3.

With independent uniform coefficient sampling, the bijection is the usual counting
argument for perfect privacy of Shamir shares. Restricting the root to the byte subset
does not affect it: the bijection holds for any two secrets. Applying it independently
across 32 coordinates explains privacy of the byte vector. This release does not formalize
a probability distribution, adaptive corruption game, randomness implementation or network
transcript leakage. It does not claim information-theoretic privacy for public confirmation
evidence, encrypted link traffic, or the eventual entire protocol.

Four valid shares determine the secret; three-share privacy never protects against four
colluding relays. Reusing masks across coordinates or sessions invalidates the stated
sampling condition. There is no robust Reed–Solomon correction of three errors asserted:
the threshold improvement here relies on the explicitly assumed confirmation property.

## 9. Verification scope

The project pins Lean 4.32.1 and Mathlib v4.32.1 (full revisions in the lock file).
See [verification](docs/VERIFICATION.md) for checked theorem names, test results, trust
assumptions and reproducible commands. No concrete PSIV cipher implementation is included
in this crate; the dependency's existing state layout remains untouched.

## 10. Rust library and refinement status

The `octave` Cargo crate is the native application library. See [API guide](docs/API.md)
for public types, error handling, byte formats, RNG requirements, and PSIV adapter use.
It runs without the Lean runtime. The checked-in fixtures are generated directly by
`lean/VectorMain.lean` and compared with Rust in `tests/lean_vectors.rs`, including all 280
worst-case fault placements. Differential testing is not a formal proof of Rust/Lean
equivalence. No PSIV state packing was changed or reimplemented.
