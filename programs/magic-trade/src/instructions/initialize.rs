use anchor_lang::prelude::*;

use crate::{
    constants::{LAMPORT_BANK_SEED, PLATFORM_SEED, TOKEN_AUTHORITY_SEED},
    platform::{Permissions, Platform},
};

#[derive(Accounts)]
pub struct InitializePlatform<'info> {
    #[account(
        init,
        payer = admin,
        space = 8 + Platform::INIT_SPACE,
        seeds = [PLATFORM_SEED],
        bump
    )]
    pub platform: Account<'info, Platform>,
    /// CHECK: empty PDA, will be set as authority for token accounts
    #[account(
        init,
        payer = admin,
        space = 0,
        seeds = [TOKEN_AUTHORITY_SEED],
        bump
    )]
    pub transfer_authority: AccountInfo<'info>,
    /// CHECK: empty PDA, will be set as authority for token accounts
    #[account(
        init,
        payer = admin,
        space = 0,
        seeds = [LAMPORT_BANK_SEED],
        bump
    )]
    pub lamport_bank: AccountInfo<'info>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<InitializePlatform>,
    version: u8,
    permissions: Permissions,
) -> Result<()> {
    let platform_bump = ctx.bumps.platform;

    let platform = &mut ctx.accounts.platform;
    platform.version = version;
    platform.platform_bump = platform_bump;
    platform.token_authority_bump = ctx.bumps.transfer_authority;
    platform.lamport_bank_bump = ctx.bumps.lamport_bank;
    platform.pool_count = 0;
    platform.permissions = permissions;
    platform.admin = ctx.accounts.admin.key();

    Ok(())
}
