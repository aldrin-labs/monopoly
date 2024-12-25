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
        use solana_program::{
            pubkey::Pubkey,
            account_info::AccountInfo,
            program_pack::Pack,
        };
        use std::cell::RefCell;
        use crate::account::GameAccount;
        use crate::state::Game;

        // Create a valid game account
        let game_account = GameAccount {
            is_initialized: true,
            game: Game {
                board: vec![],
                players: vec![],
                current_player: 0,
                free_parking: 0,
                initialized: true,
            },
        };

        // Pack the game account into a buffer
        let mut buffer = vec![0u8; GameAccount::LEN];
        game_account.pack_into_slice(&mut buffer);

        // Create account info with valid data
        let key = Pubkey::new_unique();
        let mut lamports = 0;
        let account_data = RefCell::new(buffer);
        let owner = Pubkey::new_unique();
        let account_info = AccountInfo::new(
            &key,
            false,
            true,
            &mut lamports,
            account_data.borrow_mut().as_mut_slice(),
            &owner,
            false,
            0,
        );

        // Test successful conversion
        let result = account_info.try_into_game_account();
        assert!(result.is_ok());
        let unpacked = result.unwrap();
        assert!(unpacked.is_initialized);
        assert!(unpacked.game.initialized);

        // Test with invalid data
        let invalid_data = RefCell::new(vec![0u8; 10]); // Too small
        let invalid_account = AccountInfo::new(
            &key,
            false,
            true,
            &mut lamports,
            invalid_data.borrow_mut().as_mut_slice(),
            &owner,
            false,
            0,
        );
        
        let result = invalid_account.try_into_game_account();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Failed to deserialize game account");
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
