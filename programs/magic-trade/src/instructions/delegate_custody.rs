use anchor_lang::prelude::*;

use ephemeral_rollups_sdk::anchor::delegate;
use ephemeral_rollups_sdk::cpi::DelegateConfig;

use crate::state::custody::Custody;
use crate::CUSTODY_SEED;

#[delegate]
#[derive(Accounts)]
pub struct DelegateCustody<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    ///CHECK: The pool account is used only for deriving the custody PDA
    #[account()]
    pub pool: UncheckedAccount<'info>,

    #[account(
        mut,
        del,
        seeds = [CUSTODY_SEED, pool.key().as_ref(), &[custody.id]],
        bump = custody.custody_bump
    )]
    pub custody: Account<'info, Custody>,
}

pub fn handler(
    ctx: Context<DelegateCustody>,
    commit_frequency: u32,
    validator_key: Pubkey,
) -> Result<()> {
    let delegate_config = DelegateConfig {
        commit_frequency_ms: commit_frequency,
        validator: Some(validator_key),
    };

    let pool_key = ctx.accounts.pool.key();
    let seeds = &[CUSTODY_SEED, pool_key.as_ref(), &[ctx.accounts.custody.id]];

    ctx.accounts
        .delegate_custody(&ctx.accounts.payer, seeds, delegate_config)?;

    Ok(())
}
