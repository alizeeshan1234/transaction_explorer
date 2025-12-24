use pinocchio::{
    account_info::AccountInfo, instruction::Signer, program_error::ProgramError, pubkey, seeds, sysvars::{clock::Clock, Sysvar}, ProgramResult
};
use pinocchio_token::{
    instructions::{MintToChecked, TransferChecked},
    state::{Mint, TokenAccount},
};

use crate::states::{LiquidityPool, BorrowInfo, BorrowDuration};

pub fn borrow_funds(accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {

    let [borrower, giving_mint, wanted_mint, liquidity_pool, borrower_account_info, borrower_ata, borrower_collateral_ata, token_vault_a, token_vault_b, system_program, token_program] = accounts else {
        return Err(ProgramError::InvalidAccountData);
    };

    if instruction_data.len() < 16 {
        return Err(ProgramError::InvalidInstructionData);
    };

    if !borrower.is_signer() {
        return Err(ProgramError::InvalidAccountData);
    };

    let collateral_amount = u64::from_le_bytes(
        instruction_data[0..8].try_into().map_err(|_| ProgramError::InvalidInstructionData)?
    );

    let borrow_duration = u64::from_le_bytes(
        instruction_data[8..16].try_into().map_err(|_| ProgramError::InvalidInstructionData)?
    );

    let (borrower_account_pda, borrower_account_bump) = pubkey::find_program_address(
        &[b"borrower_account", borrower.key().as_ref()],
        &crate::ID
    );

    if *borrower_account_info.key() != borrower_account_pda {
        return Err(ProgramError::InvalidAccountData);
    };

    let borrow_duration_enum = match  borrow_duration {
        0 => BorrowDuration::TenDays,
        1 => BorrowDuration::TwentyDays,
        2 => BorrowDuration::ThirtyDays,
        _ => BorrowDuration::TenDays
    };

    let liquidity_pool_account = LiquidityPool::get_account_info_mut(liquidity_pool);
    let borrower_account_info = BorrowInfo::get_account_info_mut(borrower_account_info);

    let (collateral_mint, borrow_mint) = if *giving_mint.key() == liquidity_pool_account.mint_a {
        (liquidity_pool_account.mint_a, liquidity_pool_account.mint_b)
    } else if *giving_mint.key() == liquidity_pool_account.mint_b {
        (liquidity_pool_account.mint_b, liquidity_pool_account.mint_a)
    } else {
        return Err(ProgramError::InvalidInstructionData);
    };

    let ltv = liquidity_pool_account.ltv_ratio;
    let borrow_amount = collateral_amount
        .checked_mul(ltv as u64)
        .ok_or(ProgramError::InvalidInstructionData)?
        .checked_div(100)
        .ok_or(ProgramError::InvalidInstructionData)?;

    let giving_token_mint_data = Mint::from_account_info(giving_mint)?;

    // Transfer collateral from borrower to vault
    TransferChecked {
        from: borrower_collateral_ata,
        to: if collateral_mint == liquidity_pool_account.mint_a {
            token_vault_a
        } else {
            token_vault_b
        },
        mint: giving_mint,
        authority: borrower,
        amount: collateral_amount,
        decimals: giving_token_mint_data.decimals()
    }.invoke()?;

    let vault_account = if borrow_mint == liquidity_pool_account.mint_a {
        token_vault_a
    } else {
        token_vault_b
    };

    let bump_ref = &[liquidity_pool_account.bump];
    let seeds = seeds!(
        b"liquidity_pool",
        liquidity_pool_account.mint_a.as_ref(),
        liquidity_pool_account.mint_b.as_ref(),
        liquidity_pool_account.authority.as_ref(),
        bump_ref
    );
    let signer_seeds = Signer::from(&seeds);

    let wanted_token_mint_data = Mint::from_account_info(wanted_mint)?;

    TransferChecked {
        from: vault_account,
        mint: wanted_mint,
        to: borrower_ata,
        authority: liquidity_pool,
        amount: borrow_amount,
        decimals: wanted_token_mint_data.decimals()
    }.invoke_signed(&[signer_seeds])?;

    borrower_account_info.borrower = *borrower.key();
    borrower_account_info.borrowed_from_pool = *liquidity_pool.key();
    borrower_account_info.total_borrowed = borrower_account_info.total_borrowed.checked_add(borrow_amount).ok_or(ProgramError::InvalidInstructionData)?;
    borrower_account_info.total_collateral = borrower_account_info.total_collateral.checked_add(collateral_amount).ok_or(ProgramError::InvalidInstructionData)?;
    borrower_account_info.borrowed_at = Clock::get()?.unix_timestamp;
    borrower_account_info.borrow_duration = borrow_duration_enum;
    borrower_account_info.repaid_amount = 0;
    borrower_account_info.is_closed = false;
    borrower_account_info.borrower_account_bump = borrower_account_bump;

    liquidity_pool_account.total_liquidity = liquidity_pool_account.total_liquidity.checked_add(borrow_amount).ok_or(ProgramError::InvalidInstructionData)?;
    
    if borrow_mint == liquidity_pool_account.mint_a {
        liquidity_pool_account.total_borrowed_a = liquidity_pool_account.total_borrowed_a
            .checked_add(borrow_amount).ok_or(ProgramError::InvalidInstructionData)?;
    } else if borrow_mint == liquidity_pool_account.mint_b {
        liquidity_pool_account.total_borrowed_b = liquidity_pool_account.total_borrowed_b
            .checked_add(borrow_amount).ok_or(ProgramError::InvalidInstructionData)?;
    };

    liquidity_pool_account.total_borrowed = liquidity_pool_account
        .total_borrowed_a  
        .checked_add(liquidity_pool_account.total_borrowed_b)
        .ok_or(ProgramError::InvalidInstructionData)?;

    println!("Borrower Account Info: {:?}", borrower_account_info);
    println!("Liquidity Pool Info: {:?}", liquidity_pool_account);

    Ok(())
}