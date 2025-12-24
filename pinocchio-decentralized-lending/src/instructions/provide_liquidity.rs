use pinocchio::{
    seeds, account_info::AccountInfo, instruction::Signer, program_error::ProgramError, ProgramResult,
};
use pinocchio_token::{
    instructions::{MintToChecked, TransferChecked},
    state::{Mint, TokenAccount},
};
use crate::states::{LiquidityPool, LiquidityProviderInfo};

pub fn provide_liquidity(accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
    let [
        provider,
        token_mint_a,
        token_mint_b,
        lp_token_mint,
        liquidity_pool,
        liquidity_provider_account,
        provider_token_a_ata,
        provider_token_b_ata,
        token_vault_a,
        token_vault_b,
        provider_lp_mint_ata,
        _remaining @..,
    ] = accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if instruction_data.len() < 16 {
        return Err(ProgramError::InvalidInstructionData);
    }

    let token_a_amount = u64::from_le_bytes(
        instruction_data[0..8].try_into().map_err(|_| ProgramError::InvalidInstructionData)?
    );
    let token_b_amount = u64::from_le_bytes(
        instruction_data[8..16].try_into().map_err(|_| ProgramError::InvalidInstructionData)?
    );

    // Account checks
    if !provider.is_signer() || !liquidity_pool.is_writable() || !liquidity_provider_account.is_writable() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    for acc in [provider_token_a_ata, provider_token_b_ata, token_vault_a, token_vault_b, provider_lp_mint_ata] {
        if !acc.is_writable() {
            return Err(ProgramError::InvalidAccountData);
        }
    }

    if token_a_amount != token_b_amount {
        return Err(ProgramError::InvalidInstructionData);
    }

    let token_mint_a_data = Mint::from_account_info(token_mint_a)?;
    let token_mint_b_data = Mint::from_account_info(token_mint_b)?;

    // Transfer token A
    TransferChecked {
        from: provider_token_a_ata,
        to: token_vault_a,
        mint: token_mint_a,
        authority: provider,
        amount: token_a_amount,
        decimals: token_mint_a_data.decimals(),
    }.invoke()?;

    // Transfer token B
    TransferChecked {
        from: provider_token_b_ata,
        to: token_vault_b,
        mint: token_mint_b,
        authority: provider,
        amount: token_b_amount,
        decimals: token_mint_b_data.decimals(),
    }.invoke()?;

    let lp_token_mint_data = Mint::from_account_info(lp_token_mint)?;
    let liquidity_pool_info = LiquidityPool::get_account_info_mut(liquidity_pool);

    // Determine LP tokens to mint
    let lp_token_to_mint = if lp_token_mint_data.supply() == 0 {
        token_a_amount
    } else {
        let vault_a = TokenAccount::from_account_info(token_vault_a)?;
        let vault_b = TokenAccount::from_account_info(token_vault_b)?;

        let share_a = (token_a_amount * lp_token_mint_data.supply()) / vault_a.amount();
        let share_b = (token_b_amount * lp_token_mint_data.supply()) / vault_b.amount();

        share_a.min(share_b)
    };

    // Signer seeds
    let bump_ref = &[liquidity_pool_info.bump];
    let seeds = seeds!(
        b"liquidity_pool",
        token_mint_a.key().as_ref(),
        token_mint_b.key().as_ref(),
        liquidity_pool_info.authority.as_ref(),
        bump_ref
    );
    let signer_seeds = Signer::from(&seeds);

    // Mint LP tokens to provider
    MintToChecked {
        mint: lp_token_mint,
        account: provider_lp_mint_ata,
        mint_authority: liquidity_pool,
        amount: lp_token_to_mint,
        decimals: lp_token_mint_data.decimals(),
    }.invoke_signed(&[signer_seeds])?;

    // Update pool state
    liquidity_pool_info.total_liquidity = liquidity_pool_info
        .total_liquidity
        .checked_add(token_a_amount + token_b_amount)
        .ok_or(ProgramError::InvalidInstructionData)?;

    liquidity_pool_info.lp_supply = liquidity_pool_info
        .lp_supply
        .checked_add(lp_token_to_mint)
        .ok_or(ProgramError::InvalidInstructionData)?;

    // Update provider state
    let liquidity_provider_info = LiquidityProviderInfo::get_account_info_mut(liquidity_provider_account)?;

    liquidity_provider_info.provider = *provider.key();
    liquidity_provider_info.liquidity_pool = *liquidity_pool.key();
    liquidity_provider_info.provided_token_a = liquidity_provider_info
        .provided_token_a
        .checked_add(token_a_amount)
        .ok_or(ProgramError::InvalidInstructionData)?;
    liquidity_provider_info.provided_token_b = liquidity_provider_info
        .provided_token_b
        .checked_add(token_b_amount)
        .ok_or(ProgramError::InvalidInstructionData)?;
    liquidity_provider_info.total_liquidity_provided = liquidity_provider_info
        .total_liquidity_provided
        .checked_add(token_a_amount + token_b_amount)
        .ok_or(ProgramError::InvalidInstructionData)?;
    liquidity_provider_info.total_lp_tokens = liquidity_provider_info
        .total_lp_tokens
        .checked_add(lp_token_to_mint)
        .ok_or(ProgramError::InvalidInstructionData)?;

    Ok(())
}
