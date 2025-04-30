import { Effect } from "effect"
import { switchChain as wagmiSwitchChain } from "@wagmi/core"
import { type ConfiguredChainId, getWagmiConfig } from "$lib/wallet/evm/wagmi-config.svelte.ts"
import { SwitchChainError } from "./errors.ts"
import type { SwitchChainErrorType } from "viem"

export const switchChain = (chainId: ConfiguredChainId) =>
  Effect.tryPromise({
    try: () => wagmiSwitchChain(getWagmiConfig(), { chainId }),
    catch: err => new SwitchChainError({ cause: err as SwitchChainErrorType })
  })
