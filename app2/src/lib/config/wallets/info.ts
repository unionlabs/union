import { InternalChainInfo } from "$lib/services/cosmos/chain-info/internal-chain-info"

export const unionTestnet = InternalChainInfo.make({
  chainId: "union-testnet-10",
  chainName: "Union Testnet",
  nodeProvider: {
    name: "union",
    email: "ben@union.build",
    website: "https://explorer.testnet-9.union.build",
  },
  chainSymbolImageUrl:
    "https://raw.githubusercontent.com/chainapsis/keplr-chain-registry/main/images/union-testnet/chain.png",
  rest: "https://rest.union-testnet-10.union.chain.cooking",
  rpc: "https://rpc.union-testnet-10.union.chain.cooking",
  bip44: {
    coinType: 118,
  },
  bech32Config: {
    bech32PrefixAccAddr: "union",
    bech32PrefixAccPub: "unionpub",
    bech32PrefixValAddr: "unionvaloper",
    bech32PrefixValPub: "unionvaloperpub",
    bech32PrefixConsAddr: "unionvalcons",
    bech32PrefixConsPub: "unionvalconspub",
  },
  currencies: [
    {
      coinDenom: "UNO",
      coinMinimalDenom: "muno",
      coinDecimals: 6,
      coinImageUrl:
        "https://raw.githubusercontent.com/chainapsis/keplr-chain-registry/main/images/union-testnet/chain.png",
    },
  ],
  feeCurrencies: [
    {
      coinDenom: "UNO",
      coinMinimalDenom: "muno",
      coinDecimals: 6,
      gasPriceStep: {
        low: 0.0025,
        average: 0.025,
        high: 0.04,
      },
      coinImageUrl:
        "https://raw.githubusercontent.com/chainapsis/keplr-chain-registry/main/images/union-testnet/chain.png",
    },
  ],
  stakeCurrency: {
    coinDenom: "UNO",
    coinMinimalDenom: "muno",
    coinDecimals: 6,
    coinImageUrl:
      "https://raw.githubusercontent.com/chainapsis/keplr-chain-registry/main/images/union-testnet/chain.png",
  },
  features: [
    "cosmwasm",
  ],
  theme: {
    primaryColor: "#fff",
    gradient: "linear-gradient(180deg, rgba(255, 255, 255, 0.32) 0%, rgba(255, 255, 255, 0) 100%)",
  },
  image:
    "https://raw.githubusercontent.com/cosmos/chain-registry/master/testnets/uniontestnet/images/union.png",
})

export const babylonMainnet = InternalChainInfo.make({
  chainId: "bbn-1",
  chainName: "Babylon Genesis",
  chainSymbolImageUrl:
    "https://raw.githubusercontent.com/chainapsis/keplr-chain-registry/main/images/bbn/chain.png",
  rpc: "https://rpc-babylon.keplr.app",
  rest: "https://lcd-babylon.keplr.app",
  walletUrlForStaking: "https://wallet.keplr.app/chains/babylon-genesis",
  bip44: {
    coinType: 118,
  },
  bech32Config: {
    bech32PrefixAccAddr: "bbn",
    bech32PrefixAccPub: "bbnpub",
    bech32PrefixValAddr: "bbnvaloper",
    bech32PrefixValPub: "bbnvaloperpub",
    bech32PrefixConsAddr: "bbnvalcons",
    bech32PrefixConsPub: "bbnvalconspub",
  },
  currencies: [
    {
      coinDenom: "BABY",
      coinMinimalDenom: "ubbn",
      coinDecimals: 6,
      coinGeckoId: "babylon",
      coinImageUrl:
        "https://raw.githubusercontent.com/chainapsis/keplr-chain-registry/main/images/bbn/chain.png",
    },
    {
      coinDenom: "LBTC",
      coinMinimalDenom: "ibc/89EE10FCF78800B572BAAC7080AEFA301B5F3BBC51C5371E907EB129C5B900E7",
      coinDecimals: 8,
      coinImageUrl:
        "https://raw.githubusercontent.com/chainapsis/keplr-chain-registry/main/images/bbn/LBTC.png",
    },
  ],
  feeCurrencies: [
    {
      coinDenom: "BABY",
      coinMinimalDenom: "ubbn",
      coinDecimals: 6,
      coinGeckoId: "babylon",
      coinImageUrl:
        "https://raw.githubusercontent.com/chainapsis/keplr-chain-registry/main/images/bbn/chain.png",
      gasPriceStep: {
        low: 0.007,
        average: 0.007,
        high: 0.01,
      },
    },
  ],
  stakeCurrency: {
    coinDenom: "BABY",
    coinMinimalDenom: "ubbn",
    coinDecimals: 6,
    coinGeckoId: "babylon",
    coinImageUrl:
      "https://raw.githubusercontent.com/chainapsis/keplr-chain-registry/main/images/bbn/chain.png",
  },
  features: [
    "cosmwasm",
  ],
  theme: {
    gradient: "linear-gradient(180deg, rgba(247,119,26, 0.32) 0%, rgba(247,119,26, 0) 100%)",
    primaryColor: "#f7771a",
  },
  image:
    "https://raw.githubusercontent.com/chainapsis/keplr-chain-registry/main/images/bbn/chain.png",
})

export const babylonTestnet = InternalChainInfo.make({
  chainId: "bbn-test-5",
  chainName: "Babylon Phase-2 Testnet",
  chainSymbolImageUrl:
    "https://raw.githubusercontent.com/chainapsis/keplr-chain-registry/main/images/bbn-test/chain.png",
  rpc: "https://babylon-testnet-rpc.nodes.guru",
  rest: "https://babylon-testnet-api.nodes.guru",
  nodeProvider: {
    name: "NodesGuru",
    email: "security@nodes.guru",
    website: "https://nodes.guru/",
  },
  bip44: {
    coinType: 118,
  },
  bech32Config: {
    bech32PrefixAccAddr: "bbn",
    bech32PrefixAccPub: "bbnpub",
    bech32PrefixValAddr: "bbnvaloper",
    bech32PrefixValPub: "bbnvaloperpub",
    bech32PrefixConsAddr: "bbnvalcons",
    bech32PrefixConsPub: "bbnvalconspub",
  },
  currencies: [
    {
      coinDenom: "tBABY",
      coinMinimalDenom: "ubbn",
      coinDecimals: 6,
      coinImageUrl:
        "https://raw.githubusercontent.com/chainapsis/keplr-chain-registry/main/images/bbn-test/chain.png",
    },
    {
      coinDenom: "eBABY",
      coinMinimalDenom: "ebbn",
      coinDecimals: 6,
      coinImageUrl:
        "https://raw.githubusercontent.com/chainapsis/keplr-chain-registry/main/images/bbn-test/eBABY.png",
    },
    {
      coinDenom: "LBTC",
      coinMinimalDenom: "ibc/13A78C8607F1ABD49DA5EC474262E3D69312A797FB0026BC4F9961D74EB6E066",
      coinDecimals: 8,
      coinImageUrl:
        "https://raw.githubusercontent.com/chainapsis/keplr-chain-registry/main/images/bbn-test/LBTC.png",
    },
    {
      coinDenom: "tcBABY",
      coinMinimalDenom: "cbbn",
      coinDecimals: 6,
      coinImageUrl:
        "https://raw.githubusercontent.com/chainapsis/keplr-chain-registry/main/images/bbn-test/tcBABY.png",
    },
  ],
  feeCurrencies: [
    {
      coinDenom: "tBABY",
      coinMinimalDenom: "ubbn",
      coinDecimals: 6,
      coinImageUrl:
        "https://raw.githubusercontent.com/chainapsis/keplr-chain-registry/main/images/bbn-test/chain.png",
      gasPriceStep: {
        low: 0.007,
        average: 0.007,
        high: 0.01,
      },
    },
  ],
  stakeCurrency: {
    coinDenom: "tBABY",
    coinMinimalDenom: "ubbn",
    coinDecimals: 6,
    coinImageUrl:
      "https://raw.githubusercontent.com/chainapsis/keplr-chain-registry/main/images/bbn-test/chain.png",
  },
  features: [
    "cosmwasm",
  ],
  theme: {
    gradient: "linear-gradient(180deg, rgba(247,119,26, 0.32) 0%, rgba(247,119,26, 0) 100%)",
    primaryColor: "#f7771a",
  },
  image:
    "https://raw.githubusercontent.com/chainapsis/keplr-chain-registry/main/images/bbn-test/chain.png",
})

export const xionTestnet = InternalChainInfo.make(
  {
    rpc: "https://rpc.xion-testnet-2.burnt.com/",
    rest: "https://api.xion-testnet-2.burnt.com/",
    chainId: "xion-testnet-2",
    chainName: "Xion Testnet",
    chainSymbolImageUrl:
      "https://raw.githubusercontent.com/chainapsis/keplr-chain-registry/main/images/xion-testnet/chain.png",
    stakeCurrency: {
      coinDenom: "XION",
      coinMinimalDenom: "uxion",
      coinDecimals: 6,
      coinImageUrl:
        "https://raw.githubusercontent.com/chainapsis/keplr-chain-registry/main/images/xion-testnet/chain.png",
    },
    bip44: {
      coinType: 118,
    },
    bech32Config: {
      bech32PrefixAccAddr: "xion",
      bech32PrefixAccPub: "xionpub",
      bech32PrefixValAddr: "xionvaloper",
      bech32PrefixValPub: "xionvaloperpub",
      bech32PrefixConsAddr: "xionvalcons",
      bech32PrefixConsPub: "xionvalconspub",
    },
    currencies: [
      {
        coinDenom: "XION",
        coinMinimalDenom: "uxion",
        coinDecimals: 6,
        coinImageUrl:
          "https://raw.githubusercontent.com/chainapsis/keplr-chain-registry/main/images/xion-testnet/chain.png",
      },
    ],
    feeCurrencies: [
      {
        coinDenom: "XION",
        coinMinimalDenom: "uxion",
        coinDecimals: 6,
        coinImageUrl:
          "https://raw.githubusercontent.com/chainapsis/keplr-chain-registry/main/images/xion-testnet/chain.png",
        gasPriceStep: {
          low: 0.001,
          average: 0.001,
          high: 0.002,
        },
      },
    ],
    features: [
      "cosmwasm",
    ],
    nodeProvider: {
      name: "🔥BurntLabs🔥",
      email: "security@burnt.com",
      website: "https://xion.burnt.com",
    },
    theme: {
      gradient: "linear-gradient(180deg, rgba(50, 129, 250, 0.32) 0%, rgba(50, 129, 250, 0) 100%)",
      primaryColor: "#3281fa",
    },
    image:
      "https://raw.githubusercontent.com/chainapsis/keplr-chain-registry/main/images/xion-testnet/chain.png",
  },
)
