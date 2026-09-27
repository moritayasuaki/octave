//! Dependency boundary for the existing PSIV implementation, not a new cipher.
//!
//! Matches PSIV 0.4.0-experimental's `c/psiv.h`: 32-byte keys, 12-byte nonces, 16-byte tags,
//! cached setup, and ciphertext followed by tag. Backend implementations must
//! preserve that project's state layout and authenticate before returning plaintext.
//! This module does not supply a PSIV implementation or a confirmation KDF.

use alloc::vec::Vec;
use core::fmt;
use zeroize::Zeroizing;

pub const KEY_BYTES: usize = 32;
pub const NONCE_BYTES: usize = 12;
pub const TAG_BYTES: usize = 16;
pub const MAX_PLAINTEXT_BYTES: usize = 65_536;
pub const MAX_ASSOCIATED_DATA_BYTES: usize = 65_536;
pub type Nonce = [u8; NONCE_BYTES];
pub type Tag = [u8; TAG_BYTES];

pub struct Key(Zeroizing<[u8; KEY_BYTES]>);
impl Key {
    pub fn from_bytes(bytes: [u8; KEY_BYTES]) -> Self {
        Self(Zeroizing::new(bytes))
    }
    pub fn as_bytes(&self) -> &[u8; KEY_BYTES] {
        &self.0
    }
}
impl fmt::Debug for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("PsivKey([REDACTED])")
    }
}

/// Adapter for an existing PSIV backend. `open` returns plaintext only on successful
/// authentication. `clear` must invalidate and erase the cached key setup.
/// There is no blanket implementation for unrelated AEAD algorithms.
pub trait Backend: Sized {
    type Error;
    fn init(key: &[u8; KEY_BYTES]) -> Result<Self, Self::Error>;
    fn seal(
        &mut self,
        nonce: &Nonce,
        associated_data: &[u8],
        plaintext: &[u8],
    ) -> Result<Vec<u8>, Self::Error>;
    fn open(
        &mut self,
        nonce: &Nonce,
        associated_data: &[u8],
        record: &[u8],
    ) -> Result<Vec<u8>, Self::Error>;
    fn clear(&mut self);
}

#[derive(Debug)]
#[non_exhaustive]
pub enum Error<E> {
    PlaintextLimit,
    AssociatedDataLimit,
    InvalidRecordLength,
    BackendContract,
    Backend(E),
}
impl<E: fmt::Display> fmt::Display for Error<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PlaintextLimit => f.write_str("PSIV plaintext exceeds 65,536 bytes"),
            Self::AssociatedDataLimit => f.write_str("PSIV associated data exceeds 65,536 bytes"),
            Self::InvalidRecordLength => {
                f.write_str("PSIV record length is outside 16..=65,552 bytes")
            }
            Self::BackendContract => {
                f.write_str("PSIV backend returned an unexpected output length")
            }
            Self::Backend(e) => write!(f, "PSIV backend: {e}"),
        }
    }
}
impl<E: core::error::Error + 'static> core::error::Error for Error<E> {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        if let Self::Backend(e) = self {
            Some(e)
        } else {
            None
        }
    }
}

/// Validates record limits and clears the backend on drop. Nonce/AD remain external
/// to the `ciphertext || tag` record. Nonce allocation and replay state belong to callers.
pub struct Session<B: Backend> {
    backend: B,
}
impl<B: Backend> Session<B> {
    pub fn new(key: &Key) -> Result<Self, Error<B::Error>> {
        Ok(Self {
            backend: B::init(key.as_bytes()).map_err(Error::Backend)?,
        })
    }
    pub fn seal(
        &mut self,
        nonce: &Nonce,
        ad: &[u8],
        plaintext: &[u8],
    ) -> Result<Vec<u8>, Error<B::Error>> {
        if plaintext.len() > MAX_PLAINTEXT_BYTES {
            return Err(Error::PlaintextLimit);
        }
        if ad.len() > MAX_ASSOCIATED_DATA_BYTES {
            return Err(Error::AssociatedDataLimit);
        }
        let record = self
            .backend
            .seal(nonce, ad, plaintext)
            .map_err(Error::Backend)?;
        if record.len() != plaintext.len() + TAG_BYTES {
            return Err(Error::BackendContract);
        }
        Ok(record)
    }
    pub fn open(
        &mut self,
        nonce: &Nonce,
        ad: &[u8],
        record: &[u8],
    ) -> Result<Zeroizing<Vec<u8>>, Error<B::Error>> {
        if !(TAG_BYTES..=MAX_PLAINTEXT_BYTES + TAG_BYTES).contains(&record.len()) {
            return Err(Error::InvalidRecordLength);
        }
        if ad.len() > MAX_ASSOCIATED_DATA_BYTES {
            return Err(Error::AssociatedDataLimit);
        }
        let plaintext = Zeroizing::new(
            self.backend
                .open(nonce, ad, record)
                .map_err(Error::Backend)?,
        );
        if plaintext.len() != record.len() - TAG_BYTES {
            return Err(Error::BackendContract);
        }
        Ok(plaintext)
    }
}
impl<B: Backend> Drop for Session<B> {
    fn drop(&mut self) {
        self.backend.clear();
    }
}
impl<B: Backend> fmt::Debug for Session<B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("PsivSession([REDACTED])")
    }
}
