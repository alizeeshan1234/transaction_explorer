use anchor_lang::prelude::*;

use ephemeral_rollups_sdk::anchor::delegate;
use ephemeral_rollups_sdk::cpi::DelegateConfig;

use crate::state::market::Market;
use crate::MARKET_SEED;

#[delegate]
#[derive(Accounts)]
pub struct DelegateMarket<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    ///CHECK: The target custody account is used only for deriving the custody PDA
    #[account()]
    pub target_custody: UncheckedAccount<'info>,

    ///CHECK: The target custody account is used only for deriving the custody PDA
    #[account()]
    pub lock_custody: UncheckedAccount<'info>,

    #[account(
        mut,
        del,
        seeds = [MARKET_SEED, target_custody.key().as_ref(), lock_custody.key().as_ref(), &[market.side as u8]],
        bump = market.market_bump
    )]
    pub market: Account<'info, Market>,
}

pub fn handler(
    ctx: Context<DelegateMarket>,
    commit_frequency: u32,
    validator_key: Pubkey,
) -> Result<()> {
    let delegate_config = DelegateConfig {
        commit_frequency_ms: commit_frequency,
        validator: Some(validator_key),
    };

    let target_custody_key = ctx.accounts.target_custody.key();
    let lock_custody_key = ctx.accounts.lock_custody.key();
    let seeds = &[
        MARKET_SEED,
        target_custody_key.as_ref(),
        lock_custody_key.as_ref(),
        &[ctx.accounts.market.side as u8],
    ];

    ctx.accounts
        .delegate_market(&ctx.accounts.payer, seeds, delegate_config)?;

    Ok(())
}
