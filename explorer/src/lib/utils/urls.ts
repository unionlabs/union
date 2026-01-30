// URL utilities for chain-scoped routing

import { page } from "$app/stores"
import { get } from "svelte/store"

// Get the current chainId (universal_chain_id) from URL
export function getChainId(): string {
  const $page = get(page)
  return $page.params.chainId ?? "union.union-1"
}

// Build a URL with the current chain prefix
export function chainUrl(path: string): string {
  const chainId = getChainId()
  // Ensure path starts with /
  const normalizedPath = path.startsWith("/") ? path : `/${path}`
  // Don't double up the prefix
  if (normalizedPath.startsWith(`/${chainId}`)) {
    return normalizedPath
  }
  return `/${chainId}${normalizedPath}`
}

// Build URLs for common routes
export const urls = {
  home: () => chainUrl(""),
  blocks: () => chainUrl("/blocks"),
  block: (height: string | number) => chainUrl(`/blocks/${height}`),
  transactions: () => chainUrl("/transactions"),
  transaction: (hash: string) => chainUrl(`/transactions/${hash}`),
  validators: () => chainUrl("/validators"),
  validator: (address: string) => chainUrl(`/validators/${address}`),
  accounts: () => chainUrl("/account"),
  account: (address: string) => chainUrl(`/account/${address}`),
  governance: () => chainUrl("/governance"),
  proposal: (id: string | number) => chainUrl(`/governance/${id}`),
  parameters: () => chainUrl("/parameters"),
  ibc: () => chainUrl("/ibc"),
}
