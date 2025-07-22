use solana_program::program_error::ProgramError;

#[inline]
pub fn safe_mul_shr_cast<T: num_traits::FromPrimitive>(
    x: u128,
    y: u128,
    offset: u8,
    rounding: Rounding,
) -> Result<T, ProgramError> {
    mul_shr(x, y, offset, rounding)
        .map_err(|_| ProgramError::ArithmeticOverflow)
        .and_then(|result| T::from_u128(result).ok_or(ProgramError::ArithmeticOverflow))
}

#[inline]
pub fn safe_shl_div_cast<T: num_traits::FromPrimitive>(
    x: u128,
    y: u128,
    offset: u8,
    rounding: Rounding,
) -> Result<T, ProgramError> {
    shl_div(x, y, offset, rounding)
        .map_err(|_| ProgramError::ArithmeticOverflow)
        .and_then(|result| T::from_u128(result).ok_or(ProgramError::ArithmeticOverflow))
}

#[inline]
pub fn safe_mul_div_cast<T: num_traits::FromPrimitive>(
    x: u128,
    y: u128,
    denominator: u128,
    rounding: Rounding,
) -> Result<T, ProgramError> {
    mul_div(x, y, denominator, rounding)
        .map_err(|_| ProgramError::ArithmeticOverflow)
        .and_then(|result| T::from_u128(result).ok_or(ProgramError::ArithmeticOverflow))
}