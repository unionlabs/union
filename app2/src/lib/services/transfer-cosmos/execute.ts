import type {CosmosWalletId} from "$lib/wallet/cosmos";
import type {Chain} from "$lib/schema/chain.ts";
import {Effect} from "effect";
import {CosmWasmError} from "$lib/services/transfer-cosmos/errors.ts";
import {getCosmWasmClient} from "$lib/services/cosmos/clients.ts";
import {getCosmosOfflineSigner} from "$lib/services/transfer-cosmos/offline-signer.ts";

export const executeCosmWasmInstructions = (
  chain: Chain,
  connectedWallet: CosmosWalletId,
  instructions: any
) =>
  Effect.gen(function* () {

    const client = yield* getCosmWasmClient(chain, connectedWallet)

    if (!client) {
      yield* Effect.fail(new CosmWasmError({
        cause: "Client CosmWasm is undefined",
      }))
      return null as never
    }

    const offlineSigner = yield* getCosmosOfflineSigner(chain, connectedWallet)

    if (!offlineSigner) {
      yield* Effect.fail(new CosmWasmError({
        cause: "Offline signer is undefined",
      }))
      return null as never
    }

    const accounts = yield* Effect.tryPromise({
      try: () => offlineSigner.getAccounts(),
      catch: err => new CosmWasmError({
        cause: `Failed to get accounts: ${err}`,
      })
    })

    if (accounts.length === 0) {
      yield* Effect.fail(new CosmWasmError({
        cause: "No accounts found",
      }))
      return null as never
    }

    const sender = accounts[0].address

    const formattedInstructions = instructions.map(instr => ({
      contractAddress: instr.contractAddress,
      msg: instr.msg,
      funds: instr.funds || []
    }))

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
