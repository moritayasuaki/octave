# PSIV and Octave

## Construction and sources

ChaCha20-Poly1305-PSIV is the authenticated-encryption dependency for Octave's intended
relay protocol. Its synthetic tag binds the plaintext, associated data, nonce and key;
the tag also determines the encryption stream. Opening a record authenticates the
recovered plaintext before releasing it.

The construction is described by Tim Beyne, Yu Long Chen and Michiel Verbauwhede in
[*A Robust Variant of ChaCha20-Poly1305*](https://eprint.iacr.org/2025/222). The paper
analyzes nonce-misuse resistance in the multi-user faulty-nonce model with an ideal
underlying permutation, and key commitment in the cmt-1 model. These are construction
results under stated assumptions, not a security proof of Octave's confirmation exchange.

The separate [PSIV project](https://github.com/moritayasuaki/psiv) supplies a Rust
implementation and Lean model. Its [byte specification](https://github.com/moritayasuaki/psiv/blob/970d15e40add32c041a7dd8ffc6681a3b899ddaf/docs/SPEC.md)
is the reference for domain constants, state layout, Poly1305 encoding and tag-counter
handling. Octave does not reimplement or alter those internals.

## Contract exposed by Octave

| Item | Contract |
| --- | --- |
| Key | 32 bytes |
| Nonce | 12 bytes, supplied separately |
| Tag | 16 bytes |
| Record | Ciphertext followed by tag |
| Associated data | Authenticated, supplied separately |
| Plaintext / associated data | Each at most 65,536 bytes |
| Operations | Initialize cached key setup, seal, open, clear |

The length caps are implementation limits, not a per-key security budget. Identical
inputs yield identical records. Nonce policy, replay protection and key rotation remain
protocol responsibilities even with nonce-misuse resistance.

The [`psiv` module](../src/psiv.rs) defines `Key`, `Nonce`, `Tag`, `Backend` and
`Session<B>`. A session validates lengths before calling the backend, checks returned
lengths, keeps opened plaintext in zeroizing storage and calls `clear` on drop.
The backend must authenticate before returning plaintext and invalidate its retained
key setup when cleared. These checks cannot make an incorrect cipher implementation secure.

This crate supplies no backend implementation. Its boundary tests use a non-cryptographic
mock only to exercise size checks, errors and cleanup. To encrypt data directly, use
the [PSIV Rust library](https://github.com/moritayasuaki/psiv/tree/main/rust/psiv).

## Confirmation in a relay protocol

An integration derives a confirmation key from each candidate root and a fixed session
transcript, then verifies evidence received for that transcript. Octave's synchronous
callback has the form `Fn(&RootSecret) -> Result<bool, VerificationError>`.

The transcript must bind the version, suite, 4-of-8 profile, session identifier, ordered
peer identities and roles, ordered relay roster, direction and responder challenge.
Use separate KDF domains for confirmation and traffic in each direction. Authenticate
relay identity and session context before inserting a share.

Verify the same evidence for every candidate. Generating new evidence under the candidate
being tested is circular and does not confirm anything. A successful interpolation is
also insufficient. Octave checks every distinct candidate and succeeds only when exactly
one is confirmed; verifier errors fail closed.

The Lean safety theorem assumes that every confirmed candidate equals the sender's true
root. Showing that a concrete PSIV/KDF/transcript adapter satisfies this hypothesis is
separate cryptographic work. A 16-byte tag or a primitive key-commitment result alone does
not establish that composition, a `70 / 2^128` bound or post-quantum security.

See the [normative specification](../SPEC.md#7-unique-confirmed-candidate-semantics),
[API guide](API.md#psiv-adapter) and [verification scope](VERIFICATION.md).
