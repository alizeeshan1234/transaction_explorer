use pinocchio::{
    account_info::AccountInfo,
    instruction::{AccountMeta, Instruction},
    program,
    sysvars::rent::Rent,
    ProgramResult,
};

use crate::create_token_account::{TOKEN_PROGRAM_ID, TOKEN2022_PROGRAM_ID};

use solana_program::pubkey::Pubkey;
use crate::token_type::TokenProgramType;
pub use crate::helper::create_account_instruction_data;

pub fn create_mint_account_cpi(
    token_program_type: TokenProgramType,
    mint_account: &AccountInfo,
    mint_authority: &AccountInfo,
    payer: &AccountInfo,
    system_program: &AccountInfo,
    rent_sysvar: &AccountInfo,
    decimals: u8,
) -> ProgramResult {
    let selected_token_program_id = match token_program_type {
        TokenProgramType::PToken => Pubkey::new_from_array(TOKEN_PROGRAM_ID),
        TokenProgramType::Token2022 => Pubkey::new_from_array(TOKEN2022_PROGRAM_ID),
    };

    let space = 82u64; // Mint size: 82 bytes for both SPL and Token2022
    let rent = Rent::from_account_info(rent_sysvar)?;
    let lamports = rent.minimum_balance(space as usize);

    // STEP 1: Create mint account
    let create_account_data =
        create_account_instruction_data(lamports, space, &selected_token_program_id.to_bytes());

    let create_account_meta = [
        AccountMeta::new(payer.key(), true, true),
        AccountMeta::new(mint_account.key(), true, false),
    ];

    let create_instruction = Instruction {
        program_id: system_program.key(),
        data: &create_account_data,
        accounts: &create_account_meta,
    };

    program::invoke(
        &create_instruction,
        &[&payer.clone(), &mint_account.clone(), &system_program.clone()],
    )?;

    // STEP 2: Initialize mint account via Token Program
    let initialize_mint_meta = [
        AccountMeta::writable(mint_account.key()),
        AccountMeta::readonly(rent_sysvar.key()),
    ];

    let mut initialize_data = vec![0u8]; // 0 = InitializeMint discriminator
    initialize_data.push(decimals);
    initialize_data.extend_from_slice(mint_authority.key().as_ref());
    initialize_data.extend_from_slice(&[0u8; 32]); // Freeze authority = None (can be modified if needed)
    initialize_data.push(0); // Option<Pubkey>::None

    let initialize_instruction = Instruction {
        program_id: &selected_token_program_id.to_bytes(),
        data: &initialize_data,
        accounts: &initialize_mint_meta,
    };

    program::invoke(
        &initialize_instruction,
        &[&mint_account.clone(), &rent_sysvar.clone(), &mint_authority.clone()],
    )?;

    Ok(())
}
