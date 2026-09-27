# Optional PSIV integration

## Enable the adapter

Octave's sharing and confirmation core works without PSIV. To expose `octave::psiv`:

```toml
[dependencies]
octave = { git = "https://github.com/moritayasuaki/octave.git", features = ["psiv"] }
```

The feature adds no cipher dependency. It exposes the backend trait, types and session
wrapper; applications still supply a concrete implementation. It is also available with
`default-features = false` for `no_std + alloc` integrations.

**Migration from 0.3:** Octave 0.4 requires the `psiv` feature for existing imports of
`octave::psiv`. Core Rust APIs and share encodings are unchanged. In Lean, explicitly
`import Relay.PSIV` to use the adapter contract; `import Relay` exposes the generic core.
The context is now `Relay.ConfirmationContext`, with a compatibility type alias under
`Relay.PSIV` when that module is imported.

## Construction and sources

ChaCha20-Poly1305-PSIV is an optional authenticated-encryption choice for a relay
protocol integration. Its synthetic tag binds the plaintext, associated data, nonce and key;
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

## Confirmation and security scope

PSIV's nonce-misuse resistance and key commitment may be useful in a concrete protocol,
but neither property alone establishes Octave's confirmation-soundness hypothesis.
The optional adapter is not invoked by share generation, reconstruction or candidate
selection. A backend does not install a verifier or implement a handshake.

Follow the [integration contract](INTEGRATION.md) for fixed evidence, transcript binding,
key separation, confidential share transport and the required security argument.
If a chosen profile uses PSIV, preserve the construction and parameters above; ordinary
ChaCha20-Poly1305 and XChaCha20-Poly1305 are not substitutes for that profile.

See the [normative specification](../SPEC.md#3-cryptographic-integration-contract),
[API guide](API.md#optional-psiv-adapter) and [verification scope](VERIFICATION.md).
