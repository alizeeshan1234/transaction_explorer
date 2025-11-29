use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

use crate::{
    basket::Basket,
    constants::{BASKET_SEED, CUSTODY_SEED, POOL_SEED},
    custody::Custody,
    error::PlatformError,
    pool::Pool,
};

#[derive(Accounts)]
pub struct DepositCollateral<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(mut)]
    pub owner_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [POOL_SEED, &[pool.id]],
        bump = pool.pool_bump
    )]
    pub pool: Account<'info, Pool>,

    #[account(
        mut,
        seeds = [CUSTODY_SEED, pool.key().as_ref(), &[custody.id]],
        bump = custody.custody_bump,
        address = pool.custodies[0], // Ensure deposits are made to the first custody
    )]
    pub custody: Account<'info, Custody>,

    #[account(
        mut,
        address = custody.token_account,
    )]
    pub custody_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [BASKET_SEED, owner.key().as_ref()],
        bump = basket.basket_bump
    )]
    pub basket: Account<'info, Basket>,

    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<DepositCollateral>, amount: u64) -> Result<()> {
    require!(amount > 0, PlatformError::InvalidBasketState);

    let cpi_accounts = Transfer {
        from: ctx.accounts.owner_token_account.to_account_info(),
        to: ctx.accounts.custody_token_account.to_account_info(),
        authority: ctx.accounts.owner.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);
    token::transfer(cpi_ctx, amount)?;

    let custody = &mut ctx.accounts.custody;
    custody.assets.reserved = custody.assets.reserved.saturating_add(amount);

    let basket = &mut ctx.accounts.basket;
    basket.process_deposit(ctx.accounts.pool.key(), amount);

    Ok(())
}
