import type { Chain } from "$lib/types"

export const toDisplayName = (chain_id: string | undefined | null, chains: Array<Chain>): string =>
  chains.find(c => c.chain_id === chain_id)?.display_name ?? "unknown chain"
