# Octave

Repository instructions for the Octave Rust library and Lean reference.

- The application library is native Rust in `src/`; the mathematical reference is in `lean/`.
- Keep the normative profile k=4, n=8, t=3, f=1.
- Preserve PSIV's 32-byte key, 12-byte nonce, 16-byte tag and existing state layout.
- Keep confirmation soundness and honest delivery explicit as assumptions. Do not claim a concrete cryptographic reduction or Rust/Lean equivalence proof.
- Keep the Lean namespace `Relay` and executable target `relay_tests` stable unless an API change is requested.
- Run `sh scripts/check.sh rust` for Rust changes, `sh scripts/check.sh lean` for Lean changes, or `sh scripts/check.sh` for both.
- Regenerate fixtures with `sh scripts/export-vectors.sh` after changes to Lean behavior, then run the Rust tests.
- Do not add `sorry`, admitted theorems, custom axioms, or `native_decide` to mathematical proofs.
- Generated caches and logs belong under `target/` and `lean/.lake/` and are not source files.
