use pinocchio::{
    ProgramResult, 
    account_info::AccountInfo,
    instruction::{AccountMeta, Instruction},
    sysvars::rent::Rent,
    program
};

use solana_program::{pubkey::Pubkey};

pub use crate::helper::create_account_instruction_data;

use crate::create_token_account::{TOKEN_PROGRAM_ID, TOKEN2022_PROGRAM_ID};
use crate::token_type::TokenProgramType;

pub fn transfer_tokens_cpi(
    token_program_type: TokenProgramType,
    from_token_account: &AccountInfo,
    to_token_account: &AccountInfo,
    authority: &AccountInfo,
    rent_sysvar: &AccountInfo,
    amount: u64,
) -> ProgramResult {
    
    match token_program_type {

        TokenProgramType::PToken => {

            let token_program_id = Pubkey::new_from_array(TOKEN_PROGRAM_ID);

            let space = 82u64; // Mint size: 82 bytes for both SPL and Token2022
            let rent = Rent::from_account_info(rent_sysvar)?;
            let lamports = rent.minimum_balance(space as usize);

            let create_account_data = create_account_instruction_data(lamports, space, &token_program_id.to_bytes());

            let mut transfer_data = vec![3u8]; // 3 = Transfer instruction discriminator
            transfer_data.extend_from_slice(&amount.to_le_bytes());

            let transfer_token_meta = [
                AccountMeta::new(from_token_account.key(), true, true),
                AccountMeta::new(to_token_account.key(), true, false),
                AccountMeta::readonly(authority.key()),
            ];

            let transfer_instruction = Instruction {
                program_id: &TOKEN_PROGRAM_ID,
                accounts: &transfer_token_meta,
                data: &create_account_data,
            };

            program::invoke(
                &transfer_instruction,
                &[&from_token_account.clone(), &to_token_account.clone(), &authority.clone()],
            )?;
        }

        TokenProgramType::Token2022 => {
            
            let token_program_id = Pubkey::new_from_array(TOKEN2022_PROGRAM_ID);

            #[cfg(feature = "spl-token-2022")]
            {
                use spl_token_2022::instruction as token_instruction;

                let transfer_instruction = token_instruction::transfer(
                    &token_program_id,
                    from_token_account.key,
                    to_token_account.key,
                    authority.key,
                    &[],
                    amount,
                )?;

                program::invoke(
                    &transfer_instruction,
                    &[from_token_account, to_token_account, authority],
                )?;
            }

            #[cfg(not(feature = "spl-token-2022"))]
            {
                let mut transfer_data = vec![3u8];
                transfer_data.extend_from_slice(&amount.to_le_bytes());

                let transfer_token_meta = [
                    AccountMeta::new(from_token_account.key(), true, true),
                    AccountMeta::new(to_token_account.key(), true, false),
                    AccountMeta::readonly(authority.key()),
                ];

                let transfer_instruction = Instruction {
                    program_id: &TOKEN2022_PROGRAM_ID,
                    accounts: &transfer_token_meta,
                    data: &transfer_data,
                };

                program::invoke(
                    &transfer_instruction,
                    &[&from_token_account.clone(), &to_token_account.clone(), &authority.clone()],
                )?;
            }
        }
    }

    Ok(())
}
