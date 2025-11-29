use crate::{
    constants::USD_DECIMALS, error::PlatformError, initialize, math, platform::Permissions,
    pool::Pool, BPS_POWER,
};
use anchor_lang::prelude::*;
use std::{cmp::Ordering, fmt, u128};

const ORACLE_EXPONENT_SCALE: i32 = -9;
const ORACLE_PRICE_SCALE: u64 = 1_000_000_00;
const ORACLE_MAX_PRICE: u64 = (1 << 28) - 1; // 268435455

#[account]
#[derive(Default, InitSpace, Debug)]
pub struct Market {
    pub id: u8,
    pub market_bump: u8,
    pub side: Side,
    pub is_virtual: bool,
    pub padding: [u8; 3],
    pub permissions: Permissions,
    pub target_custody: Pubkey,
    pub lock_custody: Pubkey,
    pub open_positions: u64,
    pub collective_position: Position,
}

#[derive(Copy, Clone, PartialEq, AnchorSerialize, AnchorDeserialize, Debug, InitSpace)]
pub enum Side {
    None,
    Long,
    Short,
}

impl Default for Side {
    fn default() -> Self {
        Side::None
    }
}

impl Market {
    pub fn add_position(&mut self, position: &Position) -> Result<()> {
        // Increase poisiton count
        self.open_positions = math::checked_add(self.open_positions, 1)?;
        self.collective_position.size_amount =
            math::checked_add(self.collective_position.size_amount, position.size_amount)?;
        self.collective_position.size_usd =
            math::checked_add(self.collective_position.size_usd, position.size_usd)?;
        self.collective_position.collateral_usd = math::checked_add(
            self.collective_position.collateral_usd,
            position.collateral_usd,
        )?;
        self.collective_position.locked_amount = math::checked_add(
            self.collective_position.locked_amount,
            position.locked_amount,
        )?;
        // average_entry_price = updated_size_usd / updated_size (scaled to get the final vlaue in Oracle Exponent)
        if self.open_positions > 1 {
            require!(
                self.collective_position.entry_price.exponent == position.entry_price.exponent,
                PlatformError::ExponentMismatch
            );
            let size_usd_scaled = math::scale_u128_to_exponent(
                self.collective_position.size_usd as u128,
                -(USD_DECIMALS as i32),
                -(self.collective_position.size_decimals as i32)
                    + self.collective_position.entry_price.exponent,
            )?;

            self.collective_position.entry_price.price = math::checked_as_u64(math::checked_div(
                size_usd_scaled,
                self.collective_position.size_amount as u128,
            )?)?;
        } else {
            self.collective_position.entry_price = position.entry_price;
            self.collective_position.size_decimals = position.size_decimals;
            self.collective_position.locked_decimals = position.locked_decimals;
        }
        Ok(())
    }

    pub fn remove_position(&mut self, position: &Position) -> Result<()> {
        if self.open_positions == 1 {
            // All counterparties have closed their positions
            let size_decimals = self.collective_position.size_decimals;
            let locked_decimals = self.collective_position.locked_decimals;
            self.collective_position = Position::default();
            self.collective_position.size_decimals = size_decimals;
            self.collective_position.locked_decimals = locked_decimals;
            return Ok(());
        } else {
            self.open_positions = math::checked_sub(self.open_positions, 1)?;
        }
        self.collective_position.collateral_usd = math::checked_sub(
            self.collective_position.collateral_usd,
            position.collateral_usd,
        )?;
        self.collective_position.locked_amount = math::checked_sub(
            self.collective_position.locked_amount,
            position.locked_amount,
        )?;
        self.collective_position.size_amount =
            math::checked_sub(self.collective_position.size_amount, position.size_amount)?;
        self.collective_position.size_usd =
            math::checked_sub(self.collective_position.size_usd, position.size_usd)?;
        require!(
            self.collective_position.entry_price.exponent == position.entry_price.exponent,
            PlatformError::ExponentMismatch
        );
        // average_entry_price = updated_size_usd / updated_size (scaled to get the final vlaue in Oracle Exponent)
        let size_usd_scaled = math::scale_u128_to_exponent(
            self.collective_position.size_usd as u128,
            -(USD_DECIMALS as i32),
            -(self.collective_position.size_decimals as i32)
                + self.collective_position.entry_price.exponent,
        )?;
        self.collective_position.entry_price.price = math::checked_as_u64(math::checked_div(
            size_usd_scaled,
            self.collective_position.size_amount as u128,
        )?)?;
        Ok(())
    }
}

#[derive(
    Copy, Clone, Eq, PartialEq, AnchorSerialize, AnchorDeserialize, Default, Debug, InitSpace,
)]
pub struct OraclePrice {
    pub price: u64,
    pub exponent: i32,
}

impl fmt::Display for OraclePrice {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "OraclePrice: {:?}", self)
    }
}

impl PartialOrd for OraclePrice {
    fn partial_cmp(&self, other: &OraclePrice) -> Option<Ordering> {
        let (lhs, rhs) = if self.exponent == other.exponent {
            (self.price, other.price)
        } else if self.exponent < other.exponent {
            if let Ok(scaled_price) = other.scale_to_exponent(self.exponent) {
                (self.price, scaled_price.price)
            } else {
                return None;
            }
        } else if let Ok(scaled_price) = self.scale_to_exponent(other.exponent) {
            (scaled_price.price, other.price)
        } else {
            return None;
        };
        lhs.partial_cmp(&rhs)
    }
}

impl OraclePrice {
    pub const NIL_PRICE: OraclePrice = OraclePrice {
        price: 0,
        exponent: 0,
    };

    pub fn new(price: u64, exponent: i32) -> Self {
        Self { price, exponent }
    }

    pub fn from_pyth(
        oracle_ai: &AccountInfo,
        curtime: i64,
        staleness_threshold: i64,
    ) -> Result<OraclePrice> {
        let price_data =
            pyth_solana_receiver_sdk::price_update::PriceUpdateV2::try_deserialize_unchecked(
                &mut oracle_ai.data.borrow().as_ref(),
            )
            .map_err(Into::<Error>::into)?
            .price_message;

        require!(
            curtime.saturating_sub(price_data.publish_time) <= staleness_threshold,
            PlatformError::StaleOraclePrice
        );

        Ok(OraclePrice::new(
            price_data.price.try_into().unwrap(),
            price_data.exponent,
        ))
    }

    // Converts token amount to USD with implied USD_DECIMALS decimals using oracle price
    pub fn get_asset_amount_usd(&self, token_amount: u64, token_decimals: u8) -> Result<u64> {
        if token_amount == 0 || self.price == 0 {
            return Ok(0);
        }
        math::checked_decimal_mul(
            token_amount,
            -(token_decimals as i32),
            self.price,
            self.exponent,
            -(USD_DECIMALS as i32),
        )
    }

    // Converts USD amount with implied USD_DECIMALS decimals to token amount
    pub fn get_token_amount(&self, asset_amount_usd: u64, token_decimals: u8) -> Result<u64> {
        if asset_amount_usd == 0 || self.price == 0 {
            return Ok(0);
        }
        math::checked_decimal_div(
            asset_amount_usd,
            -(USD_DECIMALS as i32),
            self.price,
            self.exponent,
            -(token_decimals as i32),
        )
    }

    /// Returns price with mantissa normalized to be less than ORACLE_MAX_PRICE
    pub fn normalize(&self) -> Result<OraclePrice> {
        let mut p = self.price;
        let mut e = self.exponent;

        while p > ORACLE_MAX_PRICE {
            p = math::checked_div(p, 10)?;
            e = math::checked_add(e, 1)?;
        }

        Ok(OraclePrice {
            price: p,
            exponent: e,
        })
    }

    pub fn checked_add(&self, other: &OraclePrice) -> Result<OraclePrice> {
        require!(
            self.exponent == other.exponent,
            PlatformError::InvalidOracleAccount
        );
        Ok(OraclePrice::new(
            math::checked_add(self.price, other.price)?,
            self.exponent,
        ))
    }

    pub fn checked_sub(&self, other: &OraclePrice) -> Result<OraclePrice> {
        require!(
            self.exponent == other.exponent,
            PlatformError::InvalidOracleAccount
        );
        Ok(OraclePrice::new(
            math::checked_sub(self.price, other.price)?,
            self.exponent,
        ))
    }

    pub fn checked_div(&self, other: &OraclePrice) -> Result<OraclePrice> {
        let base = self.normalize()?;
        let other = other.normalize()?;

        Ok(OraclePrice {
            price: math::checked_div(
                math::checked_mul(base.price, ORACLE_PRICE_SCALE)?,
                other.price,
            )?,
            exponent: math::checked_sub(
                math::checked_add(base.exponent, ORACLE_EXPONENT_SCALE)?,
                other.exponent,
            )?,
        })
    }

    pub fn checked_mul(&self, other: &OraclePrice) -> Result<OraclePrice> {
        Ok(OraclePrice {
            price: math::checked_mul(self.price, other.price)?,
            exponent: math::checked_add(self.exponent, other.exponent)?,
        })
    }

    pub fn scale_to_exponent(&self, target_exponent: i32) -> Result<OraclePrice> {
        if target_exponent == self.exponent {
            return Ok(*self);
        }
        let delta = math::checked_sub(target_exponent, self.exponent)?;
        if delta > 0 {
            Ok(OraclePrice {
                price: math::checked_div(self.price, math::checked_pow(10, delta as usize)?)?,
                exponent: target_exponent,
            })
        } else {
            Ok(OraclePrice {
                price: math::checked_mul(self.price, math::checked_pow(10, (-delta) as usize)?)?,
                exponent: target_exponent,
            })
        }
    }

    pub fn get_min_price(&self, other: &OraclePrice, is_stable: bool) -> Result<OraclePrice> {
        let min_price = if self < other { self } else { other };
        if is_stable {
            if min_price.exponent > 0 {
                if min_price.price == 0 {
                    return Ok(*min_price);
                } else {
                    return Ok(OraclePrice {
                        price: 1000000u64,
                        exponent: -6,
                    });
                }
            }
            let one_usd = math::checked_pow(10u64, (-min_price.exponent) as usize)?;
            if min_price.price > one_usd {
                Ok(OraclePrice {
                    price: one_usd,
                    exponent: min_price.exponent,
                })
            } else {
                Ok(*min_price)
            }
        } else {
            Ok(*min_price)
        }
    }
}

#[derive(Copy, Clone, PartialEq, AnchorSerialize, AnchorDeserialize, Default, Debug, InitSpace)]
pub struct Position {
    pub open_time: i64,
    pub entry_price: OraclePrice,
    pub size_amount: u64,
    pub size_usd: u64,
    pub locked_amount: u64,
    pub collateral_usd: u64,
    pub size_decimals: u8,
    pub locked_decimals: u8,
    pub padding: [u8; 6],
}

impl Position {
    pub fn is_open(&self) -> bool {
        self.size_amount > 0
    }

    pub fn get_pnl_usd(
        &self,
        side: Side,
        exit_price: &OraclePrice,
        curtime: i64,
        delay: i64,
    ) -> Result<(u64, u64)> {
        if self.size_usd == 0 || self.entry_price.price == 0 {
            return Ok((0, 0));
        }

        let (price_diff_profit, price_diff_loss) = if side == Side::Long {
            if exit_price > &self.entry_price {
                if curtime > math::checked_add(self.open_time, delay)? {
                    (
                        exit_price.checked_sub(&self.entry_price)?,
                        OraclePrice::NIL_PRICE,
                    )
                } else {
                    (OraclePrice::NIL_PRICE, OraclePrice::NIL_PRICE)
                }
            } else {
                (
                    OraclePrice::NIL_PRICE,
                    self.entry_price.checked_sub(exit_price)?,
                )
            }
        } else if exit_price < &self.entry_price {
            if curtime > math::checked_add(self.open_time, delay)? {
                (
                    self.entry_price.checked_sub(exit_price)?,
                    OraclePrice::NIL_PRICE,
                )
            } else {
                (OraclePrice::NIL_PRICE, OraclePrice::NIL_PRICE)
            }
        } else {
            (
                OraclePrice::NIL_PRICE,
                exit_price.checked_sub(&self.entry_price)?,
            )
        };

        // Is the position nominally profitable?
        if price_diff_profit.price > 0 {
            Ok((
                price_diff_profit.get_asset_amount_usd(self.size_amount, self.size_decimals)?,
                0,
            ))
        } else {
            Ok((
                0,
                price_diff_loss.get_asset_amount_usd(self.size_amount, self.size_decimals)?,
            ))
        }
    }

    pub fn get_leverage_and_margin(
        &self,
        side: Side,
        target_price: &OraclePrice,
        curtime: i64,
        delay: i64,
        initial: bool,
        unsetteled_fees: u64,
    ) -> Result<(u128, u64)> {
        if self.collateral_usd == 0 {
            return Ok((0, 0));
        }
        let (profit_usd, loss_usd) = self.get_pnl_usd(side, target_price, curtime, delay)?;
        let margin_usd = if initial {
            self.collateral_usd
        } else {
            self.collateral_usd.saturating_add(profit_usd)
        };
        let margin_usd = margin_usd.saturating_sub(math::checked_add(loss_usd, unsetteled_fees)?);
        let leverage = math::checked_div(
            math::checked_mul(self.size_usd as u128, BPS_POWER)?,
            margin_usd as u128,
        )
        .map_err(|_| u128::MAX)
        .unwrap();
        Ok((leverage, margin_usd))
    }
}
