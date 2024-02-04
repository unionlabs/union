import type { Chain } from "viem";
import { sepolia } from "viem/chains";

export const chainIds = ["1", "32382", "11155111"] as const;
export type ChainId = (typeof chainIds)[number];

/**
 * TODO: Add `ethereum.mainnet` and `union.mainnet` info on launch
 */
export const chain = {
  ethereum: {
    sepolia: {
      ...sepolia,
      portId: process.env.UCS01_SEPOLIA_PORT_ID,
      channelId: process.env.UCS01_SEPOLIA_SOURCE_CHANNEL,
      token: {
        name: "Union",
        symbol: "UNO",
        denom: "muno",
        decimals: 6,
        address: process.env.MUNO_ERC20_ADDRESS,
      },
    },
  },
  union: {
    testnet: {
      name: "union-testnet-5",
      id: 32_382,
      channelId: process.env.UCS01_UNION_SOURCE_CHANNEL,
      rpcUrls: {
        default: {
          /**
           * @see https://docs.union.build/joining_the_testnet/public_endpoints#rpc
           */
          http: [
            "https://rpc.testnet.bonlulu.uno",
            "https://union-testnet-rpc.polkachu.com",
          ],
        },
      },
      nativeCurrency: {
        name: "Union",
        symbol: "UNO",
        decimals: 6,
      },
      token: {
        name: "Union",
        symbol: "UNO",
        denom: "muno",
        decimals: 6,
        address: process.env.UCS01_UNION_ADDRESS,
      },
    },
  },
} as const satisfies TChain;

type Token = {
  name: string;
  symbol: string;
  denom: string;
  decimals: number;
  address: string;
};

type TChain =
  | Record<
      "ethereum",
      Record<
        "sepolia",
        Chain & { token: Token; portId: string; channelId: string }
      >
    >
  | Record<
      "union",
      Record<"testnet", Chain & { token: Token; channelId: string }>
    >;
