import { Effect } from "effect"
import type { Chain } from "$lib/schema/chain.ts"
import type { CosmosWalletId } from "$lib/wallet/cosmos"
import { executeCosmWasmInstructions } from "$lib/services/transfer-cosmos/execute.ts"
import type {ValidTransfer} from "$lib/schema/transfer-args.ts";
import {fromHex} from "viem";

export const approveTransfer = (
  chain: Chain,
  connectedWallet: CosmosWalletId,
  params: ValidTransfer["args"]
) => {
  return Effect.succeed([{
    contractAddress: fromHex(params.baseToken, "string"),
    msg: {
      increase_allowance: {
        spender: params.ucs03address,
        amount: params.baseAmount.toString()
      }
    }
  }]).pipe(
    Effect.flatMap(instructions => {
      return executeCosmWasmInstructions(chain, connectedWallet, instructions);
    })
  )
}