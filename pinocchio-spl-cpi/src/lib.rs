use pinocchio::{account_info::AccountInfo, pubkey::Pubkey,program_error::ProgramError, *};
use borsh::{BorshSerialize, BorshDeserialize};
use pinocchio_pubkey;

pub mod create_token_account;
pub use create_token_account::create_token_account_cpi;

pub mod create_mint_cpi;
pub use create_mint_cpi::*;

pub mod token_type;
use crate::token_type::TokenProgramType;

pub mod transfer_tokens;
pub use transfer_tokens::*;

pub mod helper;

pinocchio_pubkey::declare_id!("FHUW81Au2k38MLkgsZ7af8FZDiDa1s8t2Hzn6bKstrpC");

entrypoint!(process_instruction);

#[derive(BorshSerialize, BorshDeserialize)]
pub enum Instruction {
    InitializeTokenAccount { token_program_type: TokenProgramType },
    InitializeMintAccount { token_program_type: TokenProgramType, decimals: u8 },
    TransferTokens { token_program_type: TokenProgramType, amount: u64 },
}


pub fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8]
) -> ProgramResult {

    let instruction = Instruction::try_from_slice(instruction_data).unwrap();

    match instruction {
        Instruction::InitializeTokenAccount {token_program_type} => {
            if accounts.len() < 7 {
                return Err(ProgramError::NotEnoughAccountKeys);
            }

            create_token_account_cpi(
                token_program_type,
                &accounts[0],
                &accounts[1],
                &accounts[2],
                &accounts[3],
                &accounts[4], 
                &accounts[5],
            )?;
        },
        Instruction::InitializeMintAccount { token_program_type, decimals } => {
            if accounts.len() < 5 {
                return Err(ProgramError::NotEnoughAccountKeys);
            };

            create_mint_account_cpi(
                token_program_type,
                &accounts[0],
                &accounts[1],
                &accounts[2],
                &accounts[3],
                &accounts[4], 
                decimals
            )?;
        },
        Instruction::TransferTokens { token_program_type, amount } => {
            if accounts.len() < 3 {
                return Err(ProgramError::NotEnoughAccountKeys);
            };

            transfer_tokens_cpi(
                token_program_type,
                &accounts[0], 
                &accounts[1], 
                &accounts[2], 
                &accounts[3], 
                amount
            )?;
        }
    }

    Ok(())
}


// #[cfg(test)]
// pub mod testing {
//     use super::*;

//     use {
//         mollusk_svm::Mollusk,
//         solana_program::{
//             pubkey::Pubkey,
//             instruction::AccountMeta,
//         },
//         spl_token::state::Account as SplTokenAccount 
//     };

//     use solana_sdk::{account::Account, program_pack::Pack};


//    #[test]
//     fn test_initialize_token_account() {
//         let program_id = Pubkey::new_unique(); 
//         let mollusk = Mollusk::new(&program_id, "target/deploy/pinocchio_spl_cpi");

//         let payer = Pubkey::new_unique();
//         let mint = Pubkey::new_unique();
//         let token_account = Pubkey::new_unique();
//         let owner = Pubkey::new_unique();

//         let instruction_type = crate::Instruction::InitializeTokenAccount;
//         let instruction_data = instruction_type.try_to_vec().unwrap();

//         // Create the instruction with all 7 AccountMetas
//         let instruction = solana_program::instruction::Instruction {
//             program_id, 
//             accounts: vec![
//                 AccountMeta::new(token_account, false),         // [0] token_account (writable, not signer)
//                 AccountMeta::new_readonly(mint, false),         // [1] mint (readonly, not signer)
//                 AccountMeta::new_readonly(owner, false),        // [2] owner (readonly, not signer)
//                 AccountMeta::new(payer, true),                  // [3] payer (writable, signer)
//                 AccountMeta::new_readonly(solana_program::system_program::ID, false), // [4] system_program (readonly)
//                 AccountMeta::new_readonly(spl_token::ID, false),  // [5] spl_token_program (readonly)
//                 AccountMeta::new_readonly(solana_program::sysvar::rent::ID, false),    // [6] rent_sysvar (readonly)
//             ],
//             data: instruction_data,
//         };

//         let payer_account = Account::new(1_000_000_000, 0, &solana_program::system_program::ID);
//         let mint_account = Account::new(1_000_000, 82, &spl_token::ID); 
//         let token_account_account = Account::new(0, 0, &solana_program::system_program::ID);
//         let owner_account = Account::new(0, 0, &solana_program::system_program::ID);
//         let system_program_account = Account::new(0, 0, &solana_program::system_program::ID);
//         let spl_token_program_account = Account::new(0, 0, &spl_token::ID); 
//         let rent_sysvar_account = Account::new(0, 0, &solana_program::sysvar::rent::ID);

//         let result: mollusk_svm::result::InstructionResult = mollusk
//             .process_and_validate_instruction(
//                 &instruction,
//                 &[
//                     (token_account, token_account_account.into()), 
//                     (mint, mint_account.into()),
//                     (owner, owner_account.into()),
//                     (payer, payer_account.into()),
//                     (solana_program::system_program::ID, system_program_account.into()),
//                     (spl_token::ID, spl_token_program_account.into()),
//                     (solana_program::sysvar::rent::ID, rent_sysvar_account.into()),
//                 ],
//                 &[
//                     mollusk_svm::result::Check::success(),
//                 ],
//             );

//         let token_account_after = result.get_account(&token_account).unwrap();

//         assert_eq!(token_account_after.owner, spl_token::ID);

//         assert_eq!(token_account_after.data.len(), SplTokenAccount::LEN); 

//         let token_account_data = SplTokenAccount::unpack(&token_account_after.data).unwrap();
//         assert_eq!(token_account_data.mint, mint);
//         assert_eq!(token_account_data.owner, owner);
//         assert_eq!(token_account_data.amount, 0);
//         assert_eq!(token_account_data.state, spl_token::state::AccountState::Initialized);

//         let payer_account_after = result.get_account(&payer).unwrap();
//         assert!(payer_account_after.lamports < 1_000_000_000);
//     }
// }