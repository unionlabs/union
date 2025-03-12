import type { Chain } from "$lib/schema/chain.ts"
import type { CosmosWalletId } from "$lib/wallet/cosmos"
import {executeCosmWasmInstructions} from "$lib/services/transfer-cosmos/execute.ts";
import {Effect} from "effect";
import {type CosmWasmError, OfflineSignerError} from "$lib/services/transfer-cosmos/errors.ts";

export const approveTransfer = (
  chain: Chain,
  connectedWallet: CosmosWalletId,
  params: {
    contractAddress: string;
    amount: bigint;
    spender: string;
  }
): Effect.Effect<string, CosmWasmError | OfflineSignerError, never> => {
  const instructions = [{
    contractAddress: params.contractAddress,
    msg: {
      increase_allowance: {
        spender: params.spender,
        amount: params.amount.toString()
      }
    }
  }];

  return executeCosmWasmInstructions(chain, connectedWallet, instructions);
}
