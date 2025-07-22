const Q64_RESOLUTION: f64 = 18446744073709551616.0;

/// # Returns
/// * `f64` - The decimal price
pub fn sqrt_price_to_price(sqrt_price: U128, decimals_a: u8, decimals_b: u8) -> f64 {
    let power = pow(10f64, decimals_a as f64 - decimals_b as f64);
    let sqrt_price: u128 = sqrt_price.into();
    let sqrt_price_u128 = sqrt_price as f64;
    pow(sqrt_price_u128 / Q64_RESOLUTION, 2.0) * power
}

/// Invert a price
/// IMPORTANT: floating point operations can reduce the precision of the result.
/// Make sure to do these operations last and not to use the result for further calculations.
///
/// # Parameters
/// * `price` - The price to invert
/// * `decimals_a` - The number of decimals of the base token
/// * `decimals_b` - The number of decimals of the quote token
///
/// # Returns
/// * `f64` - The inverted price
#[cfg_attr(feature = "wasm", wasm_expose)]
pub fn invert_price(price: f64, decimals_a: u8, decimals_b: u8) -> f64 {
    let tick_index = price_to_tick_index(price, decimals_a, decimals_b);
    let inverted_tick_index = invert_tick_index(tick_index);
    tick_index_to_price(inverted_tick_index, decimals_a, decimals_b)
}


/// Get the tick index for the inverse of the price that this tick represents.
/// Eg: Consider tick i where Pb/Pa = 1.0001 ^ i
/// inverse of this, i.e. Pa/Pb = 1 / (1.0001 ^ i) = 1.0001^-i
///
/// # Parameters
/// - `tick_index` - A i32 integer representing the tick integer
///
/// # Returns
/// - A i32 integer representing the tick index for the inverse of the price
#[cfg_attr(feature = "wasm", wasm_expose)]
pub fn invert_tick_index(tick_index: i32) -> i32 {
    -tick_index
}


#[cfg_attr(feature = "wasm", wasm_expose)]
pub fn sqrt_price_to_tick_index(sqrt_price: U128) -> i32 {
    let sqrt_price_x64: u128 = sqrt_price.into();
    // Determine log_b(sqrt_ratio). First by calculating integer portion (msb)
    let msb: u32 = 128 - sqrt_price_x64.leading_zeros() - 1;
    let log2p_integer_x32 = (msb as i128 - 64) << 32;

    // get fractional value (r/2^msb), msb always > 128
    // We begin the iteration from bit 63 (0.5 in Q64.64)
    let mut bit: i128 = 0x8000_0000_0000_0000i128;
    let mut precision = 0;
    let mut log2p_fraction_x64 = 0;

    // Log2 iterative approximation for the fractional part
    // Go through each 2^(j) bit where j < 64 in a Q64.64 number
    // Append current bit value to fraction result if r^2 Q2.126 is more than 2
    let mut r = if msb >= 64 {
        sqrt_price_x64 >> (msb - 63)
    } else {
        sqrt_price_x64 << (63 - msb)
    };

    while bit > 0 && precision < BIT_PRECISION {
        r *= r;
        let is_r_more_than_two = r >> 127_u32;
        r >>= 63 + is_r_more_than_two;
        log2p_fraction_x64 += bit * is_r_more_than_two as i128;
        bit >>= 1;
        precision += 1;
    }

    let log2p_fraction_x32 = log2p_fraction_x64 >> 32;
    let log2p_x32 = log2p_integer_x32 + log2p_fraction_x32;

    // Transform from base 2 to base b
    let logbp_x64 = log2p_x32 * LOG_B_2_X32;

    // Derive tick_low & high estimate. Adjust with the possibility of under-estimating by 2^precision_bits/log_2(b) + 0.01 error margin.
    let tick_low: i32 = ((logbp_x64 - LOG_B_P_ERR_MARGIN_LOWER_X64) >> 64) as i32;
    let tick_high: i32 = ((logbp_x64 + LOG_B_P_ERR_MARGIN_UPPER_X64) >> 64) as i32;

    if tick_low == tick_high {
        tick_low
    } else {
        // If our estimation for tick_high returns a lower sqrt_price than the input
        // then the actual tick_high has to be higher than than tick_high.
        // Otherwise, the actual value is between tick_low & tick_high, so a floor value
        // (tick_low) is returned
        let actual_tick_high_sqrt_price_x64: u128 = tick_index_to_sqrt_price(tick_high).into();
        if actual_tick_high_sqrt_price_x64 <= sqrt_price_x64 {
            tick_high
        } else {
            tick_low
        }
    }
}