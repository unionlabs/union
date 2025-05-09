import { getAccountError } from "$lib/services/transfer-ucs03-evm/errors.ts"
import { getWagmiConfig } from "$lib/wallet/evm/wagmi-config.svelte.ts"
import { getAccount as getConnectedAccount } from "@wagmi/core"
import { Effect } from "effect"

export const getAccount = Effect.gen(function*() {
  return yield* Effect.try({
    try: () => getConnectedAccount(getWagmiConfig()),
    catch: () => new getAccountError({ cause: "Could not get connected account" }),
  })
})
