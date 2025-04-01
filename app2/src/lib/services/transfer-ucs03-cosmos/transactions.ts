import { Effect } from "effect"
import { CosmWasmError } from "$lib/services/transfer-ucs03-cosmos/errors.ts"
import { executeCosmWasmInstructions } from "$lib/services/transfer-ucs03-cosmos/execute.ts"
import { cosmosStore } from "$lib/wallet/cosmos"
import type { ValidTransfer } from "@unionlabs/sdk/schema"
import { generateSalt } from "$lib/services/shared"
import { fromHex, isHex } from "viem"
import { isValidBech32ContractAddress } from "$lib/utils"

export const submitTransfer = (transfer: ValidTransfer["args"]) => {
  const { connectedWallet } = cosmosStore

  if (!connectedWallet) {
    return Effect.fail(new CosmWasmError({ cause: "No wallet connected" }))
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
    return Effect.fail(new CosmWasmError({ cause: "Missing UCS03 contract address" }))
  }

  const decodedDenom = isHex(baseToken) ? fromHex(baseToken, "string") : baseToken
  const isNative = !isValidBech32ContractAddress(decodedDenom)
  const formattedBaseToken = decodedDenom
  const funds = isNative ? [{ amount: baseAmount.toString(), denom: decodedDenom }] : []

  return Effect.flatMap(generateSalt, salt => {
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

    return Effect.mapError(
      executeCosmWasmInstructions(transfer.sourceChain, connectedWallet, instructions),
      err => err
    )
  })
}
