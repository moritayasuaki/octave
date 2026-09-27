# Contributing

Octave is an experimental Rust library with a checked Lean reference. Keep the fixed
4-of-8 profile, canonical encodings and unique-confirmation semantics aligned across
the implementation, specification and proofs. The core is independent of a cipher,
KDF or MAC. Keep the optional PSIV adapter isolated; preserve its 32-byte key, 12-byte
nonce, 16-byte tag and existing state layout when changing that adapter.

## Local checks

Install Rust with [rustup](https://rustup.rs/), including `rustfmt` and `clippy`, then run:

```sh
sh scripts/check.sh rust
```

For formal work, install [Lean's elan toolchain manager](https://lean-lang.org/install/manual/).
From the repository root, prepare the pinned dependency cache and check the proofs:

```sh
sh scripts/prepare.sh
(cd lean && lake exe cache get)
sh scripts/check.sh lean
```

The Rust script checks default, no-default, all-feature and `no_std` plus PSIV builds.
The Lean script checks the core, proofs and explicitly imported PSIV contract.
Run `sh scripts/check.sh` for both. Logs, build output and local notes belong under the ignored `.local/` directory.
Cargo writes to `.local/target/`; preparation links `lean/.lake` to `.local/lean/`.
Keep reproducible tests, proof sources and public documentation in the repository.

After changing executable Lean definitions, run `sh scripts/export-vectors.sh`, review
the fixture diff, and rerun the Rust checks. Commit intentional source, lockfile and
fixture changes together. The CI workflow verifies regenerated fixtures are unchanged.

## Changes and reports

Describe the problem, resulting behavior and checks performed in each pull request.
Add regression coverage for behavior changes, especially canonical decoding, duplicate
labels, candidate ambiguity and failure handling. Keep soundness/delivery hypotheses
explicit; do not introduce `sorry`, custom axioms or `native_decide` into proofs.

For ordinary bugs, include a minimal reproduction using public dummy data, the crate
revision and toolchain version. Follow [SECURITY.md](../SECURITY.md) for security findings.
Do not put real roots, keys, credentials or private transcripts in issues or fixtures.
