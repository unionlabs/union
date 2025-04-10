import { Effect } from "effect"
import { switchChain as wagmiSwitchChain } from "@wagmi/core"
import { type ConfiguredChainId, wagmiConfig } from "$lib/wallet/evm/wagmi-config"
import { SwitchChainError } from "./errors.ts"
import type { SwitchChainErrorType } from "viem"

export const switchChainAptos = (chainId: ConfiguredChainId) =>
  Effect.gen(function* () {
    console.info("switching chain", chainId)
    // TODO: Find a way to switch chain in aptos.
    const res = yield* Effect.tryPromise({
      try: () => wagmiSwitchChain(wagmiConfig, { chainId }),
      catch: err => new SwitchChainError({ cause: err as SwitchChainErrorType })
    })
    // Some wallets, like metamask, fulfill the promise before they are actually done with switching.
    // The time it takes for metamask to switch depends on the user's device. On a modern MacBook
    // it takes about 500ms. We 3x'ed that to be safe for slower devices.
    yield* Effect.sleep("1.5 seconds")
    return res
  })
