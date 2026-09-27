use octave::psiv::{self, Backend, Key, Nonce, Session, TAG_BYTES};
use std::sync::atomic::{AtomicUsize, Ordering};

static CLEARS: AtomicUsize = AtomicUsize::new(0);
static CALLS: AtomicUsize = AtomicUsize::new(0);

// Contract mock, NOT a cipher. Nothing in this file is PSIV cryptographic evidence.
struct MockBackend(u8);
impl Backend for MockBackend {
    type Error = &'static str;
    fn init(key: &[u8; 32]) -> Result<Self, Self::Error> {
        Ok(Self(key[0]))
    }
    fn seal(&mut self, _: &Nonce, _: &[u8], plaintext: &[u8]) -> Result<Vec<u8>, Self::Error> {
        CALLS.fetch_add(1, Ordering::SeqCst);
        if self.0 == 1 {
            return Ok(Vec::new());
        }
        let mut record = plaintext.to_vec();
        record.extend_from_slice(&[42; TAG_BYTES]);
        Ok(record)
    }
    fn open(&mut self, _: &Nonce, _: &[u8], record: &[u8]) -> Result<Vec<u8>, Self::Error> {
        CALLS.fetch_add(1, Ordering::SeqCst);
        if self.0 == 2 {
            return Ok(Vec::new());
        }
        if record[record.len() - TAG_BYTES..] != [42; TAG_BYTES] {
            return Err("authentication failed");
        }
        Ok(record[..record.len() - TAG_BYTES].to_vec())
    }
    fn clear(&mut self) {
        CLEARS.fetch_add(1, Ordering::SeqCst);
        self.0 = 0;
    }
}

#[test]
fn fixed_psiv_interface_and_authentication_output_contract() {
    assert_eq!(
        (psiv::KEY_BYTES, psiv::NONCE_BYTES, psiv::TAG_BYTES),
        (32, 12, 16)
    );
    let nonce = [0; 12];
    {
        let mut session = Session::<MockBackend>::new(&Key::from_bytes([0; 32])).unwrap();
        for length in [0, 1, 65536] {
            let plaintext = vec![7; length];
            let record = session.seal(&nonce, &[], &plaintext).unwrap();
            assert_eq!(record.len(), length + 16);
            assert_eq!(
                session.open(&nonce, &[], &record).unwrap().as_slice(),
                plaintext.as_slice()
            );
        }
        let before = CALLS.load(Ordering::SeqCst);
        assert!(matches!(
            session.seal(&nonce, &[], &[0; 65537]),
            Err(psiv::Error::PlaintextLimit)
        ));
        assert!(matches!(
            session.seal(&nonce, &[0; 65537], &[]),
            Err(psiv::Error::AssociatedDataLimit)
        ));
        assert!(matches!(
            session.open(&nonce, &[], &[0; 15]),
            Err(psiv::Error::InvalidRecordLength)
        ));
        assert!(matches!(
            session.open(&nonce, &[], &[0; 65553]),
            Err(psiv::Error::InvalidRecordLength)
        ));
        assert!(matches!(
            session.open(&nonce, &[0; 65537], &[42; 16]),
            Err(psiv::Error::AssociatedDataLimit)
        ));
        assert_eq!(CALLS.load(Ordering::SeqCst), before);
        assert!(matches!(
            session.open(&nonce, &[], &[0; 16]),
            Err(psiv::Error::Backend("authentication failed"))
        ));
    }
    {
        let mut broken = Session::<MockBackend>::new(&Key::from_bytes([1; 32])).unwrap();
        assert!(matches!(
            broken.seal(&nonce, &[], &[]),
            Err(psiv::Error::BackendContract)
        ));
    }
    {
        let mut broken = Session::<MockBackend>::new(&Key::from_bytes([2; 32])).unwrap();
        assert!(matches!(
            broken.open(&nonce, &[], &[42; 17]),
            Err(psiv::Error::BackendContract)
        ));
    }
    assert_eq!(CLEARS.load(Ordering::SeqCst), 3);
}
