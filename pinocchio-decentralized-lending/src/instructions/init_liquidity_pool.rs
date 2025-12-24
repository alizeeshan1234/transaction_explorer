use pinocchio::{account_info::AccountInfo, program_error::ProgramError, pubkey::{self, Pubkey}, sysvars::{clock::Clock, rent::Rent, Sysvar}, ProgramResult};
use pinocchio_system::instructions::CreateAccount;

use crate::states::{liquidity_pool, LiquidityPool};

use pinocchio_token::{instructions::{InitializeAccount3, InitializeMint}, state::Mint};

pub fn initialize_liquidity_pool(accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {

    let [creator, token_mint_a, token_mint_b, liquidity_pool, lp_token_mint, token_vault_a, token_vault_b, fee_vault_a, fee_vault_b, system_program, token_program, rent_sysvar] = accounts else {
        return Err(ProgramError::InvalidAccountData);
    };

    if !creator.is_signer() {
        return Err(ProgramError::InvalidAccountData);
    };

    if instruction_data.len() < 4 {
        return Err(ProgramError::InvalidInstructionData);
    }

    let (liquidity_pool_pda, liquidity_pool_bump) = pubkey::find_program_address(
        &[b"liquidity_pool", token_mint_a.key().as_ref(), token_mint_b.key().as_ref(), creator.key().as_ref()],
        &crate::ID
    );

    let (lp_token_mint_pda, lp_token_mint_bump) = pubkey::find_program_address(
        &[b"lp_token_mint", liquidity_pool.key().as_ref()],
        &crate::ID
    );

    let (token_vault_a_pda, token_vault_a_bump) = pubkey::find_program_address(
        &[b"token_vault_a", token_mint_a.key().as_ref(), liquidity_pool.key().as_ref()],
        &crate::ID
    );

    let (token_vault_b_pda, token_vault_b_bump) = pubkey::find_program_address(
        &[b"token_vault_b", token_mint_b.key().as_ref(), liquidity_pool.key().as_ref()],
        &crate::ID
    );

    let (fee_vault_a_pda, fee_vault_a_bump) = pubkey::find_program_address(
        &[b"fee_vault_a", token_mint_a.key().as_ref(), liquidity_pool.key().as_ref()],
        &crate::ID
    );

    let (fee_vault_b_pda, fee_vault_b_bump) = pubkey::find_program_address(
        &[b"fee_vault_b", token_mint_b.key().as_ref(), liquidity_pool.key().as_ref()],
        &crate::ID
    );

    let ltv_ratio = instruction_data[0];
    let liquidation_threshold = instruction_data[1];
    let liquidation_penalty = instruction_data[2];
    let interest_rate = instruction_data[3];

    if token_mint_a.key() == token_mint_b.key() {
        return Err(ProgramError::InvalidArgument);
    }

    if *liquidity_pool.key() != liquidity_pool_pda {
        return Err(ProgramError::InvalidAccountData);
    };

    if *lp_token_mint.key() != lp_token_mint_pda {
        return Err(ProgramError::InvalidAccountData);
    }

    if *lp_token_mint.key() != lp_token_mint_pda {
        return Err(ProgramError::InvalidAccountData);
    };

    if *token_vault_a.key() != token_vault_a_pda {
        return Err(ProgramError::InvalidAccountData);
    };

    if *token_vault_b.key() != token_vault_b_pda {
        return Err(ProgramError::InvalidAccountData);
    };

    if *fee_vault_a.key() != fee_vault_a_pda {
        return Err(ProgramError::InvalidAccountData);
    };

    if *fee_vault_b.key() != fee_vault_b_pda {
        return Err(ProgramError::InvalidAccountData);
    };

    let rent = Rent::get()?;
    let clock = Clock::get()?;

    if liquidity_pool.owner() != &crate::ID {

        let lamports = Rent::get()?.minimum_balance(LiquidityPool::LEN);

        CreateAccount {
            from: creator,
            to: liquidity_pool,
            lamports,
            space: LiquidityPool::LEN as u64,
            owner: &crate::ID
        }.invoke()?;

        let liquidity_pool_account_mut = LiquidityPool::get_account_info_mut(liquidity_pool);

        liquidity_pool_account_mut.authority = *creator.key();
        liquidity_pool_account_mut.mint_a = *token_mint_a.key();
        liquidity_pool_account_mut.mint_b = *token_mint_b.key();
        liquidity_pool_account_mut.lp_mint = *lp_token_mint.key();
        liquidity_pool_account_mut.vault_a = *token_vault_a.key();
        liquidity_pool_account_mut.vault_b = *token_vault_b.key();
        liquidity_pool_account_mut.fees_vault_a = *fee_vault_a.key();
        liquidity_pool_account_mut.fees_vault_b = *fee_vault_b.key();
        liquidity_pool_account_mut.total_liquidity = 0;
        liquidity_pool_account_mut.total_borrowed_a = 0;
        liquidity_pool_account_mut.total_borrowed_b = 0;
        liquidity_pool_account_mut.total_borrowed = 0;
        liquidity_pool_account_mut.ltv_ratio = ltv_ratio;
        liquidity_pool_account_mut.liquidation_threshold = liquidation_threshold;
        liquidity_pool_account_mut.liquidation_penalty = liquidation_penalty;
        liquidity_pool_account_mut.interest_rate = interest_rate;
        liquidity_pool_account_mut.created_at = clock.unix_timestamp;
        liquidity_pool_account_mut.lp_supply = 0;
        liquidity_pool_account_mut.bump = liquidity_pool_bump;
        liquidity_pool_account_mut.vault_a_bump = token_vault_a_bump;
        liquidity_pool_account_mut.vault_b_bump = token_vault_b_bump;
        liquidity_pool_account_mut.fees_vault_a_bump = fee_vault_a_bump;
        liquidity_pool_account_mut.fees_vault_b_bump = fee_vault_b_bump;

    } else {
        return Err(ProgramError::InvalidAccountData);
    };

    if lp_token_mint.owner() != token_program.key() {
        let lamports = rent.minimum_balance(Mint::LEN);

        CreateAccount {
            from: creator,
            to: lp_token_mint,
            lamports,
            space: Mint::LEN as u64,
            owner: token_program.key()
        }.invoke()?;

        InitializeMint {
            mint: lp_token_mint,
            decimals: 6,
            mint_authority: &liquidity_pool_pda,
            freeze_authority: None,
            rent_sysvar: rent_sysvar,
        }.invoke()?;
    };

    if token_vault_a.owner() != token_program.key() {
        let lamports = rent.minimum_balance(pinocchio_token::state::TokenAccount::LEN);

        CreateAccount {
            from: creator,
            to: token_vault_a,
            lamports,
            space: pinocchio_token::state::TokenAccount::LEN as u64,
            owner: token_program.key()
        }.invoke()?;

        InitializeAccount3 {
            account: token_vault_a,
            mint: token_mint_a,
            owner: &liquidity_pool_pda
        }.invoke()?;
    };

    if token_vault_b.owner() != token_program.key() {
        let lamports = rent.minimum_balance(pinocchio_token::state::TokenAccount::LEN);

        CreateAccount {
            from: creator,
            to: token_vault_b,
            lamports,
            space: pinocchio_token::state::TokenAccount::LEN as u64,
            owner: token_program.key()
        }.invoke()?;

        InitializeAccount3 {
            account: token_vault_b,
            mint: token_mint_b,
            owner: &liquidity_pool_pda
        }.invoke()?;
    }

    if fee_vault_a.owner() != token_program.key() {
        let lamports = rent.minimum_balance(pinocchio_token::state::TokenAccount::LEN);

        CreateAccount {
            from: creator,
            to: fee_vault_a,
            lamports,
            space: pinocchio_token::state::TokenAccount::LEN as u64,
            owner: token_program.key(),
        }.invoke()?;

        InitializeAccount3 {
            account: fee_vault_a,
            mint: token_mint_a,
            owner: &liquidity_pool_pda,
        }.invoke()?;
    }

    if fee_vault_b.owner() != token_program.key() {
        let lamports = rent.minimum_balance(pinocchio_token::state::TokenAccount::LEN);

        CreateAccount {
            from: creator,
            to: fee_vault_b,
            lamports,
            space: pinocchio_token::state::TokenAccount::LEN as u64,
            owner: token_program.key(),
        }.invoke()?;

        InitializeAccount3 {
            account: fee_vault_b,
            mint: token_mint_b,
            owner: &liquidity_pool_pda,
        }.invoke()?;
    }


    Ok(())
}

