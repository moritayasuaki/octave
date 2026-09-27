# Project sources

Octave's protocol basis is the relay/client v0.1 design and its later 4-of-8 profile:
authenticated symmetric relay links, origin derived from link state, contextual
directional keys, and k=4, n=8, t=3, f=1. The original private design conversations and
relay archive are not distributed with this repository. The specification implements
recovered requirements rather than claiming a byte-for-byte revision of that archive.

The cryptographic boundary comes from the existing PSIV project. Its
0.4.0-experimental `c/psiv.h` and README were inspected: 32-byte keys, 12-byte nonces,
16-byte tags, external nonce/AD, `ciphertext || tag` records, 65,536-byte limits and
cached init/seal/open/clear operations. PSIV's implementation is not vendored or linked
here, and its internal state layout was not changed. Checking this relay layer makes
no claim about PSIV's cryptographic security or implementation proofs.

The GF(257)^32 sharing choice, canonical payload codec and concrete enumeration and
selection definitions were introduced in Octave. Mathlib's pinned source is authoritative
for the Lagrange interpolation API used in the proofs; dependency revisions are recorded
in `lean/lake-manifest.json`. Dependencies retain their own licenses.

The application library is native Rust. The mathematical reference is under `lean/`,
with namespace `Relay` and test target `relay_tests`. Public differential fixtures come
from `lean/VectorMain.lean`; passing them is not a formal Rust equivalence proof.
