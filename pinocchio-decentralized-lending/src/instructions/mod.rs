pub mod init_liquidity_pool;
pub use init_liquidity_pool::*;

pub mod init_liquidity_provider;
pub use init_liquidity_provider::*;

pub mod provide_liquidity;
pub use provide_liquidity::*;

pub mod borrow;
pub use borrow::*;

pub mod liquidate;
pub use liquidate::*;

pub mod repay_funds;
pub use repay_funds::*;

use pinocchio::program_error::ProgramError;

#[repr(u8)]
pub enum ProgramInstructions {
    InitializeLiquidityPool,
    InitializeLiquidityProvider,
    ProvideLiquidity,
    BorrowFunds,
    RepayFunds,
    LiquidateCollateral
}

impl TryFrom<&u8> for ProgramInstructions {
    type Error = ProgramError;

    fn try_from(value: &u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ProgramInstructions::InitializeLiquidityPool),
            1 => Ok(ProgramInstructions::InitializeLiquidityProvider),
            2 => Ok(ProgramInstructions::ProvideLiquidity),
            3 => Ok(ProgramInstructions::BorrowFunds),
            4 => Ok(ProgramInstructions::RepayFunds),
            5 => Ok(ProgramInstructions::LiquidateCollateral),
            _=> Err(ProgramError::InvalidInstructionData)
        }
    }
}