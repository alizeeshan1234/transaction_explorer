#![allow(ambiguous_glob_reexports)]

pub mod constants;
pub mod error;
pub mod instructions;
pub mod math;
pub mod state;

use anchor_lang::prelude::*;
use ephemeral_rollups_sdk::anchor::ephemeral;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("92qkdfRpsQxkuc9DnJ4oXGkLLW3xeSTYwQGRBGLD8ZaT");

#[ephemeral]
#[program]
pub mod magic_trade {

    use super::*;

    pub fn initialize(
        ctx: Context<InitializePlatform>,
        version: u8,
        permissions: platform::Permissions,
    ) -> Result<()> {
        initialize::handler(ctx, version, permissions)
    }

    pub fn initialize_pool(
        ctx: Context<InitializePool>,
        pool_id: u8,
        max_aum_usd: u64,
        buffer: u64,
        collateral_oracle: Pubkey,
    ) -> Result<()> {
        initialize_pool::handler(ctx, pool_id, max_aum_usd, buffer, collateral_oracle)
    }

    pub fn initialize_custody(
        ctx: Context<InitializeCustody>,
        custody_id: u8,
        decimals: u8,
        stablecoin: bool,
        is_virtual: bool,
        permissions: platform::Permissions,
        max_price_age: u64,
        margin_params: custody::MarginParams,
    ) -> Result<()> {
        initialize_custody::handler(
            ctx,
            custody_id,
            decimals,
            stablecoin,
            is_virtual,
            permissions,
            max_price_age,
            margin_params,
        )
    }

    pub fn initialize_market(
        ctx: Context<InitializeMarket>,
        market_id: u8,
        side: market::Side,
        is_virtual: bool,
        permissions: platform::Permissions,
    ) -> Result<()> {
        initialize_market::handler(ctx, market_id, side, is_virtual, permissions)
    }

    pub fn add_liquidity<'info>(
        ctx: Context<'_, '_, 'info, 'info, AddLiquidity<'info>>,
        amount: u64,
    ) -> Result<()> {
        add_liquidity::handler(ctx, amount)
    }

    pub fn remove_liquidity<'info>(
        ctx: Context<'_, '_, 'info, 'info, RemoveLiquidity<'info>>,
        amount: u64,
    ) -> Result<()> {
        remove_liquidity::handler(ctx, amount)
    }

    pub fn initialize_basket(ctx: Context<InitializeBasket>) -> Result<()> {
        initialize_basket::handler(ctx)
    }

    pub fn deposit_collateral(ctx: Context<DepositCollateral>, amount: u64) -> Result<()> {
        deposit_collateral::handler(ctx, amount)
    }

    pub fn withdraw_collateral(ctx: Context<WithdrawCollateral>, amount: u64) -> Result<()> {
        withdraw_collateral::handler(ctx, amount)
    }

    pub fn liquidate_position(ctx: Context<LiquidatePosition>) -> Result<()> {
        liquidate_position::handler(ctx)
    }

    pub fn open_position(
        ctx: Context<OpenPosition>,
        collateral_amount: u64,
        size_amount: u64,
    ) -> Result<()> {
        open_position::handler(ctx, collateral_amount, size_amount)
    }

    pub fn close_position(ctx: Context<ClosePosition>) -> Result<()> {
        close_position::handler(ctx)
    }

    pub fn delegate_pool(
        ctx: Context<DelegatePool>,
        commit_frequency: u32,
        validator_key: Pubkey,
    ) -> Result<()> {
        delegate_pool::handler(ctx, commit_frequency, validator_key)
    }

    pub fn delegate_custody(
        ctx: Context<DelegateCustody>,
        commit_frequency: u32,
        validator_key: Pubkey,
    ) -> Result<()> {
        delegate_custody::handler(ctx, commit_frequency, validator_key)
    }

    pub fn delegate_market(
        ctx: Context<DelegateMarket>,
        commit_frequency: u32,
        validator_key: Pubkey,
    ) -> Result<()> {
        delegate_market::handler(ctx, commit_frequency, validator_key)
    }

    pub fn delegate_basket(
        ctx: Context<DelegateBasket>,
        commit_frequency: u32,
        validator_key: Pubkey,
    ) -> Result<()> {
        delegate_basket::handler(ctx, commit_frequency, validator_key)
    }
}
