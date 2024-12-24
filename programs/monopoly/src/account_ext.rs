use solana_program::{
    account_info::AccountInfo,
    program_pack::Pack,
};
use crate::account::GameAccount;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_into_game_account() {
        // TODO: Implement test
    }
}

pub trait AccountExt {
    fn try_into_game_account(&self) -> Result<GameAccount, &'static str>;
}

impl AccountExt for AccountInfo<'_> {
    fn try_into_game_account(&self) -> Result<GameAccount, &'static str> {
        GameAccount::unpack(&self.data.borrow())
            .map_err(|_| "Failed to deserialize game account")
    }
}
