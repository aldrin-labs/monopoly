// Solana Monopoly Full Game Session Test
// This file implements a complete game session test to demonstrate full functionality

import { PublicKey, Connection } from "@solana/web3.js";
import * as anchor from "@coral-xyz/anchor";
import {
  InitializeNewWorld,
  AddEntity,
  InitializeComponent,
  ApplySystem,
} from "@magicblock-labs/bolt-sdk";
import { expect } from "chai";
import { Buffer } from "buffer";

// Player state class
class PlayerState {
  cash: number;
  position: number;
  jailTurns: number;
  properties: number[];

  constructor(fields: { cash: number; position: number; jailTurns: number; properties: number[] }) {
    this.cash = fields.cash;
    this.position = fields.position;
    this.jailTurns = fields.jailTurns;
    this.properties = fields.properties;
  }

  static deserialize(data: Buffer): PlayerState {
    // Simple deserialization assuming fixed layout:
    // u64 (8 bytes) - cash
    // u8 (1 byte) - position
    // u8 (1 byte) - jailTurns
    // remaining bytes - properties array
    const cash = data.readBigUInt64LE(0);
    const position = data.readUInt8(8);
    const jailTurns = data.readUInt8(9);
    const properties = Array.from(data.slice(10));
    
    return new PlayerState({
      cash: Number(cash),
      position,
      jailTurns,
      properties,
    });
  }
}

// Component and System IDs (as PublicKeys)
const PLAYER_COMPONENT = new PublicKey("PlayerComponent11111111111111111111111111");
const MOVEMENT_SYSTEM = new PublicKey("MovementSystem11111111111111111111111111");
const PROPERTY_SYSTEM = new PublicKey("PropertySystem11111111111111111111111111");
const CARDS_SYSTEM = new PublicKey("CardsSystem1111111111111111111111111111");

describe("Solana Monopoly - Full Game Session", () => {
  // Configure the client to use the local cluster
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  // Test constants
  const PLAYER_COUNT = 4;
  const MAX_TURNS = 100; // Maximum number of turns before declaring a draw
  const INITIAL_PLAYER_CASH = 1500; // Standard Monopoly starting cash
  const WIN_CONDITION_PROPERTY_COUNT = 20; // More than half of 40 properties

  it("Runs a complete session until a winner is found", async () => {
    // Track turn number for deterministic dice rolls
    let turnNumber = 0;

    // 1. Initialize the game world
    const initWorld = await InitializeNewWorld({
      payer: provider.wallet.publicKey,
      connection: provider.connection,
    });
    console.log("World initialized with PDA:", initWorld.worldPda.toString());
    const worldPda = initWorld.worldPda;

    // 2. Create player entities
    const playerEntities = [];
    for (let i = 0; i < PLAYER_COUNT; i++) {
      const addEntity = await AddEntity({
        payer: provider.wallet.publicKey,
        world: worldPda,
        connection: provider.connection,
      });
      playerEntities.push(addEntity.entityPda);
      
      // Initialize player component with starting cash
      await InitializeComponent({
        payer: provider.wallet.publicKey,
        entity: addEntity.entityPda,
        componentId: PLAYER_COMPONENT,
        seed: "player" + i.toString(),
      });
      console.log(`Player ${i + 1} initialized with PDA:`, addEntity.entityPda.toString());
    }

    // 3. Game loop
    let turnCount = 0;
    let winnerFound = false;
    let winnerId = -1;

    while (turnCount < MAX_TURNS && !winnerFound) {
      for (let playerId = 0; playerId < PLAYER_COUNT; playerId++) {
        // Skip if player is in jail
        const playerStateAccount = await provider.connection.getAccountInfo(playerEntities[playerId]);
        if (!playerStateAccount) {
          console.error(`Failed to fetch state for player ${playerId + 1}`);
          continue;
        }
        
        const playerState = PlayerState.deserialize(playerStateAccount.data);

        if (playerState.jailTurns > 0) {
          console.log(`Player ${playerId + 1} is in jail. Skipping turn.`);
          continue;
        }

        // Roll dice and move with test mode enabled and predetermined roll index
        await ApplySystem({
          authority: provider.wallet.publicKey,
          world: worldPda,
          systemId: MOVEMENT_SYSTEM,
          entities: [{
            entity: playerEntities[playerId],
            components: [{ componentId: PLAYER_COMPONENT }],
          }],
          args: Buffer.from([1, turn % 11]), // Test mode flag and roll index
        });
        console.log(`Player ${playerId + 1} moved`);

        // Handle property purchase/rent
        await ApplySystem({
          authority: provider.wallet.publicKey,
          world: worldPda,
          systemId: PROPERTY_SYSTEM,
          entities: [{
            entity: playerEntities[playerId],
            components: [{ componentId: PLAYER_COMPONENT }],
          }],
        });
        console.log(`Player ${playerId + 1} property action processed`);

        // Handle card draws if on special tile
        await ApplySystem({
          authority: provider.wallet.publicKey,
          world: worldPda,
          systemId: CARDS_SYSTEM,
          entities: [{
            entity: playerEntities[playerId],
            components: [{ componentId: PLAYER_COMPONENT }],
          }],
        });
        console.log(`Player ${playerId + 1} card action processed`);

        // Increment turn number after all actions
        turnNumber++;

        // Check win condition
        const updatedPlayerStateAccount = await provider.connection.getAccountInfo(playerEntities[playerId]);
        if (!updatedPlayerStateAccount) {
          console.error(`Failed to fetch updated state for player ${playerId + 1}`);
          continue;
        }

        const updatedPlayerState = PlayerState.deserialize(updatedPlayerStateAccount.data);

        if (updatedPlayerState.properties.length >= WIN_CONDITION_PROPERTY_COUNT) {
          winnerFound = true;
          winnerId = playerId;
          break;
        }
      }
      turnCount++;
      console.log(`Turn ${turnCount} completed`);
    }

    // Assert game completion
    expect(winnerFound).to.equal(true, "Expected a winner before max turns");
    expect(winnerId).to.be.greaterThan(-1, "Expected a valid winner ID");
    console.log(`Game completed in ${turnCount} turns. Player ${winnerId + 1} wins!`);
  });
});
