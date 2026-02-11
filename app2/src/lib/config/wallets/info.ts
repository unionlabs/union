import { InternalChainInfo } from "$lib/services/cosmos/chain-info/internal-chain-info"

export const unionMainnet = InternalChainInfo.make({
  chainId: "union-1",
  chainName: "Union",
  rpc: "https://rpc.union.build",
  rest: "https://rest.union.build",
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
      coinDenom: "U",
      coinMinimalDenom: "au",
      coinDecimals: 18,
      coinGeckoId: "union-2",
      coinImageUrl:
        "https://raw.githubusercontent.com/chainapsis/keplr-chain-registry/main/images/union/chain.png",
    },
  ],
  feeCurrencies: [
    {
      coinDenom: "U",
      coinMinimalDenom: "au",
      coinDecimals: 18,
      coinGeckoId: "union-2",
      coinImageUrl:
        "https://raw.githubusercontent.com/chainapsis/keplr-chain-registry/main/images/union/chain.png",
      gasPriceStep: {
        low: 100000000,
        average: 100000000,
        high: 200000000,
      },
    },
  ],
  stakeCurrency: {
    coinDenom: "U",
    coinMinimalDenom: "au",
    coinDecimals: 18,
    coinGeckoId: "union-2",
    coinImageUrl:
      "https://raw.githubusercontent.com/chainapsis/keplr-chain-registry/main/images/union/chain.png",
  },
  features: [
    "cosmwasm",
  ],
  beta: true,
  theme: {
    primaryColor: "#fff",
    gradient: "linear-gradient(180deg, rgba(255, 255, 255, 0.32) 0%, rgba(255, 255, 255, 0) 100%)",
  },
  image:
    "https://raw.githubusercontent.com/chainapsis/keplr-chain-registry/main/images/union/chain.png",
  chainSymbolImageUrl:
    "https://raw.githubusercontent.com/chainapsis/keplr-chain-registry/main/images/union/chain.png",
  walletUrlForStaking: "https://wallet.keplr.app/chains/union",
})

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
  rest: "https://rest.union-testnet-10.union.chain.kitchen",
  rpc: "https://rpc.union-testnet-10.union.chain.kitchen",
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
      coinDenom: "U",
      coinMinimalDenom: "au",
      coinDecimals: 18,
      coinImageUrl:
        "https://raw.githubusercontent.com/chainapsis/keplr-chain-registry/main/images/union-testnet/chain.png",
    },
  ],
  feeCurrencies: [
    {
      coinDenom: "U",
      coinMinimalDenom: "au",
      coinDecimals: 18,
      gasPriceStep: {
        low: 100000000000,
        average: 100000000000,
        high: 200000000000,
      },
      coinImageUrl:
        "https://raw.githubusercontent.com/chainapsis/keplr-chain-registry/main/images/union-testnet/chain.png",
    },
  ],
  stakeCurrency: {
    coinDenom: "U",
    coinMinimalDenom: "au",
    coinDecimals: 18,
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
  rpc: "https://rpc.bbn-1.babylon.chain.kitchen",
  rest: "https://rest.bbn-1.babylon.chain.kitchen",
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
  rpc: "https://rpc.bbn-test-5.babylon.chain.kitchen/",
  rest: "https://rest.bbn-test-5.babylon.chain.kitchen/",
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
      coinDenom: "BABY",
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
      coinDenom: "BABY",
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
    rpc: "https://rpc.xion-testnet-2.xion.chain.kitchen",
    rest: "https://rest.xion-testnet-2.xion.chain.kitchen",
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

export const osmosisTestnet = InternalChainInfo.make({
  chainId: "osmo-test-5",
  chainName: "Osmosis Testnet",
  chainSymbolImageUrl:
    "https://raw.githubusercontent.com/chainapsis/keplr-chain-registry/main/images/osmosis/chain.png",
  rpc: "https://rpc.osmo-test-5.osmosis.chain.kitchen",
  rest: "https://rest.osmo-test-5.osmosis.chain.kitchen",
  bip44: {
    coinType: 118,
  },
  bech32Config: {
    bech32PrefixAccAddr: "osmo",
    bech32PrefixAccPub: "osmopub",
    bech32PrefixValAddr: "osmovaloper",
    bech32PrefixValPub: "osmovaloperpub",
    bech32PrefixConsAddr: "osmovalcons",
    bech32PrefixConsPub: "osmovalconspub",
  },
  currencies: [
    {
      coinDenom: "OSMO",
      coinMinimalDenom: "uosmo",
      coinDecimals: 6,
      coinImageUrl:
        "https://raw.githubusercontent.com/chainapsis/keplr-chain-registry/main/images/osmosis/uosmo.png",
    },
    {
      coinDenom: "ION",
      coinMinimalDenom: "uion",
      coinDecimals: 6,
      coinImageUrl:
        "https://raw.githubusercontent.com/chainapsis/keplr-chain-registry/main/images/osmosis/uion.png",
    },
  ],
  feeCurrencies: [
    {
      coinDenom: "OSMO",
      coinMinimalDenom: "uosmo",
      coinDecimals: 6,
      coinImageUrl:
        "https://raw.githubusercontent.com/chainapsis/keplr-chain-registry/main/images/osmosis/uosmo.png",
      gasPriceStep: {
        low: 0.0025,
        average: 0.025,
        high: 0.04,
      },
    },
  ],
  stakeCurrency: {
    coinDenom: "OSMO",
    coinMinimalDenom: "uosmo",
    coinDecimals: 6,
    coinImageUrl:
      "https://raw.githubusercontent.com/chainapsis/keplr-chain-registry/main/images/osmosis/uosmo.png",
  },
  features: [],
  theme: {
    gradient: "linear-gradient(180deg, rgba(181, 97, 219, 0.32) 0%, rgba(181, 97, 219, 0) 100%)",
    primaryColor: "#b561db",
  },
  image:
    "https://raw.githubusercontent.com/chainapsis/keplr-chain-registry/main/images/osmosis/chain.png",
})

export const stargazeMainnet = InternalChainInfo.make({
  rpc: "https://rpc.stargaze-apis.com/",
  rest: "https://rest.stargaze-apis.com/",
  chainId: "stargaze-1",
  chainName: "Stargaze",
  chainSymbolImageUrl:
    "https://raw.githubusercontent.com/cosmos/chain-registry/master/stargaze/images/stars.png",
  stakeCurrency: {
    coinDenom: "STARS",
    coinMinimalDenom: "ustars",
    coinDecimals: 6,
    coinGeckoId: "stargaze",
    coinImageUrl:
      "https://raw.githubusercontent.com/cosmos/chain-registry/master/stargaze/images/stars.png",
  },
  walletUrlForStaking: "https://wallet.keplr.app/chains/stargaze",
  bip44: {
    coinType: 118,
  },
  bech32Config: {
    bech32PrefixAccAddr: "stars",
    bech32PrefixAccPub: "starspub",
    bech32PrefixValAddr: "starsvaloper",
    bech32PrefixValPub: "starsvaloperpub",
    bech32PrefixConsAddr: "starsvalcons",
    bech32PrefixConsPub: "starsvalconspub",
  },
  currencies: [
    {
      coinDenom: "STARS",
      coinMinimalDenom: "ustars",
      coinDecimals: 6,
      coinGeckoId: "stargaze",
      coinImageUrl:
        "https://raw.githubusercontent.com/cosmos/chain-registry/master/stargaze/images/stars.png",
    },
  ],
  feeCurrencies: [
    {
      coinDenom: "STARS",
      coinMinimalDenom: "ustars",
      coinDecimals: 6,
      coinGeckoId: "stargaze",
      coinImageUrl:
        "https://raw.githubusercontent.com/cosmos/chain-registry/master/stargaze/images/stars.png",
      gasPriceStep: {
        low: 1,
        average: 1.1,
        high: 1.2,
      },
    },
  ],
  features: [
    "cosmwasm",
  ],
  theme: {
    gradient: "linear-gradient(180deg, rgba(255, 20, 147, 0.32) 0%, rgba(255, 20, 147, 0) 100%)",
    primaryColor: "#ff1493",
  },
  image: "https://raw.githubusercontent.com/cosmos/chain-registry/master/stargaze/images/stars.png",
})

export const osmosisMainnet = InternalChainInfo.make({
  rpc: "https://rpc.osmosis.zone/",
  rest: "https://lcd.osmosis.zone/",
  chainId: "osmosis-1",
  chainName: "Osmosis",
  chainSymbolImageUrl:
    "https://raw.githubusercontent.com/chainapsis/keplr-chain-registry/main/images/osmosis/chain.png",
  stakeCurrency: {
    coinDenom: "OSMO",
    coinMinimalDenom: "uosmo",
    coinDecimals: 6,
    coinGeckoId: "osmosis",
    coinImageUrl:
      "https://raw.githubusercontent.com/chainapsis/keplr-chain-registry/main/images/osmosis/uosmo.png",
  },
  walletUrl: "https://app.osmosis.zone",
  walletUrlForStaking: "https://wallet.keplr.app/chains/osmosis",
  bip44: {
    coinType: 118,
  },
  bech32Config: {
    bech32PrefixAccAddr: "osmo",
    bech32PrefixAccPub: "osmopub",
    bech32PrefixValAddr: "osmovaloper",
    bech32PrefixValPub: "osmovaloperpub",
    bech32PrefixConsAddr: "osmovalcons",
    bech32PrefixConsPub: "osmovalconspub",
  },
  currencies: [
    {
      coinDenom: "OSMO",
      coinMinimalDenom: "uosmo",
      coinDecimals: 6,
      coinGeckoId: "osmosis",
      coinImageUrl:
        "https://raw.githubusercontent.com/chainapsis/keplr-chain-registry/main/images/osmosis/uosmo.png",
    },
    {
      coinDenom: "ION",
      coinMinimalDenom: "uion",
      coinDecimals: 6,
      coinGeckoId: "ion",
      coinImageUrl:
        "https://raw.githubusercontent.com/chainapsis/keplr-chain-registry/main/images/osmosis/uion.png",
    },
  ],
  feeCurrencies: [
    {
      coinDenom: "OSMO",
      coinMinimalDenom: "uosmo",
      coinDecimals: 6,
      coinGeckoId: "osmosis",
      coinImageUrl:
        "https://raw.githubusercontent.com/chainapsis/keplr-chain-registry/main/images/osmosis/uosmo.png",
      gasPriceStep: {
        low: 0.0025,
        average: 0.025,
        high: 0.04,
      },
    },
  ],
  features: [
    "cosmwasm",
    "osmosis-txfees",
    "osmosis-base-fee-beta",
  ],
  theme: {
    gradient: "linear-gradient(180deg, rgba(181, 97, 219, 0.32) 0%, rgba(181, 97, 219, 0) 100%)",
    primaryColor: "#b561db",
  },
  image:
    "https://raw.githubusercontent.com/chainapsis/keplr-chain-registry/main/images/osmosis/chain.png",
})
