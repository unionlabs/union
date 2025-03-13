import type { CosmosWalletId } from "$lib/wallet/cosmos"
import type { Chain } from "$lib/schema/chain.ts"
import { Effect } from "effect"
import { CosmWasmError } from "$lib/services/transfer-ucs03-cosmos/errors.ts"
import { getCosmWasmClient } from "$lib/services/cosmos/clients.ts"
import { getCosmosOfflineSigner } from "$lib/services/transfer-ucs03-cosmos/offline-signer.ts"
import type { ExecuteInstruction } from "@cosmjs/cosmwasm-stargate"
import { isValidBech32Address, isValidBech32ContractAddress } from "@unionlabs/client"

export const executeCosmWasmInstructions = (
  chain: Chain,
  connectedWallet: CosmosWalletId,
  instructions: Array<ExecuteInstruction>
) => {
  // Early validations
  if (!chain) {
    return Effect.fail(new CosmWasmError({ cause: "Chain is undefined" }))
  }

  if (!connectedWallet) {
    return Effect.fail(new CosmWasmError({ cause: "Connected wallet is undefined" }))
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
          cause: `Invalid contract address format: ${instruction.contractAddress}`
        })
      )
    }

    if (!instruction.msg) {
      return Effect.fail(new CosmWasmError({ cause: "Missing msg in instruction" }))
    }
  }

  return Effect.flatMap(
    Effect.mapError(
      getCosmWasmClient(chain, connectedWallet),
      err => new CosmWasmError({ cause: String(err) })
    ),
    client => {
      if (!client) {
        return Effect.fail(new CosmWasmError({ cause: "Client CosmWasm is undefined" }))
      }

      return Effect.flatMap(
        Effect.mapError(
          getCosmosOfflineSigner(chain, connectedWallet),
          err => new CosmWasmError({ cause: String(err) })
        ),
        offlineSigner => {
          if (!offlineSigner) {
            return Effect.fail(new CosmWasmError({ cause: "Offline signer is undefined" }))
          }

          return Effect.flatMap(
            Effect.tryPromise({
              try: () => offlineSigner.getAccounts(),
              catch: err => new CosmWasmError({ cause: `Failed to get accounts: ${err}` })
            }),
            accounts => {
              if (accounts.length === 0) {
                return Effect.fail(new CosmWasmError({ cause: "No accounts found" }))
              }

              const sender = accounts[0].address

              if (!isValidBech32Address(sender)) {
                return Effect.fail(
                  new CosmWasmError({
                    cause: `Invalid sender address format: ${sender}`
                  })
                )
              }

              const formattedInstructions = instructions.map(instr => ({
                contractAddress: instr.contractAddress,
                msg: instr.msg,
                funds: instr.funds || []
              }))

              console.log("Sender:", sender)
              console.log("Formatted instructions:", JSON.stringify(formattedInstructions, null, 2))

              return Effect.map(
                Effect.tryPromise({
                  try: () => client.executeMultiple(sender, formattedInstructions, "auto"),
                  catch: err => {
                    console.error("CosmWasm execution error:", err)
                    return new CosmWasmError({
                      cause: err instanceof Error ? err.message : String(err)
                    })
                  }
                }),
                result => {
                  console.log("Transaction hash:", result.transactionHash)
                  return result.transactionHash
                }
              )
            }
          )
        }
      )
    }
  )
}
