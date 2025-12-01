use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Transfer};

use crate::{
    constants::*, error::PlatformError, state::{basket::Basket, custody::Custody, market::Market, pool::Pool}
};

#[derive(Accounts)]
pub struct RemoveCollateralFromPosition<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        mut,
        seeds = [b"basket", owner.key().as_ref()],
        bump = basket.basket_bump
    )]
    pub basket: Account<'info, Basket>,

    #[account(
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
        seeds = [POOL_SEED, &[pool.id]],
        bump = pool.pool_bump
    )]
    pub pool: Account<'info, Pool>,

    #[account(
        seeds = [
            CUSTODY_SEED, 
            pool.key().as_ref(), 
            &[collateral_custody.id]
        ],
        bump = collateral_custody.custody_bump
    )]
    pub collateral_custody: Account<'info, Custody>,

    #[account(
        mut,
        associated_token::mint = collateral_custody.token_mint,
        associated_token::authority = owner
    )]
    pub owner_collateral_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [TOKEN_ACCOUNT_SEED, collateral_custody.key().as_ref()],
        bump,
    )]
    pub custody_collateral_account: Account<'info, TokenAccount>,

    #[account(
        seeds = [TOKEN_AUTHORITY_SEED],
        bump
    )]
    pub transfer_authority: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
    
}

pub fn handler(ctx: Context<RemoveCollateralFromPosition>, amount: u64) -> Result<()> {

    let basket = &mut ctx.accounts.basket;
    let market_key = ctx.accounts.market.key();

    let position_index = basket
        .get_position_index(&market_key)
        .ok_or(PlatformError::PositionNotFound)?;

    let position = &mut basket.positions[position_index].position;

    if position.collateral_usd < amount {
        return Err(PlatformError::InsufficientCollateral.into());
    }

    position.collateral_usd = position.collateral_usd.saturating_sub(amount);

    let authority_seeds: &[&[&[u8]]] = &[
        &[
            TOKEN_AUTHORITY_SEED,
            &[ctx.bumps.transfer_authority]
        ]
    ];

    let transfer_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.custody_collateral_account.to_account_info(),
            to: ctx.accounts.owner_collateral_account.to_account_info(),
            authority: ctx.accounts.transfer_authority.to_account_info()
        },
        authority_seeds
    );

    anchor_spl::token::transfer(transfer_ctx, amount)?;

    emit!(CollateralRemoved {
        owner: ctx.accounts.owner.key(),
        market: market_key,
        amount,
        remaining_collateral: position.collateral_usd
    });

    Ok(())
}

#[event]
pub struct CollateralRemoved {
    pub owner: Pubkey,
    pub market: Pubkey,
    pub amount: u64,
    pub remaining_collateral: u64,
}