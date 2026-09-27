use crate::{
    Coefficients, ConfirmationError, Error, MAX_CANDIDATES, RELAY_COUNT, ROOT_BYTES, RandomError,
    RootSecret, SHARE_BYTES, SHARE_PAYLOAD_BYTES, THRESHOLD, field,
};
use alloc::vec::Vec;
use core::fmt;
use rand_core::TryCryptoRng;
use zeroize::Zeroizing;

/// External relay labels 1..=8, fixed by the authenticated session roster.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct RelayId(u8);
impl RelayId {
    pub fn new(value: u8) -> Result<Self, Error> {
        if (1..=RELAY_COUNT as u8).contains(&value) {
            Ok(Self(value))
        } else {
            Err(Error::InvalidRelayId(value))
        }
    }
    pub const fn get(self) -> u8 {
        self.0
    }
    fn index(self) -> usize {
        (self.0 - 1) as usize
    }
}

/// A labelled 32-coordinate share. Parsing does not authenticate it.
#[derive(Clone)]
pub struct Share {
    relay_id: RelayId,
    values: Zeroizing<[u16; ROOT_BYTES]>,
}
impl Share {
    pub const fn relay_id(&self) -> RelayId {
        self.relay_id
    }
    pub fn from_payload(relay_id: RelayId, payload: &[u8]) -> Result<Self, Error> {
        if payload.len() != SHARE_PAYLOAD_BYTES {
            return Err(Error::Length {
                item: "share payload",
                expected: SHARE_PAYLOAD_BYTES,
                actual: payload.len(),
            });
        }
        let mut result = Self {
            relay_id,
            values: Zeroizing::new([0; ROOT_BYTES]),
        };
        for (index, pair) in payload.chunks_exact(2).enumerate() {
            let value = u16::from_le_bytes([pair[0], pair[1]]);
            if value >= field::MODULUS {
                return Err(Error::NonCanonicalField { element: index });
            }
            result.values[index] = value;
        }
        Ok(result)
    }
    /// Encoding: one external label, then 32 canonical little-endian u16 elements.
    /// Session headers and authentication are supplied by the application.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() != SHARE_BYTES {
            return Err(Error::Length {
                item: "labelled share",
                expected: SHARE_BYTES,
                actual: bytes.len(),
            });
        }
        Self::from_payload(RelayId::new(bytes[0])?, &bytes[1..])
    }
    pub fn to_payload(&self) -> Zeroizing<[u8; SHARE_PAYLOAD_BYTES]> {
        let mut out = Zeroizing::new([0; SHARE_PAYLOAD_BYTES]);
        for (index, value) in self.values.iter().enumerate() {
            out[2 * index..2 * index + 2].copy_from_slice(&value.to_le_bytes());
        }
        out
    }
    pub fn to_bytes(&self) -> Zeroizing<[u8; SHARE_BYTES]> {
        let mut out = Zeroizing::new([0; SHARE_BYTES]);
        out[0] = self.relay_id.get();
        out[1..].copy_from_slice(self.to_payload().as_ref());
        out
    }
}
impl fmt::Debug for Share {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Share")
            .field("relay_id", &self.relay_id)
            .field("payload", &"[REDACTED]")
            .finish()
    }
}

/// Generate fresh independent coefficients and split a 32-byte root into eight shares.
pub fn split<R: TryCryptoRng + ?Sized>(
    root: &RootSecret,
    rng: &mut R,
) -> Result<[Share; RELAY_COUNT], RandomError<R::Error>> {
    let coins = Coefficients::random(rng)?;
    Ok(split_with_coefficients(root, &coins))
}

/// Advanced deterministic entry point. Caller must ensure the coefficients are fresh,
/// independent and uniform. Zero leading coefficients are valid and must not be excluded.
pub fn split_with_coefficients(root: &RootSecret, coins: &Coefficients) -> [Share; RELAY_COUNT] {
    core::array::from_fn(|i| {
        let x = (i + 1) as u16;
        let mut values = Zeroizing::new([0; ROOT_BYTES]);
        for (b, value) in values.iter_mut().enumerate() {
            let [a1, a2, a3] = coins.0[b];
            *value = field::add(
                root.as_bytes()[b] as u16,
                field::mul(
                    x,
                    field::add(a1, field::mul(x, field::add(a2, field::mul(x, a3)))),
                ),
            );
        }
        Share {
            relay_id: RelayId((i + 1) as u8),
            values,
        }
    })
}

fn reconstruct_refs(shares: [&Share; THRESHOLD]) -> Result<RootSecret, Error> {
    let mut seen = [false; RELAY_COUNT];
    for share in shares {
        if seen[share.relay_id.index()] {
            return Err(Error::DuplicateRelay(share.relay_id));
        }
        seen[share.relay_id.index()] = true;
    }
    let weights: [u16; THRESHOLD] = core::array::from_fn(|i| {
        let xi = shares[i].relay_id.get() as u16;
        let mut weight = 1;
        for (j, share) in shares.iter().enumerate() {
            if i != j {
                let xj = share.relay_id.get() as u16;
                weight = field::mul(
                    weight,
                    field::mul(field::sub(0, xj), field::inv(field::sub(xi, xj))),
                );
            }
        }
        weight
    });
    let mut root = RootSecret::from_bytes([0; ROOT_BYTES]);
    for (b, byte) in root.0.iter_mut().enumerate() {
        let mut value = 0;
        for (share, weight) in shares.iter().zip(weights) {
            value = field::add(value, field::mul(share.values[b], weight));
        }
        if value == 256 {
            return Err(Error::InvalidByteCandidate);
        }
        *byte = value as u8;
    }
    Ok(root)
}

/// Reconstruct one **unconfirmed** candidate from exactly four distinct labels.
/// Never establish a session from this result without cryptographic confirmation.
pub fn reconstruct_candidate(shares: &[Share; THRESHOLD]) -> Result<RootSecret, Error> {
    reconstruct_refs(core::array::from_fn(|i| &shares[i]))
}

/// One slot per authenticated relay, belonging to a single immutable session context.
/// Conflicting or repeated insertions are errors and leave the prior slot unchanged.
#[derive(Clone)]
pub struct ReceivedShares {
    slots: [Option<Share>; RELAY_COUNT],
}
impl Default for ReceivedShares {
    fn default() -> Self {
        Self::new()
    }
}
impl ReceivedShares {
    pub fn new() -> Self {
        Self {
            slots: core::array::from_fn(|_| None),
        }
    }
    pub fn len(&self) -> usize {
        self.slots.iter().filter(|s| s.is_some()).count()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn insert(&mut self, authenticated_relay: RelayId, share: Share) -> Result<(), Error> {
        if authenticated_relay != share.relay_id {
            return Err(Error::LabelMismatch {
                authenticated: authenticated_relay,
                claimed: share.relay_id,
            });
        }
        let slot = &mut self.slots[authenticated_relay.index()];
        if slot.is_some() {
            return Err(Error::DuplicateRelay(authenticated_relay));
        }
        *slot = Some(share);
        Ok(())
    }
    /// Enumerate all four-subsets of available labels. Byte-invalid candidates are
    /// discarded and equal root values are deduplicated. No cryptographic acceptance occurs.
    pub fn candidates(&self) -> CandidateSet {
        let available: Vec<_> = self.slots.iter().flatten().collect();
        let mut result = CandidateSet {
            roots: Vec::with_capacity(MAX_CANDIDATES),
            subsets: 0,
            invalid: 0,
        };
        let n = available.len();
        for a in 0..n {
            for b in a + 1..n {
                for c in b + 1..n {
                    for d in c + 1..n {
                        result.subsets += 1;
                        match reconstruct_refs([
                            available[a],
                            available[b],
                            available[c],
                            available[d],
                        ]) {
                            Ok(root) => {
                                if !result.roots.contains(&root) {
                                    result.roots.push(root);
                                }
                            }
                            Err(Error::InvalidByteCandidate) => result.invalid += 1,
                            Err(_) => {
                                unreachable!("fixed distinct slots always give distinct labels")
                            }
                        }
                    }
                }
            }
        }
        result
    }
    /// Confirm against fixed evidence bound to the session/peers/profile/roster/direction.
    /// Safety requires: every confirmed candidate equals Alice's actual root.
    pub fn recover<E>(
        &self,
        verifies: impl Fn(&RootSecret) -> Result<bool, E>,
    ) -> Result<RootSecret, ConfirmationError<E>> {
        if self.len() < THRESHOLD {
            return Err(ConfirmationError::InsufficientShares {
                received: self.len(),
            });
        }
        self.candidates().confirm(verifies)
    }
}
impl fmt::Debug for ReceivedShares {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ReceivedShares")
            .field("count", &self.len())
            .finish_non_exhaustive()
    }
}

/// Owned snapshot of distinct, canonical, **unconfirmed** candidate roots.
pub struct CandidateSet {
    roots: Vec<RootSecret>,
    subsets: usize,
    invalid: usize,
}
impl CandidateSet {
    pub fn len(&self) -> usize {
        self.roots.len()
    }
    pub fn is_empty(&self) -> bool {
        self.roots.is_empty()
    }
    pub const fn subset_count(&self) -> usize {
        self.subsets
    }
    pub const fn invalid_byte_subsets(&self) -> usize {
        self.invalid
    }
    pub fn iter(&self) -> impl ExactSizeIterator<Item = &RootSecret> {
        self.roots.iter()
    }
    /// Verify each distinct root at most once, and finish all checks before success.
    /// Verifier errors fail closed; two different successful roots are ambiguous.
    /// A successful callback is an assumption at this layer, not a cryptographic security proof.
    pub fn confirm<E>(
        &self,
        verifies: impl Fn(&RootSecret) -> Result<bool, E>,
    ) -> Result<RootSecret, ConfirmationError<E>> {
        let mut selected = None;
        let mut ambiguous = false;
        for root in &self.roots {
            if verifies(root).map_err(ConfirmationError::Verifier)? {
                if selected.is_some() {
                    ambiguous = true;
                } else {
                    selected = Some(root);
                }
            }
        }
        if ambiguous {
            return Err(ConfirmationError::AmbiguousConfirmation);
        }
        selected
            .cloned()
            .ok_or(ConfirmationError::NoConfirmedCandidate)
    }
}
impl fmt::Debug for CandidateSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CandidateSet")
            .field("subsets", &self.subsets)
            .field("distinct_roots", &self.len())
            .field("invalid_byte_subsets", &self.invalid)
            .finish_non_exhaustive()
    }
}
