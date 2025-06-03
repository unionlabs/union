import { CosmWasmError } from "$lib/services/transfer-ucs03-cosmos/errors.ts"
import { executeCosmWasmInstructions } from "$lib/services/transfer-ucs03-cosmos/execute.ts"
import { isValidBech32ContractAddress } from "$lib/utils"
import { cosmosStore } from "$lib/wallet/cosmos"
import type { ValidTransfer } from "@unionlabs/sdk/schema"
import { generateSalt } from "@unionlabs/sdk/utils"
import { Effect } from "effect"
import { fromHex, isHex } from "viem"

/**
 * @deprecated Replace with SDK usage
 */
export const submitTransfer = (transfer: ValidTransfer["args"]) => {
  const { connectedWallet } = cosmosStore

  if (!connectedWallet) {
    return Effect.fail(new CosmWasmError({ cause: "No wallet connected" }))
  }

  const {
    baseAmount,
    baseToken,
    quoteAmount,
    // @ts-expect-error 2339
    quoteToken,
    receiver,
    sourceChannelId,
    ucs03address,
  } = transfer

  if (!ucs03address) {
    return Effect.fail(new CosmWasmError({ cause: "Missing UCS03 contract address" }))
  }

  const decodedDenom = isHex(baseToken) ? fromHex(baseToken, "string") : baseToken
  const isNative = !isValidBech32ContractAddress(decodedDenom)
  const formattedBaseToken = decodedDenom
  const funds = isNative ? [{ amount: baseAmount.toString(), denom: decodedDenom }] : []

  return Effect.flatMap(generateSalt("cosmos"), salt => {
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
            salt,
          },
        },
        funds,
      },
    ]

    return Effect.mapError(
      executeCosmWasmInstructions(transfer.sourceChain, instructions),
      err => err,
    )
  })
}
