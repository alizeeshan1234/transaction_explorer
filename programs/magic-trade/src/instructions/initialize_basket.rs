use anchor_lang::prelude::*;

use crate::{basket::Basket, constants::BASKET_SEED};

#[derive(Accounts)]
pub struct InitializeBasket<'info> {
    #[account(
        init,
        payer = owner,
        space = 8 + Basket::INIT_SPACE,
        seeds = [BASKET_SEED, owner.key().as_ref()],
        bump
    )]
    pub basket: Account<'info, Basket>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<InitializeBasket>) -> Result<()> {
    let basket = &mut ctx.accounts.basket;
    basket.basket_bump = ctx.bumps.basket;

    Ok(())
}
