use pinocchio::{account_info::AccountInfo, instruction::Instruction, program_error::ProgramError, pubkey::Pubkey, sysvars::instructions::Instructions, *};
use pinocchio_pubkey::*;
use pinocchio_system::*;

use crate::instructions::{borrow_funds, init_liquidity_pool, initialize_liquidity_pool, initialize_liquidity_provider, liquidate_collateral, provide_liquidity, repay_funds, ProgramInstructions};

entrypoint!(process_instruction);

declare_id!("69sgoFywR7WoEKXy384CwuiwEC9bRs2nc2dx5UzZBCt7");

pub mod states;
pub mod instructions;

pub fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8]
) -> ProgramResult {

    let (ix_disc, instruction_data) = instruction_data.split_first().ok_or(ProgramError::InvalidInstructionData)?;

    match ProgramInstructions::try_from(ix_disc)? {
        ProgramInstructions::InitializeLiquidityPool => {
            initialize_liquidity_pool(accounts, instruction_data)?;
        },

        ProgramInstructions::InitializeLiquidityProvider => {
            initialize_liquidity_provider(accounts)?
        },

        ProgramInstructions::ProvideLiquidity => {
            provide_liquidity(accounts, instruction_data)?;
        },

        ProgramInstructions::BorrowFunds => {
            borrow_funds(accounts, instruction_data)?;
        },

        ProgramInstructions::RepayFunds => {
            repay_funds(accounts, instruction_data)?
        },

        ProgramInstructions::LiquidateCollateral => {
            liquidate_collateral(accounts)?
        },
    }

    Ok(())
}