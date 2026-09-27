# Octave

A native Rust library for **4-of-8 relay-assisted key establishment**: share a 32-byte
root, reconstruct candidates, and accept exactly one confirmed root. The fixed profile
is **k=4, n=8, t=3, f=1**, using 32 independent polynomials over GF(257).

## Use

Add Octave as a Git dependency:

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

    // Local example; a network application obtains the ID from authenticated link state.
    for share in shares.into_iter().take(4) {
        received.insert(share.relay_id(), share)?;
    }

    // TEST ORACLE ONLY: replace with fixed, transcript-bound PSIV/KDF evidence.
    // Real Bob does not already know Alice's root.
    let recovered = received.recover(|candidate| Ok::<_, Infallible>(candidate == &root))?;
    assert_eq!(recovered, root);
    Ok(())
}
```

Run `cargo run --example roundtrip`. See the [API guide](docs/API.md) for codecs,
RNG integration, error handling and the PSIV backend contract. Default features provide
the system RNG; disabling defaults gives `no_std + alloc`.

## Develop

```sh
cargo test --locked            # Rust tests; no Lean installation needed
sh scripts/check.sh rust       # Full Rust checks
sh scripts/check.sh            # Rust and Lean checks
sh scripts/export-vectors.sh   # Regenerate Rust fixtures from Lean
```

| Path | Contents |
| --- | --- |
| `src/`, `tests/`, `examples/` | Rust library, regression fixtures and usage example |
| `lean/` | Pinned Lean project, proofs, tests and fixture exporter |
| [SPEC.md](SPEC.md) | Normative protocol specification |
| [docs/VERIFICATION.md](docs/VERIFICATION.md) | Checked results, theorem index and reproduction steps |
| [docs/PROVENANCE.md](docs/PROVENANCE.md) | Original relay and PSIV sources |

**Experimental:** Lean proofs are kernel-checked; Rust is tested against Lean, not
formally proved equivalent or established to be constant time. PSIV remains an abstract
dependency with its existing **32-byte key, 12-byte nonce and 16-byte tag**. Its state
layout is unchanged. A concrete backend, KDF and authenticated confirmation exchange
remain integration work. The crate is not published on crates.io.

## Contribute and license

See [CONTRIBUTING.md](CONTRIBUTING.md) for setup and checks, and
[SECURITY.md](SECURITY.md) for the security scope and reporting guidance.
Licensed under the [MIT license](LICENSE).
