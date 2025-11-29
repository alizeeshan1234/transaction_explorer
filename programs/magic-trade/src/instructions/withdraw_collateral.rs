use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

use crate::{
    basket::Basket,
    constants::{BASKET_SEED, CUSTODY_SEED, POOL_SEED, TOKEN_AUTHORITY_SEED},
    custody::Custody,
    error::PlatformError,
    pool::Pool,
};

#[derive(Accounts)]
pub struct WithdrawCollateral<'info> {
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
        address = pool.custodies[0]
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

    /// CHECK: Token authority PDA
    #[account(
        seeds = [TOKEN_AUTHORITY_SEED],
        bump
    )]
    pub token_authority: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<WithdrawCollateral>, amount: u64) -> Result<()> {
    require!(amount > 0, PlatformError::InvalidBasketState);

    require!(
        ctx.accounts
            .basket
            .get_deposit_amount(&ctx.accounts.pool.key())
            >= amount,
        PlatformError::InvalidBasketState
    );
    require!(
        ctx.accounts.custody.assets.reserved >= amount,
        PlatformError::InvalidCustodyState
    );

    let authority_seeds: &[&[&[u8]]] = &[&[TOKEN_AUTHORITY_SEED, &[ctx.bumps.token_authority]]];
    let cpi_accounts = Transfer {
        from: ctx.accounts.custody_token_account.to_account_info(),
        to: ctx.accounts.owner_token_account.to_account_info(),
        authority: ctx.accounts.token_authority.to_account_info(),
    };
    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        cpi_accounts,
        authority_seeds,
    );
    token::transfer(cpi_ctx, amount)?;

    let custody = &mut ctx.accounts.custody;
    custody.assets.reserved = custody.assets.reserved.saturating_sub(amount);

    let basket = &mut ctx.accounts.basket;
    basket.process_withdrawal(ctx.accounts.pool.key(), amount);

    Ok(())
}
