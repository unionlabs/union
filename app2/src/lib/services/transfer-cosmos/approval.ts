import { Effect } from "effect"
import type { Chain } from "$lib/schema/chain.ts"
import type { CosmosWalletId } from "$lib/wallet/cosmos"
import { executeCosmWasmInstructions } from "$lib/services/transfer-cosmos/execute.ts"
import type { ValidTransfer } from "$lib/schema/transfer-args.ts"
import { fromHex, isHex } from "viem"
import { isValidBech32ContractAddress } from "@unionlabs/client"

export const approveTransfer = (
  chain: Chain,
  connectedWallet: CosmosWalletId,
  params: ValidTransfer["args"]
) => {
  return Effect.gen(function* () {
    const decodedDenom = isHex(params.baseToken)
      ? fromHex(params.baseToken, "string")
      : params.baseToken

    const isNative = !isValidBech32ContractAddress(decodedDenom)

    if (isNative) {
      return yield* Effect.succeed("native-token-no-approval-needed")
    }

    const instructions = [
      {
        contractAddress: decodedDenom,
        msg: {
          increase_allowance: {
            spender: params.ucs03address,
            amount: params.baseAmount.toString()
          }
        }
      }
    ]

    return yield* executeCosmWasmInstructions(chain, connectedWallet, instructions)
  })
}
