import type {CosmosWalletId} from "$lib/wallet/cosmos";
import type {Chain} from "$lib/schema/chain.ts";
import {Effect} from "effect";
import {CosmWasmError} from "$lib/services/transfer-cosmos/errors.ts";
import {getCosmWasmClient} from "$lib/services/cosmos/clients.ts";
import {getCosmosOfflineSigner} from "$lib/services/transfer-cosmos/offline-signer.ts";
import type {ExecuteInstruction} from "@cosmjs/cosmwasm-stargate";

export const executeCosmWasmInstructions = (
  chain: Chain,
  connectedWallet: CosmosWalletId,
  instructions: Array<ExecuteInstruction>
) =>
  Effect.gen(function* () {
    // Get the client
    const client = yield* getCosmWasmClient(chain, connectedWallet)

    if (!client) {
      throw new CosmWasmError({
        cause: "Client CosmWasm is undefined",
      })
    }

    // Get the offline signer
    const offlineSigner = yield* getCosmosOfflineSigner(chain, connectedWallet)

    if (!offlineSigner) {
      throw new CosmWasmError({
        cause: "Offline signer is undefined",
      })
    }

    // Get accounts
    const accounts = yield* Effect.tryPromise({
      try: () => offlineSigner.getAccounts(),
      catch: err => new CosmWasmError({
        cause: `Failed to get accounts: ${err}`,
      })
    })

    if (accounts.length === 0) {
      throw new CosmWasmError({
        cause: "No accounts found",
      })
    }

    const sender = accounts[0].address

    // Format instructions
    const formattedInstructions = instructions.map(instr => ({
      contractAddress: instr.contractAddress,
      msg: instr.msg,
      funds: instr.funds || []
    }))

    // Execute the transaction
    const result = yield* Effect.tryPromise({
      try: () => client.executeMultiple(
        sender,
        formattedInstructions,
        "auto"
      ),
      catch: err => new CosmWasmError({
        cause: String(err),
      })
    })

    return result.transactionHash
  })