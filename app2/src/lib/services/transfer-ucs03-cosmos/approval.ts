import { executeCosmWasmInstructions } from "$lib/services/transfer-ucs03-cosmos/execute"
import { isValidBech32ContractAddress } from "$lib/utils"
import type { CosmosWalletId } from "$lib/wallet/cosmos"
import type { Chain, ValidTransfer } from "@unionlabs/sdk/schema"
import { Effect } from "effect"
import { fromHex, isHex } from "viem"

export const approveTransfer = (
  chain: Chain,
  params: ValidTransfer["args"],
) => {
  const decodedDenom = isHex(params.baseToken)
    ? fromHex(params.baseToken, "string")
    : params.baseToken

  const isNative = !isValidBech32ContractAddress(decodedDenom)

  if (isNative) {
    return Effect.succeed("native-token-no-approval-needed")
  }

  const instructions = [
    {
      contractAddress: decodedDenom,
      msg: {
        increase_allowance: {
          spender: params.ucs03address,
          amount: params.baseAmount.toString(),
        },
      },
    },
  ]

  return executeCosmWasmInstructions(chain, instructions)
}
