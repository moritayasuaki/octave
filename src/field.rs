// Every operand is a canonical 0..=256 value, validated at the byte boundary.
pub(crate) const MODULUS: u16 = 257;
pub(crate) fn add(a: u16, b: u16) -> u16 {
    (a + b) % MODULUS
}
pub(crate) fn sub(a: u16, b: u16) -> u16 {
    (a + MODULUS - b) % MODULUS
}
pub(crate) fn mul(a: u16, b: u16) -> u16 {
    ((a as u32 * b as u32) % MODULUS as u32) as u16
}
pub(crate) fn inv(a: u16) -> u16 {
    // a^255 = a^-1 in GF(257). All inversions are of public, distinct-label differences.
    debug_assert_ne!(a, 0);
    let mut result = 1;
    for _ in 0..8 {
        result = mul(mul(result, result), a);
    }
    result
}

/// There are exactly 255 accepted 16-bit words for each of the 257 results.
pub(crate) fn sample_word(word: u16) -> Option<u16> {
    (word != u16::MAX).then_some(word % MODULUS)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn inverses_and_exhaustive_sampler_balance() {
        for x in 1..MODULUS {
            assert_eq!(mul(x, inv(x)), 1);
        }
        let mut counts = [0; 257];
        for word in 0..=u16::MAX {
            if let Some(value) = sample_word(word) {
                counts[value as usize] += 1;
            }
        }
        assert_eq!(counts, [255; 257]);
        assert_eq!(sample_word(u16::MAX), None);
    }
}
