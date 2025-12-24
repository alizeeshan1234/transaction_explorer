use pinocchio::{account_info::AccountInfo, pubkey::Pubkey, *};

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BorrowInfo {
    pub borrower: Pubkey,
    pub borrowed_from_pool: Pubkey,
    pub total_borrowed: u64,
    pub total_collateral: u64,
    pub borrowed_at: i64,
    pub borrow_duration: BorrowDuration,
    pub repaid_amount: u64,
    pub is_closed: bool,
    pub borrower_account_bump: u8,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BorrowDuration {
    TenDays = 10,
    TwentyDays = 20,
    ThirtyDays = 30,
}

impl BorrowInfo {
    pub const LEN: usize = core::mem::size_of::<BorrowInfo>();

    pub fn get_account_info(accounts: &AccountInfo) -> Self {
        return unsafe {
            *(accounts.borrow_data_unchecked().as_ptr() as *const Self)
        };
    }

    pub fn get_account_info_mut(accounts: &AccountInfo) -> &mut Self {
        return unsafe {
            &mut *(accounts.borrow_mut_data_unchecked().as_mut_ptr() as *mut Self)
        }
    }
}