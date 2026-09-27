//! Expected outputs are emitted by the Lean executable, not computed by Rust tests.
use core::convert::Infallible;
use octave::{
    Coefficients, ReceivedShares, RelayId, RootSecret, Share, reconstruct_candidate,
    split_with_coefficients,
};
use std::collections::BTreeMap;

fn numbers(text: &str) -> Vec<u16> {
    text.split(',').map(|s| s.parse().unwrap()).collect()
}
fn root(text: &str) -> RootSecret {
    let bytes: Vec<u8> = numbers(text)
        .into_iter()
        .map(|n| u8::try_from(n).unwrap())
        .collect();
    RootSecret::try_from(bytes.as_slice()).unwrap()
}
fn field_bytes(text: &str) -> Vec<u8> {
    numbers(text)
        .into_iter()
        .flat_map(u16::to_le_bytes)
        .collect()
}
fn optional_root(text: &str) -> Option<RootSecret> {
    if text == "-" { None } else { Some(root(text)) }
}

#[test]
fn native_rust_matches_checked_lean_core() {
    let mut fixtures = BTreeMap::<usize, (RootSecret, [Share; 8])>::new();
    let mut recovery_cases = 0;
    let mut reconstructions = 0;
    for line in include_str!("fixtures/lean-vectors.txt")
        .lines()
        .filter(|s| !s.starts_with('#'))
    {
        let parts: Vec<_> = line.split('|').collect();
        match parts[0] {
            "S" => {
                assert_eq!(parts.len(), 5);
                let id: usize = parts[1].parse().unwrap();
                let root = root(parts[2]);
                let coefficients = Coefficients::from_bytes(&field_bytes(parts[3])).unwrap();
                let shares = split_with_coefficients(&root, &coefficients);
                let expected: Vec<_> = parts[4].split(';').collect();
                assert_eq!(expected.len(), 8);
                for (share, expected) in shares.iter().zip(expected) {
                    assert_eq!(
                        share.to_payload().as_ref(),
                        field_bytes(expected).as_slice(),
                        "fixture {id}"
                    );
                }
                for a in 0..8 {
                    for b in a + 1..8 {
                        for c in b + 1..8 {
                            for d in c + 1..8 {
                                let picked = [
                                    shares[a].clone(),
                                    shares[b].clone(),
                                    shares[c].clone(),
                                    shares[d].clone(),
                                ];
                                assert_eq!(reconstruct_candidate(&picked).unwrap(), root);
                                reconstructions += 1;
                            }
                        }
                    }
                }
                assert!(fixtures.insert(id, (root, shares)).is_none());
            }
            "R" => {
                assert_eq!(parts.len(), 9);
                let id: usize = parts[1].parse().unwrap();
                let (root, shares) = fixtures.get(&id).unwrap();
                let corrupt: u8 = parts[2].parse().unwrap();
                let missing: u8 = parts[3].parse().unwrap();
                let mut received = ReceivedShares::new();
                for (i, share) in shares.iter().enumerate() {
                    if missing & (1 << i) != 0 {
                        continue;
                    }
                    let mut payload = share.to_payload();
                    if corrupt & (1 << i) != 0 {
                        for (b, pair) in payload.chunks_exact_mut(2).enumerate() {
                            let value =
                                (u16::from_le_bytes([pair[0], pair[1]]) + (i + b + 1) as u16) % 257;
                            pair.copy_from_slice(&value.to_le_bytes());
                        }
                    }
                    let label = RelayId::new(i as u8 + 1).unwrap();
                    received
                        .insert(label, Share::from_payload(label, payload.as_ref()).unwrap())
                        .unwrap();
                }
                let candidates = received.candidates();
                assert_eq!(
                    candidates.subset_count(),
                    parts[4].parse::<usize>().unwrap()
                );
                assert_eq!(
                    candidates.invalid_byte_subsets(),
                    parts[5].parse::<usize>().unwrap()
                );
                let mut actual: Vec<_> = candidates.iter().map(|r| *r.as_bytes()).collect();
                let mut expected: Vec<_> = if parts[6].is_empty() {
                    Vec::new()
                } else {
                    parts[6]
                        .split(';')
                        .map(|s| *self::root(s).as_bytes())
                        .collect()
                };
                actual.sort();
                expected.sort();
                assert_eq!(actual, expected, "candidate set in case {recovery_cases}");
                assert_eq!(
                    received.recover(|r| Ok::<_, Infallible>(r == root)).ok(),
                    optional_root(parts[7])
                );
                assert_eq!(
                    received.recover(|_| Ok::<_, Infallible>(true)).ok(),
                    optional_root(parts[8])
                );
                recovery_cases += 1;
            }
            _ => panic!("unexpected fixture line"),
        }
    }
    assert_eq!(fixtures.len(), 7);
    assert_eq!(reconstructions, 490);
    assert_eq!(recovery_cases, 289); // 9 availability levels + all 280 worst-case fault placements
}
