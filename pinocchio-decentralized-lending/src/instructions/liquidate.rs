use pinocchio::{
    account_info::AccountInfo, instruction::Signer, program_error::ProgramError, pubkey::Pubkey, seeds, sysvars::{clock::Clock, Sysvar}, ProgramResult
};
use pinocchio_token::{
    instructions::{MintToChecked, TransferChecked},
    state::{Mint, TokenAccount},
};
use crate::states::{liquidity_pool, BorrowDuration, BorrowInfo, LiquidityPool, LiquidityProviderInfo};

pub fn liquidate_collateral(accounts: &[AccountInfo]) -> ProgramResult {

    let [liquidator, borrower, loan_mint, collateral_mint, borrower_account_info, liquidity_pool, token_vault_a, token_vault_b, fee_vault_a, fee_vault_b, borrower_collateral_ata, liquidator_collateral_ata, token_program, associated_token_program, system_program] = accounts else {
        return Err(ProgramError::InvalidAccountData);
    };

    let borrower_collateral = TokenAccount::from_account_info(borrower_collateral_ata)?;

    if *borrower_collateral.owner() != *borrower.key() {
        return Err(ProgramError::IllegalOwner);
    };

    if *borrower_collateral.mint() != *collateral_mint.key() {
        return Err(ProgramError::InvalidAccountData);
    };

    let liquidator_collateral = TokenAccount::from_account_info(liquidator_collateral_ata)?;

    if *liquidator_collateral.owner() != *liquidator.key() {
        return Err(ProgramError::IllegalOwner);
    };

    if *liquidator_collateral.mint() != *collateral_mint.key() {
        return Err(ProgramError::InvalidAccountData);
    };

    let clock = Clock::get()?.unix_timestamp;

    let borrower_info_account_mut = BorrowInfo::get_account_info_mut(borrower_account_info);

    let borrow_duration = borrower_info_account_mut.borrow_duration;

    let borrow_duration_u8 = match borrow_duration {
        BorrowDuration::TenDays => 10,
        BorrowDuration::TwentyDays => 20,
        BorrowDuration::ThirtyDays => 30,
        _ => 10
    };

    let expiry_time = borrower_info_account_mut.borrowed_at + borrow_duration_u8;

    if clock <= expiry_time {
        return Err(ProgramError::InvalidAccountData);
    };

    if borrower_info_account_mut.borrower != *borrower.key() {
        return Err(ProgramError::InvalidAccountData);
    };

    let amount = borrower_info_account_mut.total_collateral;

    let mint_info = Mint::from_account_info(collateral_mint)?;

    let liquidity_pool_info = LiquidityPool::get_account_info_mut(liquidity_pool);

    let bump_ref = &[liquidity_pool_info.bump];
    let seeds = seeds!(
        b"liquidity_pool",
        liquidity_pool_info.mint_a.as_ref(),
        liquidity_pool_info.mint_b.as_ref(),
        liquidity_pool_info.authority.as_ref(),
        bump_ref
    );
    let signer_seeds = Signer::from(&seeds);

    TransferChecked {
        from: borrower_collateral_ata,
        to: liquidator_collateral_ata,
        authority: liquidity_pool,
        mint: collateral_mint,
        amount: amount,
        decimals: mint_info.decimals(),
    }.invoke_signed(&[signer_seeds])?;

    borrower_info_account_mut.total_borrowed = 0;
    borrower_info_account_mut.total_collateral = 0;
    borrower_info_account_mut.borrowed_from_pool = Pubkey::default();
    borrower_info_account_mut.is_closed = true;

    Ok(())
}