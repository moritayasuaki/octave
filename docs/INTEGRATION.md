# Integration contract

Octave's core is independent of encryption, key derivation and authentication algorithms.
It provides a 4-of-8 sharing and candidate-selection mechanism, not a complete handshake.
This document states the obligations of a concrete protocol integration; it defines no
default cryptographic suite or wire format.

## Confidential, authenticated share delivery

Alice and Bob need separately provisioned, confidential and authenticated links to each
relay. An honest relay derives the sender's identity from its authenticated incoming link.
Bob derives the relay identity from his authenticated link and checks the session context
before passing a share to `ReceivedShares::insert`.

Share encodings provide no encryption or authentication. A MAC alone leaves their contents
visible; an observer collecting four honest shares can reconstruct the root. Protect link
confidentiality as well as identity, session and replay bindings. Existing protected
transport may satisfy these requirements without an Octave-specific cipher wrapper.

## Fixed confirmation evidence

`ReceivedShares::recover` and `CandidateSet::confirm` accept:

```rust
Fn(&RootSecret) -> Result<bool, VerificationError>
```

The application collects confirmation evidence, freezes the session transcript and
received-share snapshot, and verifies the same evidence for each candidate. Return
`Ok(true)` only after verification succeeds for that candidate and transcript. A protocol
profile must distinguish an ordinary mismatch (`Ok(false)`) from a processing failure
(`Err`); any error aborts recovery without returning a root.

The callback must not generate new evidence under each candidate and verify that evidence
against itself. It must not accept a candidate solely because interpolation succeeded.
Octave checks all distinct candidates unless an error aborts the call, and succeeds only
when exactly one is confirmed.

Bind at least these fields through an unambiguous canonical encoding:

- Protocol version, cryptographic suite and the fixed 4-of-8 field profile.
- Fresh session identifier and responder challenge.
- Ordered peer identities and roles, and the ordered eight-relay roster.
- Message purpose and direction, with separate key-derivation domains for confirmation
  and traffic in each direction.

The suite identifies the complete algorithm/parameter selection. Bind that selection to
prevent substitution or downgrade. The Lean `Relay.ConfirmationContext` records semantic
fields; it does not implement serialization, validation or cryptographic binding.

## Safety obligation

The Lean selection theorem assumes that every confirmed candidate equals Alice's real
root. Under that hypothesis, a returned root is correct. With the fault bounds, honest
delivery and confirmation of the real root, recovery succeeds. An attacker who withholds
confirmation may still cause failure.

A concrete integration needs an argument connecting its KDF, authentication primitive,
transcript and message flow to this hypothesis. Account for candidates influenced by
malicious shares, verification across multiple candidate keys, replay, reflection and
identity binding. Ordinary MAC unforgeability or AEAD correctness alone is not that
composition proof. Tag size alone does not establish a failure bound.

Local candidate selection is only one stage of a handshake. Define both confirmation
directions and when each endpoint may accept application traffic. Do not use a returned
root as evidence that the peer has completed mutual confirmation.

## Choosing a concrete profile

Confirmation can use a MAC without encrypting the confirmation payload. HKDF with
HMAC-SHA-256 is a candidate worth evaluating: [HKDF](https://www.rfc-editor.org/rfc/rfc5869.html)
supports context-bound key derivation, and [TLS 1.3 Finished](https://www.rfc-editor.org/rfc/rfc8446.html#section-4.4.4)
uses an HKDF-derived key with a transcript HMAC. This precedent does not prove Octave's
composition. No HKDF/HMAC confirmation implementation or approved profile is included here.

Before introducing a concrete profile, specify its canonical transcript and messages,
algorithm identifiers, KDF inputs and labels, authentication output length, freshness and
replay rules, state transitions, error handling and security assumptions. Public vectors
and adversarial tests should cover those choices. Confirmation and share-transport
confidentiality remain separate requirements.

The application selects its cryptographic libraries directly. Octave provides no cipher
adapters and does not prescribe an encryption algorithm.

See the [normative specification](../SPEC.md), [Rust API](API.md) and
[verification scope](VERIFICATION.md).
