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
  buildings: Map<number, number>; // Property ID -> Building count (1-4 houses, 5 hotel)

  constructor(fields: { 
    cash: number; 
    position: number; 
    jailTurns: number; 
    properties: number[];
    buildings?: Map<number, number>;
  }) {
    this.cash = fields.cash;
    this.position = fields.position;
    this.jailTurns = fields.jailTurns;
    this.properties = fields.properties;
    this.buildings = fields.buildings || new Map();
  }

  static deserialize(data: Buffer): PlayerState {
    // Enhanced deserialization with building data:
    // u64 (8 bytes) - cash
    // u8 (1 byte) - position
    // u8 (1 byte) - jailTurns
    // u8 (1 byte) - property count
    // u8[] - properties array (length from property count)
    // remaining: building data as pairs of (propertyId, buildingCount)
    const cash = data.readBigUInt64LE(0);
    const position = data.readUInt8(8);
    const jailTurns = data.readUInt8(9);
    const propertyCount = data.readUInt8(10);
    const properties = Array.from(data.slice(11, 11 + propertyCount));
    
    // Parse building data
    const buildings = new Map<number, number>();
    let offset = 11 + propertyCount;
    while (offset < data.length - 1) { // Ensure we have at least 2 bytes left
      const propertyId = data.readUInt8(offset);
      const buildingCount = data.readUInt8(offset + 1);
      buildings.set(propertyId, buildingCount);
      offset += 2;
    }
    
    return new PlayerState({
      cash: Number(cash),
      position,
      jailTurns,
      properties,
      buildings,
    });
  }
}

// Type definitions
type PlayerEntity = PublicKey;

// Constants
const PROPERTY_BASE_COST = 200; // Base cost for properties
const HOUSE_COST = 200; // Standard cost for building a house
const HOTEL_COST = 200; // Additional cost for upgrading to hotel

// Property groups for monopoly checks
const PROPERTY_GROUPS: { [key: number]: number[] } = {
  1: [1, 3], // Brown
  2: [6, 8, 9], // Light Blue
  3: [11, 13, 14], // Pink
  4: [16, 18, 19], // Orange
  5: [21, 23, 24], // Red
  6: [26, 27, 29], // Yellow
  7: [31, 32, 34], // Green
  8: [37, 39], // Dark Blue
};

// System action flags
const enum SystemAction {
  InitializePlayer = 0,
  Move = 1,
  JailEscape = 2,
  PayJailFine = 3,
  BuildProperty = 4,
  MortgageProperty = 5,
  AuctionProperty = 6,
}

// Component and System IDs (as PublicKeys)
const PLAYER_COMPONENT = new PublicKey("PlayerComponent11111111111111111111111111");
const MOVEMENT_SYSTEM = new PublicKey("MovementSystem11111111111111111111111111");
const PROPERTY_SYSTEM = new PublicKey("PropertySystem11111111111111111111111111");
const CARDS_SYSTEM = new PublicKey("CardsSystem1111111111111111111111111111");

// Helper function to find wealthiest player
async function findWealthiestPlayer(
  connection: Connection,
  playerEntities: PublicKey[]
): Promise<number> {
  let maxWealth = -1;
  let wealthiestPlayer = -1;

  for (let i = 0; i < playerEntities.length; i++) {
    const account = await connection.getAccountInfo(playerEntities[i]);
    if (!account) continue;
    
    const state = PlayerState.deserialize(account.data);
    const propertyValue = state.properties.length * PROPERTY_BASE_COST;
    const buildingValue = Array.from(state.buildings.values())
      .reduce((sum, count) => sum + (count * (count === 5 ? HOTEL_COST : HOUSE_COST)), 0);
    const totalWealth = state.cash + propertyValue + buildingValue;
    
    if (totalWealth > maxWealth) {
      maxWealth = totalWealth;
      wealthiestPlayer = i;
    }
  }
  
  return wealthiestPlayer;
}

describe("Solana Monopoly - Full Game Session", () => {
  // Configure the client to use the local cluster
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  // Test constants
  const PLAYER_COUNT = 4;
  const MAX_TURNS = 100; // Maximum number of turns before declaring a draw
  const INITIAL_PLAYER_CASH = 1500; // Standard Monopoly starting cash
  const WIN_CONDITION_PROPERTY_COUNT = 20; // More than half of 40 properties
  const JAIL_POSITION = 10; // Jail is at position 10
  const HOUSE_COST = 200; // Standard cost for building a house
  const HOTEL_COST = 200; // Additional cost for upgrading to hotel
  const PROPERTY_BASE_COST = 200; // Base cost for properties

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
    const playerEntities: PlayerEntity[] = [];
    for (let i = 0; i < PLAYER_COUNT; i++) {
      const addEntity = await AddEntity({
        payer: provider.wallet.publicKey,
        world: worldPda,
        connection: provider.connection,
      });
      playerEntities.push(addEntity.entityPda as unknown as PublicKey);
      
      // Initialize player component
      await InitializeComponent({
        payer: provider.wallet.publicKey,
        entity: addEntity.entityPda,
        componentId: PLAYER_COMPONENT,
        seed: "player" + i.toString(),
      });

      // Set initial player state with starting cash
      await ApplySystem({
        authority: provider.wallet.publicKey,
        world: worldPda,
        systemId: MOVEMENT_SYSTEM,
        entities: [{
          entity: addEntity.entityPda,
          components: [{ componentId: PLAYER_COMPONENT }],
        }],
        args: Buffer.from([
          SystemAction.InitializePlayer,
          ...new Uint8Array(new Int32Array([INITIAL_PLAYER_CASH]).buffer) // Add starting cash as argument
        ]),
      });
      console.log(`Player ${i + 1} initialized with PDA:`, addEntity.entityPda.toString());
    }

    // 3. Game loop
    let turnCount = 0;
    let winnerFound = false;
    let winnerId = -1;

    while (turnCount < MAX_TURNS && !winnerFound) {
      for (let playerId = 0; playerId < PLAYER_COUNT; playerId++) {
        // Handle jail and bankruptcy
        const playerStateAccount = await provider.connection.getAccountInfo(playerEntities[playerId]);
        if (!playerStateAccount) {
          console.error(`Failed to fetch state for player ${playerId + 1}`);
          continue;
        }
        
        const playerState = PlayerState.deserialize(playerStateAccount.data);

        // Check for bankruptcy
        if (playerState.cash <= 0) {
          console.log(`Player ${playerId + 1} is bankrupt!`);
          // Find player with most properties/cash as winner
          winnerFound = true;
          winnerId = await findWealthiestPlayer(provider.connection, playerEntities);
          break;
        }

        // Handle jail turns with doubles
        if (playerState.jailTurns > 0) {
          const roll1 = Math.floor(turnNumber % 6) + 1;
          const roll2 = Math.floor((turnNumber / 6) % 6) + 1;
          
          if (roll1 === roll2) {
            console.log(`Player ${playerId + 1} rolled doubles (${roll1},${roll2}) and got out of jail!`);
            await ApplySystem({
              authority: provider.wallet.publicKey,
              world: worldPda,
              systemId: MOVEMENT_SYSTEM,
              entities: [{
                entity: playerEntities[playerId],
                components: [{ componentId: PLAYER_COMPONENT }],
              }],
              args: Buffer.from([SystemAction.JailEscape]), // Jail escape action
            });
          } else if (playerState.cash >= 50) {
            // Pay 50 to get out of jail
            console.log(`Player ${playerId + 1} pays 50 to get out of jail`);
            await ApplySystem({
              authority: provider.wallet.publicKey,
              world: worldPda,
              systemId: MOVEMENT_SYSTEM,
              entities: [{
                entity: playerEntities[playerId],
                components: [{ componentId: PLAYER_COMPONENT }],
              }],
              args: Buffer.from([SystemAction.PayJailFine]), // Pay jail fine action
            });
          } else {
            console.log(`Player ${playerId + 1} stays in jail`);
            continue;
          }
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
          args: Buffer.from([SystemAction.Move, turnNumber % 11]), // Move action and roll index
        });
        console.log(`Player ${playerId + 1} moved`);

        // Handle property purchase/rent and building
        try {
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
        } catch (error) {
          console.log(`Player ${playerId + 1} property action failed:`, error);
          // Check if player needs to mortgage properties
          const playerStateAccount = await provider.connection.getAccountInfo(playerEntities[playerId]);
          if (!playerStateAccount) {
            console.error(`Failed to fetch state for player ${playerId + 1}`);
            continue;
          }
          const playerState = PlayerState.deserialize(playerStateAccount.data);
          
          if (playerState.cash < 0) {
            // Try to mortgage properties to avoid bankruptcy
            for (const propId of playerState.properties) {
              try {
                await ApplySystem({
                  authority: provider.wallet.publicKey,
                  world: worldPda,
                  systemId: PROPERTY_SYSTEM,
                  entities: [{
                    entity: playerEntities[playerId],
                    components: [{ componentId: PLAYER_COMPONENT }],
                  }],
                  args: Buffer.from([SystemAction.MortgageProperty, propId]), // Mortgage action + property ID
                });
                console.log(`Player ${playerId + 1} mortgaged property ${propId}`);
                
                // Check if we have enough cash now
                const updatedStateAccount = await provider.connection.getAccountInfo(playerEntities[playerId]);
                if (!updatedStateAccount) {
                  console.error(`Failed to fetch updated state for player ${playerId + 1}`);
                  continue;
                }
                const updatedState = PlayerState.deserialize(updatedStateAccount.data);
                if (updatedState.cash >= 0) break;
              } catch (error) {
                console.log(`Failed to mortgage property ${propId}:`, error);
              }
            }
            
            // Check if player is still bankrupt after mortgaging
            const finalStateAccount = await provider.connection.getAccountInfo(playerEntities[playerId]);
            if (!finalStateAccount) {
              console.error(`Failed to fetch final state for player ${playerId + 1}`);
              continue;
            }
            const finalState = PlayerState.deserialize(finalStateAccount.data);
            if (finalState.cash < 0) {
              console.log(`Player ${playerId + 1} is bankrupt after mortgaging properties`);
              winnerFound = true;
              winnerId = await findWealthiestPlayer(provider.connection, playerEntities);
              break;
            }
          }
        }

        // Check for building opportunities
        const currentStateAccount = await provider.connection.getAccountInfo(playerEntities[playerId]);
        if (!currentStateAccount) {
          console.error(`Failed to fetch state for player ${playerId + 1}`);
          continue;
        }
        const currentState = PlayerState.deserialize(currentStateAccount.data);

        // Check each property group for monopolies
        for (const [groupId, properties] of Object.entries(PROPERTY_GROUPS)) {
          // Check if player owns all properties in the group
          const hasMonopoly = properties.every(propId => 
            currentState.properties.includes(propId)
          );

          if (hasMonopoly) {
            // Try to build houses/hotels if we have a monopoly
            for (const propId of properties) {
              const buildingCount = currentState.buildings.get(propId) || 0;
              
              // Can build if we have less than 4 houses or no hotel
              if (buildingCount < 5 && currentState.cash >= HOUSE_COST) {
                await ApplySystem({
                  authority: provider.wallet.publicKey,
                  world: worldPda,
                  systemId: PROPERTY_SYSTEM,
                  entities: [{
                    entity: playerEntities[playerId],
                    components: [{ componentId: PLAYER_COMPONENT }],
                  }],
                  args: Buffer.from([SystemAction.BuildProperty, propId]), // Building action + property ID
                });
                console.log(`Player ${playerId + 1} built on property ${propId}`);
              }
            }
          }
        }

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
