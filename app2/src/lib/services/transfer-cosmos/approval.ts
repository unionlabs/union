import { Effect } from "effect"
import type { Chain } from "$lib/schema/chain.ts"
import type { CosmosWalletId } from "$lib/wallet/cosmos"
import { executeCosmWasmInstructions } from "$lib/services/transfer-cosmos/execute.ts"

export const approveTransfer = (
  chain: Chain,
  connectedWallet: CosmosWalletId,
  params: {
    contractAddress: string;
    amount: bigint;
    spender: string;
  }
) =>
  Effect.succeed([{
    contractAddress: params.contractAddress,
    msg: {
      increase_allowance: {
        spender: params.spender,
        amount: params.amount.toString()
      }
    }
  }]).pipe(
    Effect.flatMap(instructions =>
      executeCosmWasmInstructions(chain, connectedWallet, instructions)
    )
  )