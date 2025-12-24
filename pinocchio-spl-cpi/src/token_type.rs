use borsh::{BorshSerialize, BorshDeserialize};

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub enum TokenProgramType {
    PToken,
    Token2022,
}