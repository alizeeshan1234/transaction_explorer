#![allow(ambiguous_glob_reexports)]

pub mod constants;
pub mod error;
pub mod instructions;
pub mod math;
pub mod state;

use anchor_lang::prelude::*;
use ephemeral_rollups_sdk::anchor::{commit, ephemeral};

pub use constants::*;
pub use instructions::*;
pub use state::*;

use crate::{
    COLLATERAL_PRICE_MAX_AGE, constants::*, error::PlatformError, market::OraclePrice, state::{basket::Basket, custody::Custody, market::Market, pool::Pool}
};

declare_id!("FEAk5DhL8Q1TXDpj6s9tQKNBaAC9aJ8GTSfYfFkMzLSD");

#[ephemeral]
#[program]
pub mod magic_trade {

    use ephemeral_rollups_sdk::{ActionArgs, ShortAccountMeta, ephem::{CallHandler, CommitAndUndelegate, CommitType, MagicAction, MagicInstructionBuilder, UndelegateType, commit_accounts, commit_and_undelegate_accounts}};

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

    pub fn process_add_collateral_to_position(ctx: Context<AddCollateralToPosition>, collateral_amount: u64, size_amount: u64) -> Result<()> {
        add_collateral_to_position::handler(ctx, collateral_amount, size_amount)
    }

    pub fn remove_collateral_from_position(ctx: Context<RemoveCollateralFromPosition>, collateral_amount: u64, size_amount: u64) -> Result<()> {
        remove_collateral_from_position::handler(ctx, collateral_amount, size_amount)
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

    // pub fn commit_and_add_collateral_to_position(ctx: Context<CommitAndAddCollateralToPosition>, collateral_amount: u64, size_amount: u64) -> Result<()> {

    //     let instruction_data = anchor_lang::InstructionData::data(
    //         &crate::instruction::ProcessAddCollateralToPosition{ 
    //             collateral_amount,
    //             size_amount
    //         }
    //     );

    //     let action_args = ActionArgs {
    //         escrow_index: 0,
    //         data: instruction_data
    //     };

    //     let accounts = vec![
    //         ShortAccountMeta {
    //             pubkey: ctx.accounts.owner.key(),
    //             is_writable: false
    //         },
    //         ShortAccountMeta {
    //             pubkey: ctx.accounts.basket.key(),
    //             is_writable: true,
    //         },
    //         ShortAccountMeta {
    //             pubkey: ctx.accounts.market.key(),
    //             is_writable: true,
    //         },
    //         ShortAccountMeta {
    //             pubkey: ctx.accounts.pool.key(),
    //             is_writable: false
    //         },
    //         ShortAccountMeta {
    //             pubkey: ctx.accounts.target_custody.key(),
    //             is_writable: false
    //         },
    //         ShortAccountMeta {
    //             pubkey: ctx.accounts.collateral_custody.key(),
    //             is_writable: true,
    //         },
    //         ShortAccountMeta {
    //             pubkey: ctx.accounts.lock_custody.key(),
    //             is_writable: true
    //         },
    //         ShortAccountMeta {
    //             pubkey: ctx.accounts.target_oracle.key(),
    //             is_writable: false
    //         },
    //         ShortAccountMeta {
    //             pubkey: ctx.accounts.collateral_oracle.key(),
    //             is_writable: false
    //         },
    //         ShortAccountMeta {
    //             pubkey: ctx.accounts.lock_oracle.key(),
    //             is_writable: false
    //         }
    //     ];

    //     let add_collateral_to_position_handler = CallHandler {
    //         args: action_args,
    //         compute_units: 200_000,
    //         escrow_authority: ctx.accounts.owner.to_account_info(),
    //         destination_program: crate::ID,
    //         accounts
    //     };

    //     MagicInstructionBuilder {
    //         payer: ctx.accounts.owner.to_account_info(),
    //         magic_context: ctx.accounts.magic_context.to_account_info(),
    //         magic_program: ctx.accounts.magic_program.to_account_info(),
    //         magic_action: MagicAction::Commit(
    //             CommitType::WithHandler {
    //                 commited_accounts: vec![
    //                     ctx.accounts.basket.to_account_info(),
    //                     ctx.accounts.market.to_account_info(),
    //                     ctx.accounts.pool.to_account_info(),
    //                     ctx.accounts.collateral_custody.to_account_info(),
    //                     ctx.accounts.lock_custody.to_account_info()
    //                 ],
    //                 call_handlers: vec![add_collateral_to_position_handler],
    //             }
    //         )
    //     }.build_and_invoke()?;

    //     Ok(())
    // }

    pub fn process_commit_and_undelegate_accounts(ctx: Context<CommitAndUndelegateAccounts>) -> Result<()> {

        commit_and_undelegate_accounts(
            &ctx.accounts.owner.to_account_info(), 
            vec![
                &ctx.accounts.basket.to_account_info(),
                &ctx.accounts.market.to_account_info(),
                &ctx.accounts.pool.to_account_info(),
                &ctx.accounts.target_custody.to_account_info(),
                &ctx.accounts.collateral_custody.to_account_info(),
            ], 
            &ctx.accounts.magic_context.to_account_info(), 
            &ctx.accounts.magic_program.to_account_info()
        )?;

        msg!("Commit and undelegated accounts successfully!");

        Ok(())
    }

}

/*
Delegated Account: 
Pool
Custody
Market 
Basket
*/

// #[commit]
// #[derive(Accounts)]
// pub struct CommitAndAddCollateralToPosition<'info> {
//     #[account(mut)]
//     pub owner: Signer<'info>,

//     #[account(
//         mut,
//         seeds = [BASKET_SEED, owner.key().as_ref()],
//         bump = basket.basket_bump
//     )]
//     pub basket: Account<'info, Basket>,

//     #[account(
//         mut,
//         seeds = [
//             MARKET_SEED,
//             market.target_custody.key().as_ref(),
//             market.lock_custody.key().as_ref(),
//             &[market.side as u8]
//         ],
//         bump
//     )]
//     pub market: Account<'info, Market>,

//     #[account(
//         seeds = [POOL_SEED, &[pool.id]],
//         bump = pool.pool_bump
//     )]
//     pub pool: Account<'info, Pool>,

//     pub target_custody: Account<'info, Custody>,
    
//     #[account(mut)]
//     pub collateral_custody: Account<'info, Custody>,

//     #[account(mut)]
//     pub lock_custody: Account<'info, Custody>,

//     pub target_oracle: UncheckedAccount<'info>,
//     pub collateral_oracle: UncheckedAccount<'info>,
//     pub lock_oracle: UncheckedAccount<'info>,

//     /// CHECK: Magic context account
//     #[account(mut)]
//     pub magic_context: UncheckedAccount<'info>,
    
//     /// CHECK: Magic program
//     pub magic_program: UncheckedAccount<'info>,
// }

#[commit]
#[derive(Accounts)]
pub struct CommitAndUndelegateAccounts<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        mut,
        seeds = [BASKET_SEED, owner.key().as_ref()],
        bump = basket.basket_bump
    )]
    pub basket: Account<'info, Basket>,

    #[account(
        mut,
        seeds = [
            MARKET_SEED,
            market.target_custody.key().as_ref(),
            market.lock_custody.key().as_ref(),
            &[market.side as u8]
        ],
        bump
    )]
    pub market: Account<'info, Market>,

    #[account(
        mut,
        seeds = [POOL_SEED, &[pool.id]],
        bump = pool.pool_bump
    )]
    pub pool: Account<'info, Pool>,

    pub target_custody: Account<'info, Custody>,
    
    #[account(mut)]
    pub collateral_custody: Account<'info, Custody>,

    #[account(mut)]
    pub lock_custody: Account<'info, Custody>,

    pub target_oracle: UncheckedAccount<'info>,
    pub collateral_oracle: UncheckedAccount<'info>,
    pub lock_oracle: UncheckedAccount<'info>,
}