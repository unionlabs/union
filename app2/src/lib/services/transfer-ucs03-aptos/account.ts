import { getAccountError } from "$lib/services/transfer-ucs03-evm/errors"
import { getAptosWallet } from "$lib/wallet/aptos/index"
import { Effect } from "effect"

export const getAccount = Effect.gen(function*() {
  return yield* Effect.try({
    try: () => getAptosWallet(),
    catch: () => new getAccountError({ cause: "Could not get connected account" }),
  })
})
