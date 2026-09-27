use crate::RelayId;
use core::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Error {
    Length {
        item: &'static str,
        expected: usize,
        actual: usize,
    },
    NonCanonicalField {
        element: usize,
    },
    InvalidRelayId(u8),
    LabelMismatch {
        authenticated: RelayId,
        claimed: RelayId,
    },
    DuplicateRelay(RelayId),
    InvalidByteCandidate,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Length {
                item,
                expected,
                actual,
            } => write!(f, "{item}: expected {expected} bytes, got {actual}"),
            Self::NonCanonicalField { element } => {
                write!(f, "noncanonical GF(257) element at index {element}")
            }
            Self::InvalidRelayId(id) => write!(f, "relay label {id} is outside 1..=8"),
            Self::LabelMismatch {
                authenticated,
                claimed,
            } => write!(
                f,
                "authenticated relay {} differs from claimed relay {}",
                authenticated.get(),
                claimed.get()
            ),
            Self::DuplicateRelay(id) => write!(f, "duplicate relay {}", id.get()),
            Self::InvalidByteCandidate => {
                f.write_str("reconstruction contains a field value that is not a byte")
            }
        }
    }
}
impl core::error::Error for Error {}

#[derive(Debug)]
#[non_exhaustive]
pub enum RandomError<E> {
    Source(E),
    RejectionLimit,
}
impl<E: fmt::Display> fmt::Display for RandomError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Source(e) => write!(f, "random source failed: {e}"),
            Self::RejectionLimit => {
                f.write_str("random source exceeded the rejection-sampling limit")
            }
        }
    }
}
impl<E: core::error::Error + 'static> core::error::Error for RandomError<E> {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::Source(e) => Some(e),
            Self::RejectionLimit => None,
        }
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum ConfirmationError<E> {
    InsufficientShares { received: usize },
    NoConfirmedCandidate,
    AmbiguousConfirmation,
    Verifier(E),
}
impl<E: fmt::Display> fmt::Display for ConfirmationError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InsufficientShares { received } => {
                write!(f, "need four distinct shares, received {received}")
            }
            Self::NoConfirmedCandidate => f.write_str("no candidate confirmed"),
            Self::AmbiguousConfirmation => f.write_str("multiple distinct candidates confirmed"),
            Self::Verifier(e) => write!(f, "confirmation verifier failed: {e}"),
        }
    }
}
impl<E: core::error::Error + 'static> core::error::Error for ConfirmationError<E> {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        if let Self::Verifier(e) = self {
            Some(e)
        } else {
            None
        }
    }
}
