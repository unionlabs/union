import { Effect, Option } from "effect"
import { switchChain as wagmiSwitchChain } from "@wagmi/core"
import { wagmiConfig, type ConfiguredChainId } from "$lib/wallet/evm/wagmi-config"
import { SwitchChainError } from "./errors.ts"
import type { SwitchChainErrorType } from "viem"
import type { Chain } from "$lib/schema/chain.ts"

export const switchChain = (chain: Chain) =>
  Effect.gen(function* () {
    const res = yield* Effect.tryPromise({
      try: () => {
        const i = chain.toViemChain()
        if (Option.isNone(i)) {
          throw new SwitchChainError({
            cause: { message: "Chain conversion failed" } as SwitchChainErrorType
          })
        }
        return wagmiSwitchChain(wagmiConfig, { chainId: i.value.id as ConfiguredChainId })
      },
      catch: err => new SwitchChainError({ cause: err as SwitchChainErrorType })
    })

    // Some wallets, like metamask, fulfill the promise before they are actually done with switching.
    // The time it takes for metamask to switch depends on the user's device. On a modern MacBook
    // it takes about 500ms. We 3x'ed that to be safe for slower devices.
    yield* Effect.sleep("1.5 seconds")
    return res
  })
