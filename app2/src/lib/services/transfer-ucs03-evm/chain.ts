import { type ConfiguredChainId, getWagmiConfig } from "$lib/wallet/evm/wagmi-config.svelte"
import { switchChain as wagmiSwitchChain } from "@wagmi/core"
import { Effect } from "effect"
import type { Chain as ViemChain, SwitchChainErrorType } from "viem"
import { EvmSwitchChainError } from "./errors"

export const switchChain = (chain: ViemChain) =>
  Effect.gen(function*() {
    const res = yield* Effect.tryPromise({
      try: () => wagmiSwitchChain(getWagmiConfig(), { chainId: chain.id as ConfiguredChainId }),
      catch: err => new EvmSwitchChainError({ cause: err as SwitchChainErrorType }),
    })

    // Some wallets, like metamask, fulfill the promise before they are actually done with switching.
    // The time it takes for metamask to switch depends on the user's device. On a modern MacBook
    // it takes about 500ms. We 3x'ed that to be safe for slower devices.
    yield* Effect.sleep("1.5 seconds")
    return res
  })
