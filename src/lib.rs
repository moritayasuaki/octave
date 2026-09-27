//! Octave's native Rust 4-of-8 sharing and confirmation library.
//!
//! Eight independently labelled shares protect a 32-byte root. Any four honest
//! shares reconstruct it; candidate enumeration plus a sound confirmation
//! predicate tolerates three corrupt relays and one additional outage.
//!
//! # Boundary
//! The caller authenticates relay identities and session context **before**
//! inserting shares. The confirmation callback verifies fixed evidence for one
//! immutable transcript. Neither successful interpolation nor a callback that
//! simply returns `true` provides authentication.
//!
//! This is experimental Rust code tested against the checked Lean definitions.
//! The Rust implementation is not itself formally verified or established to be
//! constant time. PSIV remains an abstract dependency in [`psiv`].
//!
//! ```
//! use octave::{Coefficients, ReceivedShares, RootSecret, split_with_coefficients};
//! use core::convert::Infallible;
//!
//! // PUBLIC fixtures only. Real applications use split() with a cryptographic RNG.
//! let root = RootSecret::from_bytes([42; 32]);
//! let coins = Coefficients::from_bytes(&[0; 192]).unwrap();
//! let shares = split_with_coefficients(&root, &coins);
//! let mut received = ReceivedShares::new();
//! for share in shares.into_iter().take(4) {
//!     let authenticated_id = share.relay_id(); // supplied by trusted link state in production
//!     received.insert(authenticated_id, share).unwrap();
//! }
//! // Test oracle only; replace with transcript-bound cryptographic verification.
//! let recovered = received.recover(|candidate| Ok::<_, Infallible>(candidate == &root)).unwrap();
//! assert_eq!(recovered, root);
//! ```

#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]

extern crate alloc;

mod error;
mod field;
mod secret;
mod sharing;

pub mod psiv;

pub use error::{ConfirmationError, Error, RandomError};
pub use rand_core;
pub use secret::{Coefficients, RootSecret};
pub use sharing::{
    CandidateSet, ReceivedShares, RelayId, Share, reconstruct_candidate, split,
    split_with_coefficients,
};

#[cfg(feature = "os-rng")]
pub use getrandom::SysRng;

pub const RELAY_COUNT: usize = 8;
pub const THRESHOLD: usize = 4;
pub const MAX_CORRUPT: usize = 3;
pub const MAX_OFFLINE: usize = 1;
pub const ROOT_BYTES: usize = 32;
pub const SHARE_PAYLOAD_BYTES: usize = 64;
pub const SHARE_BYTES: usize = 65;
pub const COEFFICIENT_BYTES: usize = 192;
pub const MAX_CANDIDATES: usize = 70;
