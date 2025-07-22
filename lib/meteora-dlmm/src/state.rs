#[derive(Copy, Clone, Debug, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
/// Type of the Pair. 0 = Permissionless, 1 = Permission. Putting 0 as permissionless for backward compatibility.
pub enum PairType {
    Permissionless,
    Permission,
}

pub struct LaunchPadParams {
    pub activation_slot: u64,
    pub swap_cap_deactivate_slot: u64,
    pub max_swapped_amount: u64,
}

impl PairType {
    pub fn get_pair_default_launch_pad_params(&self) -> LaunchPadParams {
        match self {
            // The slot is unreachable. Therefore by default, the pair will be disabled until admin update the activation slot.
            Self::Permission => LaunchPadParams {
                activation_slot: u64::MAX,
                swap_cap_deactivate_slot: u64::MAX,
                max_swapped_amount: u64::MAX,
            },
            // Activation slot is not used in permissionless pair. Therefore, default to 0.
            Self::Permissionless => LaunchPadParams {
                activation_slot: 0,
                swap_cap_deactivate_slot: 0,
                max_swapped_amount: 0,
            },
        }
    }
}

#[derive(
    AnchorSerialize, AnchorDeserialize, Debug, PartialEq, Eq, IntoPrimitive, TryFromPrimitive,
)]
#[repr(u8)]
/// Pair status. 0 = Enabled, 1 = Disabled. Putting 0 as enabled for backward compatibility.
pub enum PairStatus {
    // Fully enabled.
    // Condition:
    // Permissionless: PairStatus::Enabled
    // Permission: PairStatus::Enabled and clock.slot > activation_slot
    Enabled,
    // Similar as emergency mode. User can only withdraw (Only outflow). Except whitelisted wallet still have full privileges.
    Disabled,
}

#[zero_copy]
#[derive(InitSpace, Default, Debug)]
pub struct ProtocolFee {
    pub amount_x: u64,
    pub amount_y: u64,
}


#[derive(InitSpace, Debug)]
pub struct LbPair {
    pub parameters: StaticParameters,
    pub v_parameters: VariableParameters,
    pub bump_seed: [u8; 1],
    /// Bin step signer seed
    pub bin_step_seed: [u8; 2],
    /// Type of the pair
    pub pair_type: u8,
    /// Active bin id
    pub active_id: i32,
    /// Bin step. Represent the price increment / decrement.
    pub bin_step: u16,
    /// Status of the pair. Check PairStatus enum.
    pub status: u8,
    pub _padding1: [u8; 5],
    /// Token X mint
    pub token_x_mint: Pubkey,
    /// Token Y mint
    pub token_y_mint: Pubkey,
    /// LB token X vault
    pub reserve_x: Pubkey,
    /// LB token Y vault
    pub reserve_y: Pubkey,
    /// Uncollected protocol fee
    pub protocol_fee: ProtocolFee,
    /// Protocol fee owner,
    pub fee_owner: Pubkey,
    /// Farming reward information
    pub reward_infos: [RewardInfo; 2], // TODO: Bug in anchor IDL parser when using InitSpace macro. Temp hardcode it. https://github.com/coral-xyz/anchor/issues/2556
    /// Oracle pubkey
    pub oracle: Pubkey,
    /// Packed initialized bin array state
    pub bin_array_bitmap: [u64; 16], // store default bin id from -512 to 511 (bin id from -35840 to 35840, price from 2.7e-16 to 3.6e15)
    /// Last time the pool fee parameter was updated
    pub last_updated_at: i64,
    /// Whitelisted wallet
    pub whitelisted_wallet: [Pubkey; 2],
    /// Base keypair. Only required for permission pair
    pub base_key: Pubkey,
    /// Slot to enable the pair. Only available for permission pair.
    pub activation_slot: u64,
    /// Last slot until pool remove max_swapped_amount for buying
    pub swap_cap_deactivate_slot: u64,
    /// Max X swapped amount user can swap from y to x between activation_slot and last_slot
    pub max_swapped_amount: u64,
    /// Reserved space for future use
    pub _reserved: [u8; 64],
}

impl Default for LbPair {
    fn default() -> Self {
        let LaunchPadParams {
            activation_slot,
            max_swapped_amount,
            swap_cap_deactivate_slot,
        } = PairType::Permissionless.get_pair_default_launch_pad_params();
        Self {
            active_id: 0,
            parameters: StaticParameters::default(),
            v_parameters: VariableParameters::default(),
            bump_seed: [0u8; 1],
            bin_step: 0,
            token_x_mint: Pubkey::default(),
            token_y_mint: Pubkey::default(),
            bin_step_seed: [0u8; 2],
            fee_owner: Pubkey::default(),
            protocol_fee: ProtocolFee::default(),
            reserve_x: Pubkey::default(),
            reserve_y: Pubkey::default(),
            reward_infos: [RewardInfo::default(); 2],
            oracle: Pubkey::default(),
            bin_array_bitmap: [0u64; 16],
            last_updated_at: 0,
            pair_type: PairType::Permissionless.into(),
            status: 0,
            whitelisted_wallet: [Pubkey::default(); 2],
            base_key: Pubkey::default(),
            activation_slot,
            swap_cap_deactivate_slot,
            max_swapped_amount,
            _padding1: [0u8; 5],
            _reserved: [0u8; 64],
        }
    }
}

/// Stores the state relevant for tracking liquidity mining rewards
#[derive(InitSpace, Default, Debug, PartialEq)]
pub struct RewardInfo {
    /// Reward token mint.
    pub mint: Pubkey,
    /// Reward vault token account.
    pub vault: Pubkey,
    /// Authority account that allows to fund rewards
    pub funder: Pubkey,
    /// TODO check whether we need to store it in pool
    pub reward_duration: u64, // 8
    /// TODO check whether we need to store it in pool
    pub reward_duration_end: u64, // 8
    /// TODO check whether we need to store it in pool
    pub reward_rate: u128, // 8
    /// The last time reward states were updated.
    pub last_update_time: u64, // 8
    /// Accumulated seconds where when farm distribute rewards, but the bin is empty. The reward will be accumulated for next reward time window.
    pub cumulative_seconds_with_empty_liquidity_reward: u64,
}