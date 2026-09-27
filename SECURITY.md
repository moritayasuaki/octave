# Security scope

Octave is experimental research software. It is not ready to protect production data.
The Lean results prove algebraic and protocol properties under explicit hypotheses;
they do not prove the native Rust implementation, a cryptographic primitive or an end-to-end
network protocol secure. See [the verification report](docs/VERIFICATION.md).

The core selects no cipher, KDF or MAC. Real integrations must provide confidential
authenticated relay links, fixed transcript-bound confirmation, suitable key derivation,
freshness/replay management and cryptographic randomness. No cipher backend or
cryptographic confirmation exchange is included. Follow the
[integration contract](docs/INTEGRATION.md). Examples use public test oracles.
Constant-time execution, complete memory erasure and a post-quantum security reduction
are not established. There is no supported production release or response-time guarantee.

## Reporting

If this repository offers **Security → Report a vulnerability**, use that private channel.
If private reporting is unavailable, open an issue requesting a private contact channel
without including exploit details, keys, credentials or private transcripts. Do not assume
that an ordinary GitHub issue is private.

A useful report identifies the affected revision, the violated property or assumption,
and a minimal reproduction using dummy secrets. Distinguish implementation bugs from
limitations already documented in the specification and verification report.
