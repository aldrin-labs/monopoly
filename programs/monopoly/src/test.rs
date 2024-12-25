use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    system_instruction,
};
use solana_program_test::{processor, ProgramTest, BanksClient};
use solana_sdk::{
    signature::{Keypair, Signer},
    transaction::Transaction,
    hash::Hash,
};

use crate::{
    account::{GameAccount, PlayerAccount},
    instruction::MonopolyInstruction,
    processor::process_instruction,
    id,
};

// We'll use the main process_instruction directly since we've implemented
// proper instruction unpacking in the main processor

pub async fn setup_test() -> (BanksClient, Keypair, Hash) {
    let program_id = id();
    let mut program_test = ProgramTest::new(
        "monopoly",
        program_id,
        None,
    );

    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;
    (banks_client, payer, recent_blockhash)
}

#[tokio::test]
async fn test_full_game_session() {
    // Initialize test environment
    let (mut banks_client, payer, recent_blockhash) = setup_test().await;
    
    // Create game account
    let game_account = Keypair::new();
    let rent = banks_client.get_rent().await.unwrap();
    let game_account_rent = rent.minimum_balance(GameAccount::LEN);
    
    let mut transaction = Transaction::new_with_payer(
        &[
            system_instruction::create_account(
                &payer.pubkey(),
                &game_account.pubkey(),
                game_account_rent,
                GameAccount::LEN as u64,
                &crate::id(),
            ),
            MonopolyInstruction::InitGame {
                player_names: vec!["Alice".to_string(), "Bob".to_string()],
            }
            .to_instruction(&crate::id(), &[&game_account.pubkey()]),
        ],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer, &game_account], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    // Create player accounts
    let player1 = Keypair::new();
    let player2 = Keypair::new();
    let player_rent = rent.minimum_balance(PlayerAccount::LEN);

    // Initialize players
    for player in [&player1, &player2] {
        let transaction = Transaction::new_with_payer(
            &[system_instruction::create_account(
                &payer.pubkey(),
                &player.pubkey(),
                player_rent,
                PlayerAccount::LEN as u64,
                &crate::id(),
            )],
            Some(&payer.pubkey()),
        );
        transaction.sign(&[&payer, player], recent_blockhash);
        banks_client.process_transaction(transaction).await.unwrap();
    }

    // Simulate 10 rounds of gameplay
    for round in 0..10 {
        println!("Round {}", round + 1);

        for player in [&player1, &player2] {
            // Roll and move
            let transaction = Transaction::new_with_payer(
                &[MonopolyInstruction::MovePlayer
                    .to_instruction(&crate::id(), &[&game_account.pubkey(), &player.pubkey()])],
                Some(&payer.pubkey()),
            );
            transaction.sign(&[&payer, player], recent_blockhash);
            banks_client.process_transaction(transaction).await.unwrap();

            // Get player position and attempt to buy property if available
            let game_state = GameAccount::unpack(
                &banks_client
                    .get_account(game_account.pubkey())
                    .await
                    .unwrap()
                    .unwrap()
                    .data
            ).unwrap();

            let current_player = game_state.game.current_player as usize;
            let position = game_state.game.players[current_player].position;

            // Try to buy property if on a property tile
            if let Some(property) = game_state.game.board[position as usize].as_property() {
                if property.owner.is_none() {
                    let transaction = Transaction::new_with_payer(
                        &[MonopolyInstruction::BuyProperty { property_index: position }
                            .to_instruction(&crate::id(), &[&game_account.pubkey(), &player.pubkey()])],
                        Some(&payer.pubkey()),
                    );
                    transaction.sign(&[&payer, player], recent_blockhash);
                    let result = banks_client.process_transaction(transaction).await;
                    if result.is_ok() {
                        println!("Player bought property at position {}", position);
                    }
                }
            }

            // Try to build houses on owned properties
            for (idx, tile) in game_state.game.board.iter().enumerate() {
                if let Some(property) = tile.as_property() {
                    if property.owner == Some(player.pubkey()) {
                        let transaction = Transaction::new_with_payer(
                            &[MonopolyInstruction::BuildHouse { property_index: idx as u8 }
                                .to_instruction(&crate::id(), &[&game_account.pubkey(), &player.pubkey()])],
                            Some(&payer.pubkey()),
                        );
                        transaction.sign(&[&payer, player], recent_blockhash);
                        let result = banks_client.process_transaction(transaction).await;
                        if result.is_ok() {
                            println!("Player built house on property at position {}", idx);
                        }
                    }
                }
            }

            // End turn
            let transaction = Transaction::new_with_payer(
                &[MonopolyInstruction::NextTurn
                    .to_instruction(&crate::id(), &[&game_account.pubkey(), &player.pubkey()])],
                Some(&payer.pubkey()),
            );
            transaction.sign(&[&payer, player], recent_blockhash);
            banks_client.process_transaction(transaction).await.unwrap();
        }

        // Check for winner every 5 rounds
        if (round + 1) % 5 == 0 {
            let transaction = Transaction::new_with_payer(
                &[MonopolyInstruction::CheckWinner
                    .to_instruction(&crate::id(), &[&game_account.pubkey()])],
                Some(&payer.pubkey()),
            );
            transaction.sign(&[&payer], recent_blockhash);
            banks_client.process_transaction(transaction).await.unwrap();
        }
    }

    // Final winner check
    let transaction = Transaction::new_with_payer(
        &[MonopolyInstruction::CheckWinner
            .to_instruction(&crate::id(), &[&game_account.pubkey()])],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();
}
