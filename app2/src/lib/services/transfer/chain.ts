import { type ConfiguredChainId, getWagmiConfig } from "$lib/wallet/evm/wagmi-config.svelte.ts"
import { switchChain as wagmiSwitchChain } from "@wagmi/core"
import { Effect } from "effect"
import type { SwitchChainErrorType } from "viem"
import { EvmSwitchChainError } from "../transfer-ucs03-evm/errors"

export const switchChain = (chainId: ConfiguredChainId) =>
  Effect.tryPromise({
    try: () => wagmiSwitchChain(getWagmiConfig(), { chainId }),
    catch: err => new EvmSwitchChainError({ cause: err as SwitchChainErrorType }),
  })
