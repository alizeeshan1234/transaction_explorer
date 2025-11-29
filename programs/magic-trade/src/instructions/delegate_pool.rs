use anchor_lang::prelude::*;

use ephemeral_rollups_sdk::anchor::delegate;
use ephemeral_rollups_sdk::cpi::DelegateConfig;

use crate::state::pool::Pool;
use crate::POOL_SEED;

#[delegate]
#[derive(Accounts)]
pub struct DelegatePool<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        mut,
        del,
        seeds = [POOL_SEED, &[pool.id]],
        bump
    )]
    pub pool: Account<'info, Pool>,
}

pub fn handler(
    ctx: Context<DelegatePool>,
    commit_frequency: u32,
    validator_key: Pubkey,
) -> Result<()> {
    let delegate_config = DelegateConfig {
        commit_frequency_ms: commit_frequency,
        validator: Some(validator_key),
    };
    let pool = &ctx.accounts.pool;
    let seeds = &[POOL_SEED, &[pool.id]];

    ctx.accounts
        .delegate_pool(&ctx.accounts.payer, seeds, delegate_config)?;

    Ok(())
}
