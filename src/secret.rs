use crate::{COEFFICIENT_BYTES, Error, ROOT_BYTES, RandomError, field};
use core::fmt;
use rand_core::TryCryptoRng;
use subtle::ConstantTimeEq;
use zeroize::Zeroizing;

/// Bootstrap material. Debug is redacted; the owned buffer is zeroized on drop.
/// Copies made by the caller, compiler or allocator are outside this guarantee.
#[derive(Clone)]
pub struct RootSecret(pub(crate) Zeroizing<[u8; ROOT_BYTES]>);

impl RootSecret {
    pub fn from_bytes(bytes: [u8; ROOT_BYTES]) -> Self {
        Self(Zeroizing::new(bytes))
    }
    pub fn as_bytes(&self) -> &[u8; ROOT_BYTES] {
        &self.0
    }
    pub fn random<R: TryCryptoRng + ?Sized>(rng: &mut R) -> Result<Self, RandomError<R::Error>> {
        let mut root = Self::from_bytes([0; ROOT_BYTES]);
        rng.try_fill_bytes(root.0.as_mut())
            .map_err(RandomError::Source)?;
        Ok(root)
    }
}
impl TryFrom<&[u8]> for RootSecret {
    type Error = Error;
    fn try_from(bytes: &[u8]) -> Result<Self, Error> {
        let bytes: [u8; ROOT_BYTES] = bytes.try_into().map_err(|_| Error::Length {
            item: "root",
            expected: ROOT_BYTES,
            actual: bytes.len(),
        })?;
        Ok(Self::from_bytes(bytes))
    }
}
impl PartialEq for RootSecret {
    fn eq(&self, other: &Self) -> bool {
        bool::from(self.as_bytes().ct_eq(other.as_bytes()))
    }
}
impl Eq for RootSecret {}
impl fmt::Debug for RootSecret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("RootSecret([REDACTED])")
    }
}

/// 96 independent field samples, in byte-coordinate order, then degree 1, 2, 3.
/// Never reuse a coefficient set for another root or session.
pub struct Coefficients(pub(crate) Zeroizing<[[u16; 3]; ROOT_BYTES]>);

impl Coefficients {
    pub fn random<R: TryCryptoRng + ?Sized>(rng: &mut R) -> Result<Self, RandomError<R::Error>> {
        let mut result = Self(Zeroizing::new([[0; 3]; ROOT_BYTES]));
        let mut bytes = Zeroizing::new([0; 2]);
        for row in result.0.iter_mut() {
            for coefficient in row {
                let mut sampled = None;
                for _ in 0..128 {
                    rng.try_fill_bytes(bytes.as_mut())
                        .map_err(RandomError::Source)?;
                    sampled = field::sample_word(u16::from_le_bytes(*bytes));
                    if sampled.is_some() {
                        break;
                    }
                }
                *coefficient = sampled.ok_or(RandomError::RejectionLimit)?;
            }
        }
        Ok(result)
    }

    /// Advanced deterministic interface. Valid encoding does not imply random input.
    /// All coefficients must have been sampled independently, uniformly and freshly.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() != COEFFICIENT_BYTES {
            return Err(Error::Length {
                item: "coefficients",
                expected: COEFFICIENT_BYTES,
                actual: bytes.len(),
            });
        }
        let mut result = Self(Zeroizing::new([[0; 3]; ROOT_BYTES]));
        for (index, pair) in bytes.chunks_exact(2).enumerate() {
            let value = u16::from_le_bytes([pair[0], pair[1]]);
            if value >= field::MODULUS {
                return Err(Error::NonCanonicalField { element: index });
            }
            result.0[index / 3][index % 3] = value;
        }
        Ok(result)
    }

    /// Secret material; returned storage is also zeroized on drop.
    pub fn to_bytes(&self) -> Zeroizing<[u8; COEFFICIENT_BYTES]> {
        let mut out = Zeroizing::new([0; COEFFICIENT_BYTES]);
        for (index, value) in self.0.iter().flatten().enumerate() {
            out[2 * index..2 * index + 2].copy_from_slice(&value.to_le_bytes());
        }
        out
    }
}
impl fmt::Debug for Coefficients {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Coefficients([REDACTED])")
    }
}
