use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Transfer};

use crate::{
    constants::*, error::PlatformError, state::{basket::Basket, custody::Custody, market::Market, pool::Pool}
};

#[derive(Accounts)]
pub struct AddCollateralToPosition<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        mut,
        seeds = [BASKET_SEED, owner.key().as_ref()],
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

pub fn handler(ctx: Context<AddCollateralToPosition>, amount: u64) -> Result<()> {

    let basket = &mut ctx.accounts.basket;
    let market_key = ctx.accounts.market.key();

    let position_index = basket
        .get_position_index(&market_key)
        .ok_or(PlatformError::PositionNotFound)?;

    basket.positions[position_index].position.collateral_usd = 
        basket.positions[position_index]
            .position
            .collateral_usd
            .checked_add(amount)
            .ok_or(PlatformError::MathError)?;

    let transfer_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.owner_collateral_account.to_account_info(),
            to: ctx.accounts.custody_collateral_account.to_account_info(),
            authority: ctx.accounts.owner.to_account_info()
        },
    );

    anchor_spl::token::transfer(transfer_ctx, amount)?;

    emit!(CollateralAdded {
        owner: ctx.accounts.owner.key(),
        market: market_key,
        amount,
        new_collateral: basket.positions[position_index].position.collateral_usd
    });

    Ok(())
}

#[event]
pub struct CollateralAdded {
    pub owner: Pubkey,
    pub market: Pubkey,
    pub amount: u64,
    pub new_collateral: u64,
}