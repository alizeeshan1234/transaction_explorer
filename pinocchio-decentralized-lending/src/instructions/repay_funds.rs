use pinocchio::{
    seeds, account_info::AccountInfo, instruction::Signer, program_error::ProgramError, ProgramResult, pubkey::Pubkey
};
use pinocchio_token::{
    instructions::{MintToChecked, TransferChecked},
    state::{Mint, TokenAccount},
};
use crate::states::{BorrowInfo, LiquidityPool, LiquidityProviderInfo};

pub fn repay_funds(accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {

    let [borrower, wanted_mint, giving_mint, borrower_account_info, liquidity_pool, token_vault_a, token_vault_b, fee_vault_a, fee_vault_b, borrower_ata, borrower_collateral_ata, token_program, associated_token_program, system_program] = accounts else {
        return Err(ProgramError::InvalidAccountData);
    };

    if !borrower.is_signer() && !borrower.is_writable() {
        return Err(ProgramError::InvalidAccountData);
    };

    for account in [borrower_account_info, liquidity_pool, token_vault_a, token_vault_b, fee_vault_a, fee_vault_b, borrower_ata, borrower_collateral_ata] {
        if !account.is_writable() {
            return Err(ProgramError::InvalidAccountData);
        }
    }

    let collateral_token_account = TokenAccount::from_account_info(borrower_collateral_ata)?;

    if *collateral_token_account.mint() != *giving_mint.key() {
        return Err(ProgramError::InvalidAccountData); // Wrong mint
    }

    if *collateral_token_account.owner() != *borrower.key() {
        return Err(ProgramError::IllegalOwner); 
    }

    let repay_amount = u64::from_le_bytes(
        instruction_data[0..8].try_into().map_err(|_| ProgramError::InvalidInstructionData)?
    );

    let borrower_info_account_mut = BorrowInfo::get_account_info_mut(borrower_account_info);
    let liquidity_pool_info = LiquidityPool::get_account_info_mut(liquidity_pool);

    if repay_amount > borrower_info_account_mut.total_borrowed {
        return Err(ProgramError::InvalidInstructionData);
    }

    let wanted_mint_info = Mint::from_account_info(wanted_mint)?;
        let (repay_vault, repay_mint_account, repay_decimals) = if borrower_info_account_mut.borrowed_from_pool == liquidity_pool_info.mint_a {
        let mint_info = Mint::from_account_info(wanted_mint)?;
        (token_vault_a, wanted_mint, mint_info.decimals())
    } else {
        let mint_info = Mint::from_account_info(giving_mint)?;
        (token_vault_b, giving_mint, mint_info.decimals())
    };

    TransferChecked {
        from: borrower_ata,
        to: repay_vault,
        mint: repay_mint_account, 
        authority: borrower,
        amount: repay_amount,
        decimals: repay_decimals,
    }.invoke()?;

    borrower_info_account_mut.total_borrowed = borrower_info_account_mut.total_borrowed.checked_sub(repay_amount).unwrap();
    
    // If loan fully repaid, return collateral
    if borrower_info_account_mut.total_borrowed == 0 {
       let (collateral_vault, collateral_mint, collateral_decimals) = 
        if borrower_info_account_mut.borrowed_from_pool == liquidity_pool_info.mint_a {
            let mint_info = Mint::from_account_info(giving_mint)?;
            (token_vault_b, giving_mint, mint_info.decimals())
        } else {
            let mint_info = Mint::from_account_info(wanted_mint)?;
            (token_vault_a, wanted_mint, mint_info.decimals())
        };

        let bump_ref = &[liquidity_pool_info.bump];
        let seeds = seeds!(
            b"liquidity_pool",
            liquidity_pool_info.mint_a.as_ref(),
            liquidity_pool_info.mint_b.as_ref(),
            liquidity_pool_info.authority.as_ref(),
            bump_ref
        );
        let signer_seeds = Signer::from(&seeds);

        let collateral_mint_info = Mint::from_account_info(collateral_mint)?;

        TransferChecked {
            from: collateral_vault,
            to: borrower_collateral_ata,
            authority: liquidity_pool,
            mint: collateral_mint,
            amount: borrower_info_account_mut.total_collateral,
            decimals: collateral_mint_info.decimals()
        }.invoke_signed(&[signer_seeds])?;

        borrower_info_account_mut.total_collateral = 0;
        borrower_info_account_mut.borrowed_from_pool = Pubkey::default();
    }

    Ok(())
}