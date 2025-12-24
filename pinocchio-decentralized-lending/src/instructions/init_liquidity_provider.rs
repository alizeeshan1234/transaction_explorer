use pinocchio::{account_info::AccountInfo, program_error::ProgramError, pubkey::{self, Pubkey}, sysvars::{clock::Clock, rent::Rent, Sysvar}, ProgramResult};
use pinocchio_system::instructions::CreateAccount;
use crate::states::{LiquidityPool, LiquidityProviderInfo};

pub fn initialize_liquidity_provider(accounts: &[AccountInfo]) -> ProgramResult {
    let [provider, liquidity_pool, provider_info_account, system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !provider.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let (expected_pda, bump) = pubkey::find_program_address(
        &[
            b"liquidity_provider_info",
            liquidity_pool.key().as_ref(),
            provider.key().as_ref()
        ],
        &crate::ID,
    );

    let liquidity_pool_account_info = LiquidityPool::get_account_info(liquidity_pool);

    let (liquidity_pool_pda, liquidity_pool_bump) = pubkey::find_program_address(
        &[b"liquidity_pool",  liquidity_pool_account_info.mint_a.as_ref(), liquidity_pool_account_info.mint_b.as_ref(), liquidity_pool_account_info.authority.as_ref()],
        &crate::ID
    );

    if *liquidity_pool.key() != liquidity_pool_pda {
        return Err(ProgramError::InvalidSeeds);
    }

    if *provider_info_account.key() != expected_pda {
        return Err(ProgramError::InvalidSeeds);
    }

    if provider_info_account.owner() != &crate::ID {

        let lamports = Rent::get()?.minimum_balance(LiquidityProviderInfo::LEN);

        CreateAccount {
            from: provider,
            to: provider_info_account,
            lamports,
            space: LiquidityProviderInfo::LEN as u64,
            owner: &crate::ID,
        }
        .invoke()?;

        let provider_info_mut = LiquidityProviderInfo::get_account_info_mut(provider_info_account)?;

        *provider_info_mut = LiquidityProviderInfo {
            provider: *provider.key(),
            liquidity_pool: *liquidity_pool.key(),
            provided_token_a: 0,
            provided_token_b: 0,
            total_liquidity_provided: 0,
            total_lp_tokens: 0,
        };
    }

    Ok(())
}
