use num_derive::FromPrimitive;
use solana_program::{
    decode_error::DecodeError,
    msg,
    program_error::ProgramError,
};
use thiserror::Error;

#[derive(Error, Debug, Copy, Clone, FromPrimitive)]
pub enum MonopolyError {
    #[error("Invalid Instruction")]
    InvalidInstruction,
    #[error("Not Rent Owner")]
    NotRentOwner,
    #[error("Insufficient Balance")]
    InsufficientBalance,
    #[error("Invalid Move")]
    InvalidMove,
    #[error("Property Already Owned")]
    PropertyAlreadyOwned,
    #[error("Maximum Houses Reached")]
    MaximumHousesReached,
    #[error("Invalid Property")]
    InvalidProperty,
    #[error("Not Property Owner")]
    NotPropertyOwner,
    #[error("Invalid Game State")]
    InvalidGameState,
    #[error("Insufficient Funds")]
    InsufficientFunds,
    #[error("Property Not Owned")]
    PropertyNotOwned,
    #[error("Own Property")]
    OwnProperty,
    #[error("Not Player's Turn")]
    NotPlayerTurn,
    #[error("Not A Property")]
    NotAProperty,
    #[error("Invalid Dice Roll")]
    InvalidDiceRoll,
    #[error("Incomplete Color Set")]
    IncompleteColorSet,
    #[error("Invalid Property Index")]
    InvalidPropertyIndex,
    #[error("Maximum Houses Reached")]
    MaxHousesReached,
    #[error("Invalid Player Count")]
    InvalidPlayerCount,
}

impl From<MonopolyError> for ProgramError {
    fn from(e: MonopolyError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl<T> DecodeError<T> for MonopolyError {
    fn type_of() -> &'static str {
        "MonopolyError"
    }
}
