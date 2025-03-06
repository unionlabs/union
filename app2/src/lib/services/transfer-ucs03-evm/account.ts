import { wagmiConfig } from "$lib/wallet/evm/wagmi-config.ts"
import { Effect } from "effect"
import { getAccountError } from "$lib/services/transfer-ucs03-evm/errors.ts"
import { getAccount as getConnectedAccount } from "@wagmi/core"

export const getAccount = Effect.gen(function* () {
  return yield* Effect.try({
    try: () => getConnectedAccount(wagmiConfig),
    catch: () => new getAccountError({ cause: "Could not get connected account" })
  })
})
