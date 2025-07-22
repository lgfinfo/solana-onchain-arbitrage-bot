const MAX_EXPONENTIAL: u32 = 0x80000; // 1048576
const SCALE_OFFSET: u32 = 64;

/// Computes `base^exp` in Q64.64 fixed-point format for Solana on-chain programs.
/// Returns `None` on overflow, division by zero, or invalid inputs.
pub fn pow(base: u128, exp: i32) -> Option<u128> {
    // Constants for Q64.64 fixed-point arithmetic
    const ONE: u128 = 1u128 << 64; // 1.0 in Q64.64 format
    const SCALE_OFFSET: u32 = 64; // Right shift to maintain Q64.64 precision
    const MAX_EXPONENTIAL: u32 = 19; // Maximum exponent to prevent overflow

    // Handle negative exponents by computing 1 / base^|exp|
    let mut invert = exp.is_negative();

    // Edge case: exponent = 0 returns 1.0
    if exp == 0 {
        return Some(ONE);
    }

    // Convert exponent to positive u32, handling i32::MIN edge case
    let exp: u32 = if invert {
        if exp == i32::MIN {
            return None; // Absolute value of i32::MIN cannot be represented as u32
        }
        exp.abs() as u32
    } else {
        exp as u32
    };

    // Check for exponent overflow
    if exp >= MAX_EXPONENTIAL {
        return None;
    }

    let mut squared_base = base;
    let mut result = ONE;

    // Invert base if it is >= 1.0 to prevent overflow in multiplications
    // Uses property: base^exp = 1 / (1/base)^exp
    if squared_base >= result {
        squared_base = u128::MAX.checked_div(squared_base)?;
        invert = !invert; // Toggle inversion flag
    }

    // Macro to unroll square-and-multiply algorithm for fixed 19 bits
    macro_rules! pow_bits {
        ($result:expr, $squared_base:expr, $exp:expr, $($bit:expr),*) => {
            $(
                if $exp & (1 << $bit) > 0 {
                    $result = ($result.checked_mul($squared_base)?) >> SCALE_OFFSET;
                }
                $squared_base = ($squared_base.checked_mul($squared_base)?) >> SCALE_OFFSET;
            )*
        };
    }

    // Unroll square-and-multiply for bits 0 to 18 (MAX_EXPONENTIAL = 19)
    pow_bits!(result, squared_base, exp, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18);

    // Return None if result is zero (assumes zero is invalid in this context)
    if result == 0 {
        return None;
    }

    // Apply final inversion for negative exponents
    if invert {
        result = u128::MAX.checked_div(result)?;
    }

    Some(result)
}

/// Unit tests for the pow function
#[cfg(test)]
mod tests {
    use super::*;

    const ONE: u128 = 1u128 << 64; // 1.0 in Q64.64 format

    #[test]
    fn test_zero_exponent() {
        assert_eq!(pow(2u128 << 64, 0), Some(ONE)); // Any base^0 = 1
    }

    #[test]
    fn test_positive_exponent() {
        let base = ONE + (ONE >> 10); // ~1.0009765625 in Q64.64
        let result = pow(base, 2).unwrap();
        assert!(result > ONE); // Should be slightly > 1
    }

    #[test]
    fn test_negative_exponent() {
        let base = ONE + (ONE >> 10); // ~1.0009765625
        let result = pow(base, -1).unwrap();
        assert!(result < ONE); // Should be slightly < 1
    }

    #[test]
    fn test_overflow_exponent() {
        assert_eq!(pow(ONE, 20), None); // Exceeds MAX_EXPONENTIAL
    }

    #[test]
    fn test_division_by_zero() {
        assert_eq!(pow(0, -1), None); // 1/base^1 with base=0
    }

    #[test]
    fn test_min_exponent() {
        assert_eq!(pow(ONE, i32::MIN), None); // i32::MIN cannot be converted to u32
    }
}