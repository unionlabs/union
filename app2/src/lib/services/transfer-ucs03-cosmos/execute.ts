import { getCosmWasmClient } from "$lib/services/cosmos/clients"
import { CosmWasmError } from "$lib/services/transfer-ucs03-cosmos/errors"
import { getCosmosOfflineSigner } from "$lib/services/transfer-ucs03-cosmos/offline-signer"
import { isValidBech32ContractAddress } from "$lib/utils"
import { isValidBech32Address } from "$lib/utils/format"
import type { ExecuteInstruction } from "@cosmjs/cosmwasm-stargate"
import type { Chain } from "@unionlabs/sdk/schema"
import { Effect } from "effect"

export const executeCosmWasmInstructions = (
  chain: Chain,
  instructions: Array<ExecuteInstruction>,
) => {
  // Early validations
  if (!chain) {
    return Effect.fail(new CosmWasmError({ cause: "Chain is undefined" }))
  }

  if (!instructions || instructions.length === 0) {
    return Effect.fail(new CosmWasmError({ cause: "Instructions are empty or undefined" }))
  }

  for (const instruction of instructions) {
    if (!instruction.contractAddress) {
      return Effect.fail(new CosmWasmError({ cause: "Missing contractAddress in instruction" }))
    }

    if (!isValidBech32ContractAddress(instruction.contractAddress)) {
      return Effect.fail(
        new CosmWasmError({
          cause: `Invalid contract address format: ${instruction.contractAddress}`,
        }),
      )
    }

    if (!instruction.msg) {
      return Effect.fail(new CosmWasmError({ cause: "Missing msg in instruction" }))
    }
  }

  return Effect.flatMap(
    Effect.mapError(
      getCosmWasmClient(chain),
      err => new CosmWasmError({ cause: String(err) }),
    ),
    client => {
      if (!client) {
        return Effect.fail(new CosmWasmError({ cause: "Client CosmWasm is undefined" }))
      }

      return Effect.flatMap(getCosmosOfflineSigner(chain), offlineSigner => {
        if (!offlineSigner) {
          return Effect.fail(new CosmWasmError({ cause: "Offline signer is undefined" }))
        }

        return Effect.flatMap(
          Effect.tryPromise({
            try: () => offlineSigner.getAccounts(),
            catch: err => new CosmWasmError({ cause: `Failed to get accounts: ${err}` }),
          }),
          accounts => {
            if (accounts.length === 0) {
              return Effect.fail(new CosmWasmError({ cause: "No accounts found" }))
            }

            const sender = accounts[0].address

            if (!isValidBech32Address(sender)) {
              return Effect.fail(
                new CosmWasmError({
                  cause: `Invalid sender address format: ${sender}`,
                }),
              )
            }

            const formattedInstructions = instructions.map(instr => ({
              contractAddress: instr.contractAddress,
              msg: instr.msg,
              funds: instr.funds || [],
            }))

            return Effect.map(
              Effect.tryPromise({
                try: () => client.executeMultiple(sender, formattedInstructions, "auto"),
                catch: err => {
                  console.error("CosmWasm execution error:", err)
                  return new CosmWasmError({
                    cause: err instanceof Error ? err.message : String(err),
                  })
                },
              }),
              result => {
                return result.transactionHash
              },
            )
          },
        )
      })
    },
  )
}
