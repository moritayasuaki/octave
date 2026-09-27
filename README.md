# Octave

**A cipher-independent Rust library for 4-of-8 relay-assisted key establishment.**
Octave splits a 32-byte root among eight relays, reconstructs candidate roots from
received shares, and returns a root only when exactly one candidate is confirmed.
Its mathematical core has a checked Lean specification.

## What Octave provides

- **Eight labelled shares** using 32 independent polynomials over GF(257).
- **Four-share reconstruction** with at most 70 candidate subsets.
- **Unique confirmation:** no matches, multiple matches or a verifier error fail closed.
- **Canonical byte encodings**, cryptographic randomness integration and zeroizing secret storage.
- **Lean proofs** of reconstruction, threshold structure and conditional selection safety.

The fixed profile is `k=4, n=8, t=3, f=1`. Up to three corrupt relays plus one
additional honest outage leave four honest shares available. Successful recovery also
requires honest delivery and sound, available confirmation evidence.

## Integration boundary

The core chooses no cipher, KDF or MAC. It accepts a callback that checks each candidate
against fixed confirmation evidence for one session transcript. Encrypted, authenticated
relay links and a concrete confirmation exchange are supplied by the application.

| Layer | Responsibility |
| --- | --- |
| Octave | Share generation, labelled receipt, reconstruction and unique selection |
| Relay transport | Confidentiality, authenticated identities, session binding and replay protection |
| Confirmation integration | Key derivation, transcript encoding and verification of received evidence |
| Application traffic | Encryption and key lifecycle after the handshake completes |

See the [integration contract](docs/INTEGRATION.md) before connecting a network protocol.
The provided example uses a test oracle; it does not implement cryptographic confirmation.

## Use from Rust

```toml
[dependencies]
octave = { git = "https://github.com/moritayasuaki/octave.git" }
```

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
    // Test oracle only: a real receiver verifies fixed transcript-bound evidence.
    let recovered = received.recover(|candidate| Ok::<_, Infallible>(candidate == &root))?;
    assert_eq!(recovered, root);
    Ok(())
}
```

Run `cargo run --example roundtrip`. Default features provide the system RNG;
`default-features = false` enables `no_std + alloc` with caller-supplied randomness.
See the [API guide](docs/API.md) for encoding, receipt, recovery and error handling.
The crate is available as a Git dependency and is not published on crates.io.

## Optional PSIV adapter

The `psiv` feature exposes an adapter contract for **ChaCha20-Poly1305-PSIV**, an
authenticated-encryption construction designed for nonce-misuse resistance and key
commitment. Enable it only when integrating that construction:

```toml
octave = { git = "https://github.com/moritayasuaki/octave.git", features = ["psiv"] }
```

The adapter preserves PSIV's 32-byte key, 12-byte nonce and 16-byte tag. It does not
link a concrete backend or provide confirmation automatically. Read [the PSIV guide](docs/PSIV.md)
for the construction, contract and migration from Octave 0.3. Other integrations use
the core callback without this feature.

## Specification and verification

- [Protocol specification](SPEC.md): normative profile and conditional guarantees.
- [Verification](docs/VERIFICATION.md): theorem index, test coverage and reproduction steps.
- [Contributing](.github/CONTRIBUTING.md): build setup and checks.
- [Security](SECURITY.md): scope and reporting.

```sh
cargo test --locked       # Core tests; no Lean installation needed
sh scripts/check.sh rust # Core and optional-feature checks
```

**Experimental research software.** The Lean proofs assume confirmation soundness and
honest delivery. Rust-to-Lean equivalence, compiled constant-time behavior and end-to-end
cryptographic security are not established. This is not a production-ready protocol.

Licensed under the [MIT license](LICENSE).
