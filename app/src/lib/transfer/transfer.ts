import type { Chain } from "$lib/types";
import { raise } from "$lib/utilities";
import { type CustomTransportConfig, createPublicClient, createWalletClient, fallback, http, custom,  defineChain, type PublicClient, type Chain as ViemChain, type CustomTransport, type SimulateContractReturnType } from "viem";
import { berachainTestnetbArtio, sepolia } from "viem/chains";


export type DiscriminatedUnion<K extends PropertyKey, T extends object> = {
  [P in keyof T]: ({ [Q in K]: P } & T[P]) extends infer U ? { [Q in keyof U]: U[Q] } : never
}[keyof T]

export type TransferState = DiscriminatedUnion<"kind", {
  PRE_TRANSFER: {},
  FLIPPING: {},
  ADDING_CHAIN: { error?: Error },
  SWITCHING_TO_CHAIN: { error?: Error },
  APPROVING_ASSET: { error?: Error},
  AWAITING_APPROVAL_RECEIPT: { error?: Error, hash: `0x${string}` },
  SIMULATING_TRANSFER: { error?: Error },
  CONFIRMING_TRANSFER: { error?: Error, simulationResult: SimulateContractReturnType },
  AWAITING_TRANSFER_RECEIPT: { error?: Error, transferHash: `0x${string}` },
  TRANSFERRING: { transferHash: string}
}>;

export const transferStep = (state: TransferState): number => {
  switch (state.kind) {
    case "PRE_TRANSFER": return 1;
    case "FLIPPING": return 2;
    case "ADDING_CHAIN": return 3;
    case "SWITCHING_TO_CHAIN": return 4;
    case "APPROVING_ASSET": return 5;
    case "AWAITING_APPROVAL_RECEIPT": return 6;
    case "SIMULATING_TRANSFER": return 7;
    case "CONFIRMING_TRANSFER": return 8;
    case "AWAITING_TRANSFER_RECEIPT": return 9;
    case "TRANSFERRING": return 10;
  }
} 

export const stepBefore = (state: TransferState, targetStateKind: TransferState['kind']): boolean =>
  // @ts-ignore
  transferStep(state) < transferStep({ kind: targetStateKind})


export const chainToViemChain = (chain: Chain): ViemChain => {
    const rpcUrls = chain.rpcs.filter(c => c.type === "rpc").map(c => `https://${c.url}`)

    if (rpcUrls.length === 0) raise(`No RPC url for ${chain.display_name}`)
  
    const nativeCurrency = chain.assets.filter(asset => asset.denom === "native").at(0);

    if (nativeCurrency === undefined) raise(`No native currency for ${chain.display_name}`)

    return chain.chain_id === "11155111"
        ? sepolia
        : chain.chain_id === "80084"
          ? berachainTestnetbArtio
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

export const createViemClients = (chain: Chain, walletTransport: CustomTransport): {publicClient: ReturnType<typeof createPublicClient>, walletClient: ReturnType<typeof createWalletClient>} => {
    const viemChain = chainToViemChain(chain);

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

    return {publicClient, walletClient};
};
