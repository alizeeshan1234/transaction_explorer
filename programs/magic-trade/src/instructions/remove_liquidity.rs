use crate::{
    constants::{CUSTODY_SEED, LP_MINT_SEED, POOL_SEED},
    custody::Custody,
    error::PlatformError,
    market::OraclePrice,
    math,
    pool::Pool,
    TOKEN_AUTHORITY_SEED,
};
use anchor_lang::prelude::*;
use anchor_spl::token::{Burn, Mint, Token, TokenAccount, Transfer};

#[derive(Accounts)]
pub struct RemoveLiquidity<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(mut)]
    pub owner_lp_account: Account<'info, TokenAccount>,

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
        bump = custody.custody_bump
    )]
    pub custody: Account<'info, Custody>,

    /// CHECK: Oracle account is validated by address
    #[account(
        address = custody.oracle,
    )]
    pub oracle: UncheckedAccount<'info>,

    #[account(
        mut,
        address = custody.token_account,
    )]
    pub custody_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [LP_MINT_SEED, &[pool.id]],
        bump = pool.lp_mint_bump
    )]
    pub lp_token_mint: Account<'info, Mint>,

    /// CHECK: Token authority PDA
    #[account(
        seeds = [TOKEN_AUTHORITY_SEED],
        bump
    )]
    pub token_authority: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,

    /*
    Remaining Accounts:
    [Custodies, Oracles, Markets...] 
    */
}

pub fn handler<'info>(
    ctx: Context<'_, '_, 'info, 'info, RemoveLiquidity<'info>>,
    amount: u64,
) -> Result<()> {
    let custody = &mut ctx.accounts.custody;
    let pool = &mut ctx.accounts.pool;

    let curtime = Clock::get()?.unix_timestamp;

    if curtime.saturating_sub(pool.last_updated_at) >= pool.staleness_threshold as i64 {
        pool.update_aum(&ctx.remaining_accounts, curtime)?;
    }

    let remove_usd = math::checked_as_u64(math::checked_div(
        math::checked_mul(pool.equity_usd as u128, amount as u128)?,
        ctx.accounts.lp_token_mint.supply as u128,
    )?)?;

    // Burn LP tokens from owner
    let cpi_accounts = Burn {
        mint: ctx.accounts.lp_token_mint.to_account_info(),
        from: ctx.accounts.owner_lp_account.to_account_info(),
        authority: ctx.accounts.owner.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    anchor_spl::token::burn(cpi_ctx, amount)?;

    // let fees = pool.get_fee_value(custody.lp_fee, amount)?;

    let withdraw_amount =
        OraclePrice::from_pyth(&ctx.accounts.oracle, curtime, custody.max_price_age as i64)?
            .get_token_amount(remove_usd, custody.decimals)?;

    let withdraw_amount =
        withdraw_amount.saturating_sub(pool.get_fee_value(custody.lp_fee, withdraw_amount)?);

    // Transfer tokens to owner
    let cpi_accounts = Transfer {
        from: ctx.accounts.custody_token_account.to_account_info(),
        to: ctx.accounts.owner_token_account.to_account_info(),
        authority: ctx.accounts.token_authority.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let authotiy_seeds: &[&[&[u8]]] = &[&[TOKEN_AUTHORITY_SEED, &[ctx.bumps.token_authority]]];
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, authotiy_seeds);
    anchor_spl::token::transfer(cpi_ctx, withdraw_amount)?;

    pool.raw_aum_usd = pool.raw_aum_usd.saturating_sub(remove_usd);
    pool.equity_usd = pool.equity_usd.saturating_sub(remove_usd);

    // Update custody assets
    custody.assets.owned = custody.assets.owned.saturating_sub(amount);

    Ok(())
}
