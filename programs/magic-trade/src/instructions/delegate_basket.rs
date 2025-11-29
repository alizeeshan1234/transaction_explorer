use anchor_lang::prelude::*;

use ephemeral_rollups_sdk::anchor::delegate;
use ephemeral_rollups_sdk::cpi::DelegateConfig;

use crate::state::basket::Basket;
use crate::BASKET_SEED;

#[delegate]
#[derive(Accounts)]
pub struct DelegateBasket<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        mut,
        del,
        seeds = [BASKET_SEED, owner.key().as_ref()],
        bump = basket.basket_bump
    )]
    pub basket: Account<'info, Basket>,
}

pub fn handler(
    ctx: Context<DelegateBasket>,
    commit_frequency: u32,
    validator_key: Pubkey,
) -> Result<()> {
    let delegate_config = DelegateConfig {
        commit_frequency_ms: commit_frequency,
        validator: Some(validator_key),
    };

    let owner_key = ctx.accounts.owner.key();
    let seeds = &[BASKET_SEED, owner_key.as_ref()];

    ctx.accounts
        .delegate_basket(&ctx.accounts.payer, seeds, delegate_config)?;

    Ok(())
}
