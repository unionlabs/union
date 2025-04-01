import { Effect } from "effect"
import { switchChain as wagmiSwitchChain } from "@wagmi/core"
import { type ConfiguredChainId, wagmiConfig } from "$lib/wallet/evm/wagmi-config"
import { SwitchChainError } from "./errors.ts"
import type { SwitchChainErrorType } from "viem"

export const switchChain = (chainId: ConfiguredChainId) =>
  Effect.tryPromise({
    try: () => wagmiSwitchChain(wagmiConfig, { chainId }),
    catch: err => new SwitchChainError({ cause: err as SwitchChainErrorType })
  })
