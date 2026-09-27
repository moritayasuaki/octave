# Verification

Octave has kernel-checked Lean proofs and a native Rust implementation tested against
executable Lean fixtures. These establish algebraic and conditional protocol properties;
they do not establish Rust/Lean equivalence, constant-time execution or the security of
a concrete confirmation exchange. The core proofs are independent of a cipher, KDF
or MAC; PSIV is an optional adapter contract.

## Reproduce

```sh
sh scripts/check.sh rust       # Rust only
sh scripts/prepare.sh          # Place Lean caches in the ignored local workspace
(cd lean && lake exe cache get)
sh scripts/check.sh lean       # Lean build, axiom audit and native tests
sh scripts/export-vectors.sh   # Regenerate differential fixtures
```

Run `sh scripts/check.sh` for both Rust and Lean after preparing the pinned Mathlib
cache. The first cache fetch needs network access. Rust tests need no Lean installation.
Fixtures remain available when the Lean cache is absent.

Check logs and build output live under the ignored `.local/` directory. Cargo uses
`.local/target/`; `lean/.lake` links to `.local/lean/`. The preparation script preserves
an existing Lake cache when migrating it and refuses conflicting cache locations.

[GitHub Actions](https://github.com/moritayasuaki/octave/actions/workflows/ci.yml) runs
Rust checks on Linux and macOS, tests Rust 1.85, and checks Lean plus regenerated fixture
consistency. The linked run results are authoritative for hosted status at each commit.
The workflow does not publish packages.

## Reproducible checks

| Component | Check |
| --- | --- |
| Rust | Default, no-default, all-feature and no-default plus PSIV tests; all-feature release tests and core release build |
| Rust tooling | Formatting, Clippy and API documentation with warnings denied; runnable example |
| Lean | Core build and explicit `Relay.PSIV` check with pinned Lean 4.32.1 and Mathlib v4.32.1 |
| Proof audit | 30 principal declarations; reject `sorryAx` and `Lean.ofReduceBool` |
| Cross-language fixtures | Regenerate from Lean and compare with checked-in data |

The audit reports only Lean's standard foundations `propext`, `Classical.choice` and
`Quot.sound`, or no axioms for elementary parameter checks. Proof sources use no custom
axioms, admitted theorems or `native_decide`. The public theorem index and fixtures below
make the verification scope inspectable and reproducible.

## Checked theorem index

Names are in namespace `Relay`, under [lean/Relay/](../lean/Relay/).
Confirmation soundness and honest delivery are explicit theorem hypotheses.

| Requirement | Main checked declarations |
| --- | --- |
| Fixed profile and parameter inequalities | `parameter_bounds`, `enoughHonest` |
| ≥4 honest available labels under ≤3 corrupt + ≤1 offline | `four_honest_remain` |
| Complete, unique enumeration of 70 four-subsets | `allFourSubsets_length`, `allFourSubsets_nodup`, `allFourSubsets_complete`, `mem_allFourSubsets` |
| An all-honest four-subset exists | `exists_honest_four` |
| Sharing polynomial has degree ≤3 | `sharingPolynomial_degree` |
| Four distinct evaluations determine a cubic | `four_points_unique` |
| Executable scalar formula agrees with interpolation | `reconstructScalar_eq_interpolate` |
| Scalar reconstruction correctness | `reconstruct_polynomial`, `shamir_reconstruction_correct` |
| GF(257) and distinct nonzero labels | primality instance, `node_injective`, `node_nonzero` |
| Canonical 32-byte root roundtrip | `decode_encode_root` |
| Full-vector reconstruction correctness | `reconstruct_share` |
| Candidates use exactly four available labels | `candidateSubsets_spec` |
| Exactly choose(m,4), bounded by 70 | `candidate_count_exact`, `candidate_count_bound` |
| True root in actual executable candidate list | `true_secret_present` |
| Exact unique-confirmed-value semantics | `selectUnique_spec` |
| No wrong accepted value under soundness | `unique_selection_safe`, `recover_safe`, `recoverRoot_safe` |
| Success under soundness, completeness and honest delivery | `unique_selection_complete`, `recover_correct`, `recoverRoot_correct` |
| Mask vanishes at any ≤3 observed nonzero nodes | `exists_privacy_mask` |
| Secret-conditioned view fibers are finite over a finite field | `viewFiber_finite` |
| Bijection between view fibers for arbitrary secrets | `privacyEquiv`, `privacy_threshold_structure` |
| Fiber cardinality independent of the secret | `view_fiber_card_independent` |


The polynomial proofs are generic over a field and instantiated over GF(257) for each
of 32 byte coordinates. Privacy is proved as finite view-fiber bijections and equal
cardinalities, the structure underlying uniform-sampling privacy; this is not a formal
probability-distribution or full-network-transcript theorem.

## Executed coverage

[Lean's exporter](../lean/VectorMain.lean) produces the public
[fixtures](../tests/fixtures/lean-vectors.txt) used by Rust. Seven sharing fixtures
compare every share coordinate and all 70 four-subset reconstructions (490 checks).
Rust's separate zero-coefficient test adds another 70. The 289 recovery cases cover
nine availability levels and all 280 placements of three corrupt relays plus one
additional honest outage, comparing subset counts, invalid-byte counts, complete
distinct candidate sets and selection results.

Additional Rust tests cover canonical encodings, label bounds and identity mismatch,
duplicate labels, insufficient shares, invalid byte roots, candidate deduplication and
snapshots, ambiguity, verifier errors after an earlier success, redacted debug output,
RNG failures and rejection limits. Field tests exhaust all 256 nonzero inverses and
all 65,536 sampler inputs. With the `psiv` feature enabled, its non-cryptographic mock checks sizes,
limits, error propagation, output-length contracts and cleanup on drop.

[Lean's native tests](../lean/Main.lean) cover 70 unique subsets, availability counts,
560 direct reconstructions, byte boundaries, unique selection and all 280 fault
placements. An independent scalar vector uses secret 42 and coefficients (17,256,5),
yielding `(63,112,219,157,213,160,28,104)` at labels 1..8.

## Limits

Differential testing is not a Rust refinement proof. Runtime fault cases use
deterministic corruptions; the Lean safety theorem permits arbitrary corrupt values
under its hypotheses. The equality oracle used in fixtures and examples is a test aid,
not a real confirmation exchange.

No concrete cipher, contextual KDF, protected transport, transcript codec, replay/freshness
state machine or cryptographic confirmation exchange is included. The optional PSIV adapter
retains its 32/12/16-byte interface and unchanged state layout; its wrapper tests are not
cryptographic evidence. A tag length alone does not prove confirmation soundness.
Constant-time execution, complete memory erasure, forward secrecy and adaptive/full-transcript
privacy are not proved here. See the [integration contract](INTEGRATION.md).
