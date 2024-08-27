import type { Chain } from "$lib/types"
import { raise } from "$lib/utilities"
import {
  createPublicClient,
  createWalletClient,
  fallback,
  http,
  defineChain,
  type Chain as ViemChain,
  type CustomTransport
} from "viem"
import { arbitrumSepolia, scrollSepolia, berachainTestnetbArtio, sepolia } from "viem/chains"
import type { DiscriminatedUnion } from "$lib/utilities/types.ts"

export type TransferState = DiscriminatedUnion<
  "kind",
  {
    PRE_TRANSFER: {}
    FLIPPING: {}
    SWITCHING_TO_CHAIN: { warning?: Error }
    APPROVING_ASSET: { error?: Error }
    AWAITING_APPROVAL_RECEIPT: { error?: Error; hash: `0x${string}` }
    SIMULATING_TRANSFER: { warning?: Error }
    CONFIRMING_TRANSFER: { error?: Error; contractRequest: unknown }
    AWAITING_TRANSFER_RECEIPT: { error?: Error; transferHash: `0x${string}` }
    TRANSFERRING: { transferHash: string }
    TRANSFERRED: { transferHash: string }
  }
>

export const transferStep = (state: TransferState): number => {
  // biome-ignore lint/nursery/useDefaultSwitchClause: i want typescript to error if we forgot a case
  switch (state.kind) {
    case "PRE_TRANSFER":
      return 1
    case "FLIPPING":
      return 2
    case "SWITCHING_TO_CHAIN":
      return 3
    case "APPROVING_ASSET":
      return 4
    case "AWAITING_APPROVAL_RECEIPT":
      return 5
    case "SIMULATING_TRANSFER":
      return 6
    case "CONFIRMING_TRANSFER":
      return 7
    case "AWAITING_TRANSFER_RECEIPT":
      return 8
    case "TRANSFERRING":
      return 9
    case "TRANSFERRED":
      return 10
  }
}

export const stepBefore = (state: TransferState, targetStateKind: TransferState["kind"]): boolean =>
  // @ts-ignore
  transferStep(state) < transferStep({ kind: targetStateKind })

export const stepAfter = (state: TransferState, targetStateKind: TransferState["kind"]): boolean =>
  // @ts-ignore
  transferStep(state) > transferStep({ kind: targetStateKind })

export const chainToViemChain = (chain: Chain): ViemChain => {
  const rpcUrls = chain.rpcs.filter(c => c.type === "rpc").map(c => `https://${c.url}`)

  if (rpcUrls.length === 0) raise(`No RPC url for ${chain.display_name}`)

  const nativeCurrency = chain.assets.filter(asset => asset.denom === "native").at(0)

  if (nativeCurrency === undefined) raise(`No native currency for ${chain.display_name}`)

  return chain.chain_id === "11155111"
    ? sepolia
    : chain.chain_id === "80084"
      ? berachainTestnetbArtio
      : chain.chain_id === "421614"
        ? arbitrumSepolia
        : chain.chain_id === "534351"
          ? scrollSepolia
          : defineChain({
              name: chain.display_name,
              nativeCurrency: {
                name: nativeCurrency.display_name ?? nativeCurrency.display_symbol,
                /** 2-6 characters long */
                symbol: nativeCurrency.display_symbol,
                decimals: nativeCurrency.decimals
              },
              id: Number(chain.chain_id),
              rpcUrls: {
                default: {
                  http: rpcUrls
                }
              },
              testnet: chain.testnet
            })
}

export const createViemClients = (
  chain: Chain,
  walletTransport: CustomTransport
): {
  publicClient: ReturnType<typeof createPublicClient>
  walletClient: ReturnType<typeof createWalletClient>
} => {
  const viemChain = chainToViemChain(chain)

  // TODO: make this dry given its already done in chainToViemChain
  const rpcUrls = chain.rpcs.filter(c => c.type === "rpc").map(c => `https://${c.url}`)

  const publicClient = createPublicClient({
    chain: viemChain,
    transport: fallback(rpcUrls.map(url => http(url)))
  })

  const walletClient = createWalletClient({
    chain: viemChain,
    transport: walletTransport
  })

  return { publicClient, walletClient }
}
