import { browser } from "$app/environment"
import { type ChainConfig, CHAINS, DEFAULT_CHAIN } from "$lib/chains/config"

const STORAGE_KEY = "selected-chain"

// Get initial value from localStorage or default
function getInitialChain(): string {
  if (!browser) {
    return DEFAULT_CHAIN
  }
  const stored = localStorage.getItem(STORAGE_KEY)
  if (stored && CHAINS[stored]) {
    return stored
  }
  return DEFAULT_CHAIN
}

let universalChainId = $state<string>(getInitialChain())

export const chainStore = {
  // The universal chain ID (used in URLs and as key)
  get id() {
    return universalChainId
  },
  // Legacy alias
  get name() {
    return universalChainId
  },
  get config(): ChainConfig {
    return CHAINS[universalChainId] ?? CHAINS[DEFAULT_CHAIN]
  },
  set(id: string) {
    if (!CHAINS[id]) {
      return
    }
    universalChainId = id
    if (browser) {
      localStorage.setItem(STORAGE_KEY, id)
    }
  },
}
