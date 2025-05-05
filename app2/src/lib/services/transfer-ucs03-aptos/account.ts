import { getAccountError } from "$lib/services/transfer-ucs03-evm/errors.ts"
import { getAptosWallet } from "$lib/wallet/aptos/index.ts"
import { Effect } from "effect"

export const getAccount = Effect.gen(function*() {
  return yield* Effect.try({
    try: () => getAptosWallet(),
    catch: () => new getAccountError({ cause: "Could not get connected account" }),
  })
})
