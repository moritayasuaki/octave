use core::convert::Infallible;
use octave::{ReceivedShares, RootSecret, SysRng, split};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut rng = SysRng;
    let root = RootSecret::random(&mut rng)?;
    let shares = split(&root, &mut rng)?;
    let mut received = ReceivedShares::new();

    // Confidential, authenticated transport would supply these trusted peer/session IDs.
    // This local demo just delivers the first four honest shares.
    for share in shares.into_iter().take(4) {
        let authenticated_relay = share.relay_id();
        received.insert(authenticated_relay, share)?;
    }

    // Test oracle ONLY: real Bob does not already know Alice's root. An application
    // must verify confirmation evidence bound to the complete fixed transcript.
    let established = received.recover(|candidate| Ok::<_, Infallible>(candidate == &root))?;
    assert_eq!(established, root);
    println!("Octave Rust roundtrip passed (local confirmation test oracle).");
    Ok(())
}
