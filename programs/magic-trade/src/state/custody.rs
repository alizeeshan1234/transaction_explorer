use anchor_lang::prelude::*;

use crate::{error::PlatformError, math, platform::Permissions, RATE_POWER};

#[account]
#[derive(Default, InitSpace, Debug)]
pub struct Custody {
    pub id: u8,
    pub custody_bump: u8,
    pub decimals: u8,
    pub stablecoin: bool,
    pub is_virtual: bool,
    pub padding: [u8; 5],
    pub permissions: Permissions,
    pub token_mint: Pubkey,
    pub token_account: Pubkey,
    pub oracle: Pubkey,
    pub max_price_age: u64,
    pub trade_fee: u64, // RATE_DECIMALS
    pub lp_fee: u64,    // RATE_DECIMALS
    pub margin_params: MarginParams,
    pub assets: Assets,
    #[max_len(4)]
    pub supported_markets: Vec<Pubkey>,
}

#[derive(Copy, Clone, PartialEq, AnchorSerialize, AnchorDeserialize, Default, Debug, InitSpace)]
pub struct MarginParams {
    pub trade_fee_bps: u32,     // BPS_DECIMALS
    pub trade_spread_min: u64,  // in 100th of bps => USD_DECIMALS
    pub trade_spread_max: u64,  // in 100th of bps => USD_DECIMALS
    pub min_init_leverage: u32, // BPS_DECIMALS
    pub max_init_leverage: u32, // BPS_DECIMALS
    pub max_leverage: u32,      // BPS_DECIMALS
    pub max_utilization: u32,   // BPS_DECIMALS
    pub min_collateral_usd: u32,
    pub padding: [u8; 4],
    pub virtual_delay: i64,
    pub max_position_size_usd: u64,
    pub max_exposure_usd: u64,
}

#[derive(Copy, Clone, PartialEq, AnchorSerialize, AnchorDeserialize, Default, Debug, InitSpace)]
pub struct Assets {
    pub owned: u64,
    pub locked: u64,
    pub reserved: u64,
}

impl Custody {
    pub fn available_to_lock(&self) -> u64 {
        self.assets.owned.saturating_sub(self.assets.locked)
    }
    pub fn reserved_to_owned(&mut self, amount: u64) -> Result<()> {
        if amount > self.assets.reserved {
            return Err(ProgramError::InsufficientFunds.into());
        }
        self.assets.reserved = self.assets.reserved.saturating_sub(amount);
        self.assets.owned = self.assets.owned.saturating_add(amount);
        Ok(())
    }
    pub fn owned_to_reserved(&mut self, amount: u64) -> Result<()> {
        if amount > self.assets.owned {
            return Err(ProgramError::InsufficientFunds.into());
        }
        self.assets.owned = self.assets.owned.saturating_sub(amount);
        self.assets.reserved = self.assets.reserved.saturating_add(amount);
        Ok(())
    }
    pub fn lock_funds(&mut self, amount: u64) -> Result<()> {
        require!(!self.is_virtual, PlatformError::InvalidCollateralCustody);

        self.assets.locked = math::checked_add(self.assets.locked, amount)?;

        // check for max utilization
        if self.margin_params.max_utilization > 0
            && (self.margin_params.max_utilization as u128) < RATE_POWER
            && self.assets.owned > 0
        {
            let current_utilization = math::checked_as_u64(math::checked_div(
                math::checked_mul(self.assets.locked as u128, RATE_POWER)?,
                self.assets.owned as u128,
            )?)?;
            require!(
                current_utilization <= self.margin_params.max_utilization.into(),
                PlatformError::MaxUtilization
            );
        }

        if self.assets.owned < self.assets.locked {
            Err(ProgramError::InsufficientFunds.into())
        } else {
            Ok(())
        }
    }

    pub fn unlock_funds(&mut self, amount: u64) -> Result<()> {
        require!(!self.is_virtual, PlatformError::InvalidCollateralCustody);

        if amount > self.assets.locked {
            self.assets.locked = 0;
        } else {
            self.assets.locked = math::checked_sub(self.assets.locked, amount)?;
        }

        Ok(())
    }
}
