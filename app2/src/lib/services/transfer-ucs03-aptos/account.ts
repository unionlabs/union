import { Effect } from "effect"
import { getAccountError } from "$lib/services/transfer-ucs03-evm/errors.ts"
import { getAptosWallet } from "$lib/wallet/aptos/index.ts"

export const getAccount = Effect.gen(function* () {
  return yield* Effect.try({
    try: () => getAptosWallet(),
    catch: () => new getAccountError({ cause: "Could not get connected account" })
  })
})
