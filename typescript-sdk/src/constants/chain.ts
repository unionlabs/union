import type { Chain } from 'viem'
import { sepolia } from 'viem/chains'

export const chainIds = ['1', '32382', '11155111'] as const
export type ChainId = (typeof chainIds)[number]

/**
 * TODO: Add `ethereum.mainnet` and `union.mainnet` info on launch
 */
export const chain = {
  ethereum: {
    sepolia: {
      ...sepolia,
      portId: process.env.UCS01_SEPOLIA_PORT_ID ?? 'ucs01-relay',
      channelId: process.env.UCS01_SEPOLIA_SOURCE_CHANNEL ?? 'channel-0',
      token: {
        name: 'Union',
        symbol: 'UNO',
        denom: 'muno',
        decimals: 6,
        address:
          process.env.MUNO_ERC20_ADDRESS ??
          '0xbCe4f3C33B330800ac11208e2726a8551B3d0E99'
      }
    }
  },
  union: {
    testnet: {
      name: 'union-testnet-1',
      id: 32_382,
      channelId: process.env.UCS01_UNION_SOURCE_CHANNEL ?? 'channel-3',
      rpcUrls: {
        default: {
          /**
           * @see https://docs.union.build/joining_the_testnet/public_endpoints#rpc
           */
          http: [
            'https://union-testnet-rpc.polkachu.com',
            'https://rpc-union-testnet-01.stakeflow.io',
            'https://rpc-t.union.nodestake.top'
          ]
        }
      },
      nativeCurrency: {
        name: 'Union',
        symbol: 'UNO',
        decimals: 6
      },
      token: {
        name: 'Union',
        symbol: 'UNO',
        denom: 'muno',
        decimals: 6,
        address:
          process.env.UCS01_UNION_ADDRESS ??
          'union1mkdwqejs8ph0q0cu4n285g83e4zmsjxdjncjl8rpktgd02jy6gwslm960p'
      }
    }
  }
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
      Record<
        'sepolia',
        Chain & { token: Token; portId: string; channelId: string }
      >
    >
  | Record<
      'union',
      Record<'testnet', Chain & { token: Token; channelId: string }>
    >
