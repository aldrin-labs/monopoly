import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Monopoly } from "../target/types/monopoly";
import { PublicKey, SystemProgram } from "@solana/web3.js";
import { assert } from "chai";

describe("monopoly", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Monopoly as Program<Monopoly>;

  async function createGameAndPlayers(numPlayers: number) {
    // Initialize game
    const [gamePda] = PublicKey.findProgramAddressSync(
      [Buffer.from("game")],
      program.programId
    );

    const [player1Pda] = PublicKey.findProgramAddressSync(
      [Buffer.from("player"), provider.wallet.publicKey.toBuffer()],
      program.programId
    );

    await program.methods
      .initialize("Player 1")
      .accounts({
        game: gamePda,
        player: player1Pda,
        creator: provider.wallet.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    // Add additional players
    const players = [
      { pda: player1Pda, wallet: provider.wallet.publicKey },
    ];

    for (let i = 1; i < numPlayers; i++) {
      const playerWallet = anchor.web3.Keypair.generate();
      
      // Fund the player wallet
      const signature = await provider.connection.requestAirdrop(
        playerWallet.publicKey,
        2 * anchor.web3.LAMPORTS_PER_SOL
      );
      await provider.connection.confirmTransaction(signature);

      const [playerPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("player"), playerWallet.publicKey.toBuffer()],
        program.programId
      );

      await program.methods
        .joinGame(`Player ${i + 1}`)
        .accounts({
          game: gamePda,
          player: playerPda,
          playerOwner: playerWallet.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([playerWallet])
        .rpc();

      players.push({ pda: playerPda, wallet: playerWallet.publicKey });
    }

    return { gamePda, players };
  }

  it("Plays a full game session", async () => {
    const { gamePda, players } = await createGameAndPlayers(3);
    let currentPlayerIndex = 0;
    let turnCount = 0;
    const maxTurns = 50; // Prevent infinite loops

    while (turnCount < maxTurns) {
      const currentPlayer = players[currentPlayerIndex];

      // Roll dice and move
      await program.methods
        .rollDice()
        .accounts({
          game: gamePda,
          player: currentPlayer.pda,
          playerOwner: currentPlayer.wallet,
        })
        .rpc();

      // Process tile (handles rent, special tiles, etc.)
      await program.methods
        .processTile()
        .accounts({
          game: gamePda,
          player: currentPlayer.pda,
          propertyOwner: null, // Optional, would need proper owner for rent
          playerOwner: currentPlayer.wallet,
        })
        .rpc();

      // Try to buy property (will fail if not available or insufficient funds)
      try {
        await program.methods
          .buyProperty()
          .accounts({
            game: gamePda,
            player: currentPlayer.pda,
            playerOwner: currentPlayer.wallet,
          })
          .rpc();
      } catch (e) {
        // Property might not be available or player might not have enough funds
      }

      // Check for winner
      await program.methods
        .checkWinner()
        .accounts({
          game: gamePda,
          player: currentPlayer.pda,
          playerOwner: currentPlayer.wallet,
        })
        .rpc();

      // Move to next player
      await program.methods
        .nextTurn()
        .accounts({
          game: gamePda,
          player: currentPlayer.pda,
          playerOwner: currentPlayer.wallet,
        })
        .rpc();

      currentPlayerIndex = (currentPlayerIndex + 1) % players.length;
      turnCount++;

      // Fetch game state to check if we have a winner
      const gameState = await program.account.gameAccount.fetch(gamePda);
      const playerState = await program.account.playerAccount.fetch(currentPlayer.pda);

      // Log game progress
      console.log(`Turn ${turnCount}: Player ${currentPlayerIndex + 1}`);
      console.log(`Position: ${playerState.pos}`);
      console.log(`Cash: ${playerState.cash}`);
      console.log(`Properties: ${playerState.properties.length}`);
    }

    // Verify game state
    const finalGameState = await program.account.gameAccount.fetch(gamePda);
    assert(finalGameState.initialized, "Game should still be initialized");
    assert(finalGameState.players.length === 3, "Should have 3 players");
  });
});
