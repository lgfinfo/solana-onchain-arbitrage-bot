use uint::construct_uint;
use solana_program::pubkey::Pubkey;
use solana_program::program_error::ProgramError;
const PRECISION: u64 = 1_000_000_000; // 用于滑点计算

construct_uint! {
    pub struct U128(2);
}

construct_uint! {
    pub struct U256(4);
}

// 计算费用的辅助函数
fn fee(amount: &U256, fee_bps: &U256) -> U256 {
    let fee_amount = (amount * fee_bps) / U256::from(10_000u64);
    fee_amount
}

// 计算购买基础代币的输入量
pub fn buy_base_input_internal(
    base: &U256,
    slippage: f64, // 1 => 1%
    base_reserve: &U256,
    quote_reserve: &U256,
    lp_fee_bps: &U256,
    protocol_fee_bps: &U256,
    coin_creator_fee_bps: &U256,
    coin_creator: &Pubkey,
) -> Result<BuyBaseInputResult, ProgramError> {
    // 基本验证
    if base_reserve.is_zero() || quote_reserve.is_zero() {
        return Err(ProgramError::Custom(1));
    }

    if base > base_reserve {
        return Err(ProgramError::Custom(1));
    }

    // 计算需要的原始报价（Raydium 类似公式）
    let numerator = quote_reserve * base;
    let denominator = base_reserve - base;

    if denominator.is_zero() {
        return Err(ProgramError::Custom(1));
    }

    let quote_amount_in = ceil_div(&numerator, &denominator);

    // 计算费用
    let lp_fee = fee(&quote_amount_in, lp_fee_bps);
    let protocol_fee = fee(&quote_amount_in, protocol_fee_bps);
    let coin_creator_fee = if coin_creator == &Pubkey::default() {
        U256::zero()
    } else {
        fee(&quote_amount_in, coin_creator_fee_bps)
    };

    let total_quote = &quote_amount_in + &lp_fee + &protocol_fee + &coin_creator_fee;

    // 计算包含滑点的最大报价
    let slippage_factor = ((1.0 + slippage / 100.0) * PRECISION as f64).round() as u64;
    let max_quote = (total_quote * U256::from(slippage_factor)) / U256::from(PRECISION);

    Ok(BuyBaseInputResult {
        internal_quote_amount: quote_amount_in,
        ui_quote: total_quote,
        max_quote,
    })
}

// 计算购买报价的输入量
pub fn buy_quote_input_internal(
    quote: &U256,
    slippage: f64, // 1 => 1%
    base_reserve: &U256,
    quote_reserve: &U256,
    lp_fee_bps: &U256,
    protocol_fee_bps: &U256,
    coin_creator_fee_bps: &U256,
    coin_creator: &Pubkey,
) -> Result<BuyQuoteInputResult, ProgramError> {
    // 基本验证
    if base_reserve.is_zero() || quote_reserve.is_zero() {
        return Err(ProgramError::Custom(1));
    }

    // 计算总费用基础点和分母
    let total_fee_bps = lp_fee_bps
        + protocol_fee_bps
        + if coin_creator == &Pubkey::default() {
            U256::zero()
        } else {
            coin_creator_fee_bps.clone()
        };
    let denominator = U256::from(10_000u64) + total_fee_bps;

    // 计算有效报价
    let effective_quote = (quote * U256::from(10_000u64)) / denominator;

    // 计算获得的基础代币数量
    let numerator = base_reserve * effective_quote;
    let denominator_effective = quote_reserve + effective_quote;

    if denominator_effective.is_zero() {
        return Err(ProgramError::Custom(1));
    }

    let base_amount_out = numerator / denominator_effective;

    // 计算包含滑点的最大报价
    let slippage_factor = ((1.0 + slippage / 100.0) * PRECISION as f64).round() as u64;
    let max_quote = (quote * U256::from(slippage_factor)) / U256::from(PRECISION);

    Ok(BuyQuoteInputResult {
        base: base_amount_out,
        internal_quote_without_fees: effective_quote,
        max_quote,
    })
}

pub fn sell_quote_input_internal(
    quote: &U256,
    slippage: f64,
    base_reserve: &U256,
    quote_reserve: &U256,
    lp_fee_bps: &U256,
    protocol_fee_bps: &U256,
    coin_creator_fee_bps: &U256,
    coin_creator: &Pubkey,
) -> Result<SellQuoteInputResult, ProgramError> {
    if base_reserve.is_zero() || quote_reserve.is_zero() {
        return Err(ProgramError::Custom(1));
    }
    if quote > quote_reserve {
        return Err(ProgramError::Custom(1));
    }

    // 1. 总费用反推 quote_raw
    let total_fee_bps = lp_fee_bps + protocol_fee_bps + if coin_creator == &Pubkey::default() {
        U256::zero()
    } else {
        coin_creator_fee_bps.clone()
    };
    
    let max_fee_bps = U256::from(10_000u64);
    let denominator = max_fee_bps - total_fee_bps;

    let raw_quote = ceil_div(&(quote * max_fee_bps), &denominator);

    if raw_quote >= *quote_reserve {
        return Err(ProgramError::Custom(1));
    }

    // 2. base_amount_in = ceil((baseReserve * rawQuote) / (quoteReserve - rawQuote))
    let base_amount_in = ceil_div(
        &(base_reserve * raw_quote),
        &(quote_reserve - raw_quote),
    );

    // 3. 滑点 min_quote
    let slippage_factor = ((1.0 - slippage / 100.0) * PRECISION as f64).round() as u64;
    let min_quote = (quote * U256::from(slippage_factor)) / U256::from(PRECISION);

    Ok(SellQuoteInputResult {
        base: base_amount_in,
        internal_raw_quote: raw_quote,
        min_quote,
    })
}

pub fn sell_base_input_internal(
    base: &U256,
    slippage: f64,
    base_reserve: &U256,
    quote_reserve: &U256,
    lp_fee_bps: &U256,
    protocol_fee_bps: &U256,
    coin_creator_fee_bps: &U256,
    coin_creator: &Pubkey,
) -> Result<SellBaseInputResult, ProgramError> {
    // 1. 基础验证
    if base_reserve.is_zero() || quote_reserve.is_zero() {
        return Err(ProgramError::Custom(1));
    }

    // 2. quote_amount_out = floor((quoteReserve * base) / (baseReserve + base))
    let numerator = quote_reserve * base;
    let denominator = base_reserve + base;
    let quote_amount_out = numerator / denominator;

    // 3. 计算费用
    let lp_fee = fee(&quote_amount_out, lp_fee_bps);
    let protocol_fee = fee(&quote_amount_out, protocol_fee_bps);
    let coin_creator_fee = if coin_creator == &Pubkey::default() {
        U256::zero()
    } else {
        fee(&quote_amount_out, coin_creator_fee_bps)
    };

    let total_fee = &lp_fee + &protocol_fee + &coin_creator_fee;
    if quote_amount_out < total_fee {
        return Err(ProgramError::Custom(1));
    }

    let final_quote = quote_amount_out - total_fee;

    // 4. 滑点 min_quote = final_quote * (1 - slippage)
    let slippage_factor = ((1.0 - slippage / 100.0) * PRECISION as f64).round() as u64;
    let min_quote = (final_quote * U256::from(slippage_factor)) / U256::from(PRECISION);

    Ok(SellBaseInputResult {
        ui_quote: final_quote,
        min_quote,
        internal_quote_amount_out: quote_amount_out,
    })
}

// 辅助函数：计算向上取整的除法
fn ceil_div(numerator: &U256, denominator: &U256) -> U256 {
    let result = numerator / denominator;
    if numerator % denominator != U256::zero() {
        result + U256::one()
    } else {
        result
    }
}

// 结果结构体定义
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuyBaseInputResult {
    pub internal_quote_amount: U256,
    pub ui_quote: U256,
    pub max_quote: U256,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuyQuoteInputResult {
    pub base: U256,
    pub internal_quote_without_fees: U256,
    pub max_quote: U256,
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SellBaseInputResult {
    pub ui_quote: U256,
    pub min_quote: U256,
    pub internal_quote_amount_out: U256,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SellQuoteInputResult {
    pub base: U256,
    pub internal_raw_quote: U256,
    pub min_quote: U256,
}