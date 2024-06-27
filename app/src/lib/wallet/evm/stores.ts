import {
  getClient,
  getAccount,
  getChainId,
  watchClient,
  watchAccount,
  watchChainId,
  getConnectors,
  getConnections,
  watchConnectors,
  watchConnections
} from "@wagmi/core"
import { config } from "./config.ts"
import { readable, derived } from "svelte/store"

export const evmClient = readable(getClient(config), set => watchClient(config, { onChange: set }))
export const evmChainId = readable(getChainId(config), set =>
  watchChainId(config, { onChange: set })
)
export const evmAccount = readable(getAccount(config), set =>
  watchAccount(config, { onChange: set })
)
export const evmConnectors = readable(getConnectors(config), set =>
  watchConnectors(config, { onChange: set })
)
export const evmConnections = readable(getConnections(config), set =>
  watchConnections(config, { onChange: set })
)

export const evmProvider = derived(
  [evmAccount, evmConnectors, evmChainId],
  ([$account, $connectors, $chainId]) => {
    const connector = $connectors.find(async connector => await connector.isAuthorized())
    if (!connector) return undefined
    return connector.getProvider({ chainId: $chainId })
  }
)
