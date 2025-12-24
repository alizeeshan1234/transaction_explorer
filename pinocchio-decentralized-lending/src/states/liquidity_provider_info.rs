use pinocchio::{account_info::AccountInfo, pubkey::Pubkey, program_error::ProgramError, *};

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LiquidityProviderInfo {
    pub provider: Pubkey,
    pub liquidity_pool: Pubkey,
    pub provided_token_a: u64,
    pub provided_token_b: u64,
    pub total_liquidity_provided: u64,
    pub total_lp_tokens: u64,
}

impl LiquidityProviderInfo {
    pub const LEN: usize = core::mem::size_of::<LiquidityProviderInfo>();

    pub fn get_account_info(accounts: &AccountInfo) -> Result<&Self, ProgramError> {
        if accounts.data_len() < Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        };

        return Ok(unsafe {
            &*(accounts.borrow_data_unchecked().as_ptr() as *const &Self)
        });
    }

    pub fn get_account_info_mut(accounts: &AccountInfo) -> Result<&mut Self, ProgramError> {
        if accounts.data_len() < Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        };

        return Ok(unsafe {
            &mut *(accounts.borrow_mut_data_unchecked().as_mut_ptr() as *mut &mut Self)
        });
    }
}