import { Effect } from "effect"
import type { Chain } from "$lib/schema/chain.ts"
import { CosmWasmError } from "$lib/services/transfer-cosmos/errors.ts"
import { executeCosmWasmInstructions } from "$lib/services/transfer-cosmos/execute.ts"
import { cosmosStore } from "$lib/wallet/cosmos"
import type { ValidTransfer } from "$lib/schema/transfer-args.ts"
import { generateSalt } from "$lib/services/shared/salt.ts"
import { fromHex, isHex } from "viem"
import { isValidBech32ContractAddress } from "$lib/utils"

export const submitTransfer = (chain: Chain, transfer: ValidTransfer["args"]) =>
  Effect.gen(function* () {
    const { connectedWallet } = cosmosStore

    if (!connectedWallet) {
      throw new CosmWasmError({
        cause: "No wallet connected"
      })
    }

    const {
      baseAmount,
      baseToken,
      quoteAmount,
      quoteToken,
      receiver,
      sourceChannelId,
      ucs03address
    } = transfer

    if (!ucs03address) {
      throw new CosmWasmError({
        cause: "Missing UCS03 contract address"
      })
    }

    // Decode the token denom
    const decodedDenom = isHex(baseToken) ? fromHex(baseToken, "string") : baseToken

    const isNative = !isValidBech32ContractAddress(decodedDenom)
    const formattedBaseToken = decodedDenom
    const funds = isNative ? [{ amount: baseAmount.toString(), denom: decodedDenom }] : []

    const salt = yield* generateSalt

    const instructions = [
      {
        contractAddress: ucs03address,
        msg: {
          transfer: {
            channel_id: sourceChannelId,
            receiver: receiver,
            base_token: formattedBaseToken,
            base_amount: baseAmount,
            quote_token: quoteToken,
            quote_amount: quoteAmount,
            timeout_height: 1000000000,
            timeout_timestamp: 0,
            salt
          }
        },
        funds
      }
    ]

    // Use the executeCosmWasmInstructions function to execute the transfer
    return yield* executeCosmWasmInstructions(chain, connectedWallet, instructions)
  })
