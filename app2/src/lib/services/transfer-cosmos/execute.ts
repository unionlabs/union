import type {CosmosWalletId} from "$lib/wallet/cosmos";
import type {Chain} from "$lib/schema/chain.ts";
import {Effect} from "effect";
import {CosmWasmError} from "$lib/services/transfer-cosmos/errors.ts";
import {getCosmWasmClient} from "$lib/services/cosmos/clients.ts";
import {getCosmosOfflineSigner} from "$lib/services/transfer-cosmos/offline-signer.ts";
import type {ExecuteInstruction} from "@cosmjs/cosmwasm-stargate";
import {isValidBech32Address, isValidBech32ContractAddress} from "@unionlabs/client";

export const executeCosmWasmInstructions = (
  chain: Chain,
  connectedWallet: CosmosWalletId,
  instructions: Array<ExecuteInstruction>
) =>
  Effect.gen(function* () {
    // Validate inputs
    if (!chain) {
      throw new CosmWasmError({
        cause: "Chain is undefined",
      });
    }

    if (!connectedWallet) {
      throw new CosmWasmError({
        cause: "Connected wallet is undefined",
      });
    }

    if (!instructions || instructions.length === 0) {
      throw new CosmWasmError({
        cause: "Instructions are empty or undefined",
      });
    }

    // Validate each instruction's contract address
    for (const instruction of instructions) {
      if (!instruction.contractAddress) {
        throw new CosmWasmError({
          cause: "Missing contractAddress in instruction",
        });
      }

      // Validate contract address format
      if (!isValidBech32ContractAddress(instruction.contractAddress)) {
        throw new CosmWasmError({
          cause: `Invalid contract address format: ${instruction.contractAddress}`,
        });
      }

      if (!instruction.msg) {
        throw new CosmWasmError({
          cause: "Missing msg in instruction",
        });
      }
    }

    const client = yield* getCosmWasmClient(chain, connectedWallet);

    if (!client) {
      throw new CosmWasmError({
        cause: "Client CosmWasm is undefined",
      });
    }

    const offlineSigner = yield* getCosmosOfflineSigner(chain, connectedWallet);

    if (!offlineSigner) {
      throw new CosmWasmError({
        cause: "Offline signer is undefined",
      });
    }

    // Get accounts
    const accounts = yield* Effect.tryPromise({
      try: () => offlineSigner.getAccounts(),
      catch: err => new CosmWasmError({
        cause: `Failed to get accounts: ${err}`,
      })
    });

    if (accounts.length === 0) {
      throw new CosmWasmError({
        cause: "No accounts found",
      });
    }

    const sender = accounts[0].address;

    // Validate sender address
    if (!isValidBech32Address(sender)) {
      throw new CosmWasmError({
        cause: `Invalid sender address format: ${sender}`,
      });
    }

    const formattedInstructions = instructions.map(instr => ({
      contractAddress: instr.contractAddress,
      msg: instr.msg,
      funds: instr.funds || []
    }));

    console.log("Sender:", sender);
    console.log("Formatted instructions:", JSON.stringify(formattedInstructions, null, 2));

    // Execute the transaction
    const result = yield* Effect.tryPromise({
      try: () => client.executeMultiple(
        sender,
        formattedInstructions,
        "auto"
      ),
      catch: err => {
        console.error("CosmWasm execution error:", err);
        return new CosmWasmError({
          cause: err instanceof Error ? err.message : String(err),
        });
      }
    });

    console.log("Transaction hash:", result.transactionHash);
    return result.transactionHash;
  });