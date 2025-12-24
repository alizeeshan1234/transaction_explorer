use pinocchio::{
    account_info::AccountInfo, instruction::{AccountMeta, Instruction}, program, sysvars::rent::Rent, ProgramResult, program_error::ProgramError
};

use pinocchio_token::ID;

pub const TOKEN_PROGRAM_ID: [u8; 32] = [
    6, 221, 246, 225, 215, 101, 161, 147, 217, 203, 225, 70, 206, 235, 121, 172,
    28, 180, 133, 237, 95, 91, 55, 145, 58, 140, 245, 133, 126, 255, 0, 169,
];

pub const TOKEN2022_PROGRAM_ID: [u8; 32] = [
    10, 112, 131, 147, 84, 51, 84, 60, 164, 244, 75, 206, 19, 222, 193, 205,
    72, 94, 156, 56, 196, 228, 137, 19, 29, 150, 42, 168, 58, 88, 217, 19,
];

pub const SYSTEM_PROGRAM_ID: [u8; 32] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

pub use crate::helper::create_account_instruction_data;

pub use crate::token_type::TokenProgramType;

use solana_program::pubkey::Pubkey;

/*
Need to be implemented:
Adding seeds paramter.

create_token_account_cpi_with_seeds(
    token_program_type: TokenProgramType,
    token_account: &AccountInfo,
    mint: &AccountInfo,
    owner: &AccountInfo,
    payer: &AccountInfo,
    system_program: &AccountInfo,
    rent_sysvar: &AccountInfo,
    seeds: &[u8]
)
*/

pub fn create_token_account_cpi(
    token_program_type: TokenProgramType,
    token_account: &AccountInfo,
    mint: &AccountInfo,
    owner: &AccountInfo,
    payer: &AccountInfo,
    system_program: &AccountInfo,
    rent_sysvar: &AccountInfo,
) -> ProgramResult {

    let selected_token_program_id = match token_program_type {
        TokenProgramType::PToken => Pubkey::new_from_array(TOKEN_PROGRAM_ID),
        TokenProgramType::Token2022 => Pubkey::new_from_array(TOKEN2022_PROGRAM_ID),
    };

    if *system_program.key() != SYSTEM_PROGRAM_ID {
        return Err(ProgramError::InvalidAccountOwner)
    }

    let space = 165u64;
    let rent = Rent::from_account_info(rent_sysvar)?;
    let lamports = rent.minimum_balance(space as usize);

    // STEP 1: CPI to System Program to create generic account

    let create_account_data = create_account_instruction_data(lamports, space, &selected_token_program_id.to_bytes());

    let create_account_meta = [
        AccountMeta::new(payer.key(), true, true),
        AccountMeta::new(token_account.key(), true, false),
    ];

    let create_instruction = Instruction {
        program_id: &SYSTEM_PROGRAM_ID,
        data: &create_account_data,
        accounts: &create_account_meta,
    };

    program::invoke(
        &create_instruction, 
        &[
            &payer.clone(), 
            &token_account.clone(), 
            &system_program.clone()
        ]
    )?;

    // STEP 2: CPI to Token Program to initialize as token account

    let initialize_token_account_meta = [
        AccountMeta::writable(token_account.key()),
        AccountMeta::readonly(mint.key()),
        AccountMeta::readonly(owner.key()),
        AccountMeta::readonly(rent_sysvar.key()),
    ];

    let initialize_data = vec![1u8]; // InitializeAccount instruction discriminator (the instruction to be called)

    let initialize_instruction = Instruction {
        program_id: &selected_token_program_id.to_bytes(),
        data: &initialize_data, // InitializeAccount instruction discriminator
        accounts: &initialize_token_account_meta,
    };

    program::invoke(
        &initialize_instruction,
        &[&token_account, &mint, &owner, &rent_sysvar,]
    )?;

    Ok(())
}


// #[cfg(test)]
// mod testing {

//     use super::*;
//     use mollusk_svm::Mollusk;
//     use pinocchio::{
//         account_info::AccountInfo,
//         pubkey::Pubkey,
//         sysvars::rent::Rent
//     };
//     use std::collections::HashMap;

//     fn test_program_entrypoint(
//         _program_id: &Pubkey,
//         accounts: &[AccountInfo],
//         _instruction_data: &[u8],
//     ) -> ProgramResult {
//         create_token_account_cpi(
//             &accounts[0], // token_account
//             &accounts[1], // mint
//             &accounts[2], // owner
//             &accounts[3], // payer
//             &accounts[4], // system_program
//             &accounts[5], // token_program
//             &accounts[6], // rent_sysvar
//             b"test",      // seed_data
//         )
//     }

//     #[test]
//     fn test_create_token_account_works() {
//       let program_id = Pubkey::default();

//       let mollusk = Mollusk::new(&program_id, test_program_entrypoint);
//     }
// }