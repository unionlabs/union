import type { Chain } from 'viem'
import { sepolia } from 'viem/chains'

/**
 * Union chain ID is `6` on testnet: 'union-testnet-6'
 */
export const chainIds = ['1', '6', '11155111'] as const
export type ChainId = (typeof chainIds)[number]

export const isValidChainId = (chainId: string): chainId is ChainId => chainIds.includes(chainId)

/**
 * TODO: Add `ethereum.mainnet` and `union.mainnet` info on launch
 */
export const chain = {
  ethereum: {
    sepolia: {
      ...sepolia,
      portId: process.env.UCS01_SEPOLIA_PORT_ID || 'ucs01-relay',
      channelId: process.env.UCS01_SEPOLIA_SOURCE_CHANNEL || 'channel-0',
      token: {
        name: 'Union',
        symbol: 'UNO',
        denom: 'muno',
        decimals: 18,
        address: '0x',
      },
    },
  },
  union: {
    testnet: {
      name: process.env.UNION_CHAIN_ID || 'union-testnet-6',
      // id: Number(process.env?.UNION_CHAIN_ID.split('-')?.at(-1) || 6) || 6,
      id: 6,
      channelId: process.env.UCS01_UNION_SOURCE_CHANNEL,
      rpcUrls: {
        default: {
          /**
           * @see https://docs.union.build/joining_the_testnet/public_endpoints#rpc
           */
          http: ['https://rpc.testnet.bonlulu.uno', 'https://union-testnet-rpc.polkachu.com'],
        },
      },
      nativeCurrency: {
        name: 'Union',
        symbol: 'UNO',
        decimals: 6,
      },
      token: {
        name: 'Union',
        symbol: 'UNO',
        denom: 'muno',
        decimals: 6,
        address:
          process.env.UCS01_UNION_ADDRESS ||
          'union14pfzjnvzacqsmgjyf0avksc8cr70hsyt5epzcp66tmjpswf8sq8sn5meuy',
      },
    },
  },
} as const satisfies TChain

type Token = {
  name: string
  symbol: string
  denom: string
  decimals: number
  address: string
}

type TChain =
  | Record<
      'ethereum',
      Record<'sepolia', Chain & { token: Token; portId: string; channelId: string }>
    >
  | Record<'union', Record<'testnet', Chain & { token: Token; channelId: string }>>
