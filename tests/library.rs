use core::{cell::Cell, convert::Infallible, fmt};
use octave::rand_core::{TryCryptoRng, TryRng};
use octave::{
    Coefficients, ConfirmationError, Error, RandomError, ReceivedShares, RelayId, RootSecret,
    Share, reconstruct_candidate, split, split_with_coefficients,
};
use std::collections::VecDeque;

fn id(n: u8) -> RelayId {
    RelayId::new(n).unwrap()
}
fn fixture() -> (RootSecret, [Share; 8]) {
    let root = RootSecret::from_bytes(core::array::from_fn(|b| b as u8));
    let bytes: Vec<_> = (0..96)
        .flat_map(|i| ((31 * i % 257) as u16).to_le_bytes())
        .collect();
    let coins = Coefficients::from_bytes(&bytes).unwrap();
    let shares = split_with_coefficients(&root, &coins);
    (root, shares)
}
fn receive(shares: impl IntoIterator<Item = Share>) -> ReceivedShares {
    let mut received = ReceivedShares::new();
    for share in shares {
        received.insert(share.relay_id(), share).unwrap();
    }
    received
}

#[test]
fn encodings_are_canonical_and_labels_are_bound() {
    for label in [0, 9, 255] {
        assert!(matches!(RelayId::new(label), Err(Error::InvalidRelayId(_))));
    }
    for len in [0, 31, 33] {
        assert!(RootSecret::try_from(vec![0; len].as_slice()).is_err());
    }
    for len in [0, 63, 65, 1000] {
        assert!(Share::from_payload(id(1), &vec![0; len]).is_err());
    }
    for len in [0, 64, 66, 1000] {
        assert!(Share::from_bytes(&vec![0; len]).is_err());
    }
    for len in [0, 191, 193] {
        assert!(Coefficients::from_bytes(&vec![0; len]).is_err());
    }
    for value in [257u16, 65535] {
        for position in 0..32 {
            let mut bytes = [0; 64];
            bytes[2 * position..2 * position + 2].copy_from_slice(&value.to_le_bytes());
            assert!(
                matches!(Share::from_payload(id(8), &bytes), Err(Error::NonCanonicalField { element }) if element == position)
            );
        }
        let mut coins = [0; 192];
        coins[190..].copy_from_slice(&value.to_le_bytes());
        assert!(matches!(
            Coefficients::from_bytes(&coins),
            Err(Error::NonCanonicalField { element: 95 })
        ));
    }
    for value in 0u16..=256 {
        let payload: Vec<_> = (0..32).flat_map(|_| value.to_le_bytes()).collect();
        let share = Share::from_payload(id(3), &payload).unwrap();
        assert_eq!(share.to_payload().as_ref(), payload.as_slice());
        let restored = Share::from_bytes(share.to_bytes().as_ref()).unwrap();
        assert_eq!(restored.relay_id(), id(3));
        assert_eq!(restored.to_payload().as_ref(), payload.as_slice());
    }
    let (_, shares) = fixture();
    let mut received = ReceivedShares::new();
    assert!(matches!(
        received.insert(id(2), shares[0].clone()),
        Err(Error::LabelMismatch { .. })
    ));
    assert!(received.is_empty());
    received.insert(id(1), shares[0].clone()).unwrap();
    assert!(matches!(
        received.insert(id(1), shares[0].clone()),
        Err(Error::DuplicateRelay(_))
    ));
    assert_eq!(received.len(), 1);
    assert!(matches!(
        reconstruct_candidate(&[
            shares[0].clone(),
            shares[1].clone(),
            shares[2].clone(),
            shares[0].clone()
        ]),
        Err(Error::DuplicateRelay(_))
    ));
}

#[test]
fn zero_coefficients_and_every_four_subset_work() {
    let root = RootSecret::from_bytes([255; 32]);
    let coins = Coefficients::from_bytes(&[0; 192]).unwrap();
    let shares = split_with_coefficients(&root, &coins);
    let mut count = 0;
    for a in 0..8 {
        for b in a + 1..8 {
            for c in b + 1..8 {
                for d in c + 1..8 {
                    assert_eq!(
                        reconstruct_candidate(&[
                            shares[a].clone(),
                            shares[b].clone(),
                            shares[c].clone(),
                            shares[d].clone()
                        ])
                        .unwrap(),
                        root
                    );
                    count += 1;
                }
            }
        }
    }
    assert_eq!(count, 70);
}

#[test]
fn confirmation_checks_unique_values_and_fails_closed() {
    let (root, shares) = fixture();
    let received = receive(shares.clone());
    let snapshot = received.candidates();
    assert_eq!(snapshot.subset_count(), 70);
    assert_eq!(snapshot.len(), 1);
    let calls = Cell::new(0);
    let recovered = snapshot
        .confirm(|candidate| {
            calls.set(calls.get() + 1);
            Ok::<_, Infallible>(candidate == &root)
        })
        .unwrap();
    assert_eq!(calls.get(), 1);
    assert_eq!(recovered, root);
    assert!(matches!(
        snapshot.confirm(|_| Ok::<_, Infallible>(false)),
        Err(ConfirmationError::NoConfirmedCandidate)
    ));

    let mut modified = shares;
    // One corrupt relay changes each coordinate by one, producing distinct canonical candidates.
    let mut payload = modified[0].to_payload();
    for pair in payload.chunks_exact_mut(2) {
        let value = (u16::from_le_bytes([pair[0], pair[1]]) + 1) % 257;
        pair.copy_from_slice(&value.to_le_bytes());
    }
    modified[0] = Share::from_payload(id(1), payload.as_ref()).unwrap();
    let candidates = receive(modified).candidates();
    assert!(candidates.len() > 1);
    let calls = Cell::new(0);
    assert!(matches!(
        candidates.confirm(|_| {
            calls.set(calls.get() + 1);
            Ok::<_, Infallible>(true)
        }),
        Err(ConfirmationError::AmbiguousConfirmation)
    ));
    assert_eq!(calls.get(), candidates.len()); // no early acceptance of the first candidate
    let calls = Cell::new(0);
    assert!(matches!(
        candidates.confirm(|_| {
            calls.set(calls.get() + 1);
            if calls.get() == 1 {
                Ok(true)
            } else {
                Err("verification unavailable")
            }
        }),
        Err(ConfirmationError::Verifier("verification unavailable"))
    ));
    // The original snapshot is independent of later receipts and mutations.
    assert_eq!(
        snapshot
            .confirm(|r| Ok::<_, Infallible>(r == &root))
            .unwrap(),
        root
    );
}

#[test]
fn absent_shares_and_non_byte_candidates_are_never_confirmed() {
    let (_, shares) = fixture();
    for n in 0..4 {
        let received = receive(shares.iter().take(n).cloned());
        assert_eq!(received.candidates().subset_count(), 0);
        assert!(received.candidates().is_empty());
        assert!(
            matches!(received.recover::<Infallible>(|_| panic!("verifier must not run")), Err(ConfirmationError::InsufficientShares { received: m }) if m == n)
        );
    }
    let invalid: Vec<_> = (1..=8)
        .map(|i| Share::from_payload(id(i), &[0, 1].repeat(32)).unwrap())
        .collect();
    let received = receive(invalid);
    let candidates = received.candidates();
    assert_eq!(candidates.subset_count(), 70);
    assert_eq!(candidates.invalid_byte_subsets(), 70);
    assert!(matches!(
        received.recover::<Infallible>(|_| panic!("non-byte candidate leaked")),
        Err(ConfirmationError::NoConfirmedCandidate)
    ));
    let (_, valid) = fixture();
    let mut one_bad_coordinate = valid[0].to_payload();
    // Constant polynomial 256 in just the last coordinate must also be rejected.
    one_bad_coordinate[62..].copy_from_slice(&[0, 1]);
    let shares: [Share; 4] = core::array::from_fn(|i| {
        Share::from_payload(id(i as u8 + 1), one_bad_coordinate.as_ref()).unwrap()
    });
    assert!(matches!(
        reconstruct_candidate(&shares),
        Err(Error::InvalidByteCandidate)
    ));
}

#[derive(Debug)]
struct RngFailure;
impl fmt::Display for RngFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("test RNG failure")
    }
}
impl core::error::Error for RngFailure {}
struct StubRng {
    words: VecDeque<u16>,
    fill: u16,
    calls: usize,
    fail_at: Option<usize>,
}
impl TryRng for StubRng {
    type Error = RngFailure;
    fn try_next_u32(&mut self) -> Result<u32, Self::Error> {
        let mut bytes = [0; 4];
        self.try_fill_bytes(&mut bytes)?;
        Ok(u32::from_le_bytes(bytes))
    }
    fn try_next_u64(&mut self) -> Result<u64, Self::Error> {
        let mut bytes = [0; 8];
        self.try_fill_bytes(&mut bytes)?;
        Ok(u64::from_le_bytes(bytes))
    }
    fn try_fill_bytes(&mut self, dst: &mut [u8]) -> Result<(), Self::Error> {
        self.calls += 1;
        if self.fail_at == Some(self.calls) {
            return Err(RngFailure);
        }
        for chunk in dst.chunks_mut(2) {
            let bytes = self.words.pop_front().unwrap_or(self.fill).to_le_bytes();
            chunk.copy_from_slice(&bytes[..chunk.len()]);
        }
        Ok(())
    }
}
// Intentionally insecure mock confined to tests; never an application RNG.
impl TryCryptoRng for StubRng {}

#[test]
fn randomness_failures_and_rejection_sampling() {
    let mut rng = StubRng {
        words: [65535, 256].into(),
        fill: 0,
        calls: 0,
        fail_at: None,
    };
    let coins = Coefficients::random(&mut rng).unwrap();
    assert_eq!(&coins.to_bytes()[..2], &[0, 1]);
    assert_eq!(rng.calls, 97);
    let root = RootSecret::from_bytes([42; 32]);
    let shares = split(&root, &mut rng).unwrap();
    assert_eq!(
        receive(shares)
            .recover(|r| Ok::<_, Infallible>(r == &root))
            .unwrap(),
        root
    );
    let mut rng = StubRng {
        words: VecDeque::new(),
        fill: 65535,
        calls: 0,
        fail_at: None,
    };
    assert!(matches!(
        Coefficients::random(&mut rng),
        Err(RandomError::RejectionLimit)
    ));
    assert_eq!(rng.calls, 128);
    let mut rng = StubRng {
        words: VecDeque::new(),
        fill: 0,
        calls: 0,
        fail_at: Some(3),
    };
    assert!(matches!(
        split(&root, &mut rng),
        Err(RandomError::Source(_))
    ));
    let mut rng = StubRng {
        words: VecDeque::new(),
        fill: 0,
        calls: 0,
        fail_at: Some(1),
    };
    assert!(matches!(
        RootSecret::random(&mut rng),
        Err(RandomError::Source(_))
    ));
}

#[test]
fn debug_never_formats_secret_values() {
    let (root, shares) = fixture();
    assert_eq!(format!("{root:?}"), "RootSecret([REDACTED])");
    assert_eq!(
        format!("{:?}", Coefficients::from_bytes(&[0; 192]).unwrap()),
        "Coefficients([REDACTED])"
    );
    assert!(format!("{:?}", shares[0]).contains("[REDACTED]"));
}
