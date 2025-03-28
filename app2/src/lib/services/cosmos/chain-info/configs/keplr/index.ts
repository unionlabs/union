import type { ChainInfo } from "@keplr-wallet/types"

export const unionKeplrChainInfo: ChainInfo = {
  chainId: "union-testnet-10",
  chainName: "uniontestnet",
  rest: "https://rest.union-testnet-10.union.chain.cooking",
  rpc: "https://rest.union-testnet-10.union.chain.cooking",
  bip44: {
    coinType: 118
  },
  bech32Config: {
    bech32PrefixAccAddr: "union",
    bech32PrefixAccPub: "unionpub",
    bech32PrefixValAddr: "unionvaloper",
    bech32PrefixValPub: "unionvaloperpub",
    bech32PrefixConsAddr: "unionvalcons",
    bech32PrefixConsPub: "unionvalconspub"
  },
  currencies: [
    {
      coinDenom: "UNO",
      coinMinimalDenom: "muno",
      coinDecimals: 6,
      coinGeckoId: "cosmos"
    }
  ],
  feeCurrencies: [
    {
      coinDenom: "UNO",
      coinMinimalDenom: "muno",
      coinDecimals: 6,
      coinGeckoId: "union",
      gasPriceStep: {
        low: 0.0025,
        average: 0.025,
        high: 0.04
      }
    }
  ],
  stakeCurrency: {
    coinDenom: "UNO",
    coinMinimalDenom: "muno",
    coinDecimals: 6,
    coinGeckoId: "union"
  }
}

// source: https://github.com/chainapsis/keplr-chain-registry/blob/main/cosmos/bbn-test.json
export const babylonKeplrChaininfo: ChainInfo = {
  chainId: "bbn-test-5",
  chainName: "Babylon Phase-2 Testnet",
  chainSymbolImageUrl:
    "https://raw.githubusercontent.com/chainapsis/keplr-chain-registry/main/images/bbn-test/chain.png",
  rpc: "https://babylon-testnet-rpc.nodes.guru",
  rest: "https://babylon-testnet-api.nodes.guru",
  nodeProvider: {
    name: "NodesGuru",
    email: "security@nodes.guru",
    website: "https://nodes.guru/"
  },
  bip44: {
    coinType: 118
  },
  bech32Config: {
    bech32PrefixAccAddr: "bbn",
    bech32PrefixAccPub: "bbnpub",
    bech32PrefixValAddr: "bbnvaloper",
    bech32PrefixValPub: "bbnvaloperpub",
    bech32PrefixConsAddr: "bbnvalcons",
    bech32PrefixConsPub: "bbnvalconspub"
  },
  currencies: [
    {
      coinDenom: "BABY",
      coinMinimalDenom: "ubbn",
      coinDecimals: 6,
      coinImageUrl:
        "https://raw.githubusercontent.com/chainapsis/keplr-chain-registry/main/images/bbn-test/chain.png"
    }
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
        high: 0.01
      }
    }
  ],
  stakeCurrency: {
    coinDenom: "BABY",
    coinMinimalDenom: "ubbn",
    coinDecimals: 6,
    coinImageUrl:
      "https://raw.githubusercontent.com/chainapsis/keplr-chain-registry/main/images/bbn-test/chain.png"
  },
  features: ["cosmwasm"]
}

export const strideKeplrChainInfo: ChainInfo = {
  rpc: "https://stride.testnet-1.stridenet.co",
  rest: "https://stride.testnet-1.stridenet.co/api",
  chainId: "stride-internal-1",
  chainName: "Stride Testnet",
  chainSymbolImageUrl:
    "https://raw.githubusercontent.com/chainapsis/keplr-chain-registry/main/images/stride-internal/chain.png",
  nodeProvider: {
    name: "Stride Labs",
    email: "hello@stridelabs.co",
    website: "https://stride.zone/"
  },
  stakeCurrency: {
    coinDenom: "STRD",
    coinMinimalDenom: "ustrd",
    coinDecimals: 6,
    coinImageUrl:
      "https://raw.githubusercontent.com/chainapsis/keplr-chain-registry/main/images/stride-internal/ustrd.png"
  },
  bip44: {
    coinType: 118
  },
  bech32Config: {
    bech32PrefixAccAddr: "stride",
    bech32PrefixAccPub: "stridepub",
    bech32PrefixValAddr: "stridevaloper",
    bech32PrefixValPub: "stridevaloperpub",
    bech32PrefixConsAddr: "stridevalcons",
    bech32PrefixConsPub: "stridevalconspub"
  },
  currencies: [
    {
      coinDenom: "STRD",
      coinMinimalDenom: "ustrd",
      coinDecimals: 6,
      coinImageUrl:
        "https://raw.githubusercontent.com/chainapsis/keplr-chain-registry/main/images/stride-internal/ustrd.png"
    }
  ],
  feeCurrencies: [
    {
      coinDenom: "STRD",
      coinMinimalDenom: "ustrd",
      coinDecimals: 6,
      coinImageUrl:
        "https://raw.githubusercontent.com/chainapsis/keplr-chain-registry/main/images/stride-internal/ustrd.png",
      gasPriceStep: {
        low: 0.0005,
        average: 0.005,
        high: 0.05
      }
    },
    {
      coinDenom: "TIA",
      coinMinimalDenom: "ibc/1A7653323C1A9E267FF7BEBF40B3EEA8065E8F069F47F2493ABC3E0B621BF793",
      coinDecimals: 6,
      gasPriceStep: {
        low: 0.01,
        average: 0.01,
        high: 0.01
      }
    }
  ],
  features: []
}

export const elgafarKeplrChainInfo: ChainInfo = {
  rpc: "https://rpc.elgafar-1.stargaze-apis.com",
  rest: "https://rest.elgafar-1.stargaze-apis.com",
  chainId: "elgafar-1",
  chainName: "Stargaze Testnet",
  nodeProvider: {
    name: "Stargaze",
    email: "admin@stargaze.zone",
    website: "https://www.stargaze.zone/"
  },
  chainSymbolImageUrl:
    "https://raw.githubusercontent.com/chainapsis/keplr-chain-registry/main/images/stargaze/chain.png",
  stakeCurrency: {
    coinDenom: "STARS",
    coinMinimalDenom: "ustars",
    coinDecimals: 6,
    coinImageUrl:
      "https://raw.githubusercontent.com/chainapsis/keplr-chain-registry/main/images/stargaze/ustars.png"
  },
  bip44: {
    coinType: 118
  },
  bech32Config: {
    bech32PrefixAccAddr: "stars",
    bech32PrefixAccPub: "starspub",
    bech32PrefixValAddr: "starsvaloper",
    bech32PrefixValPub: "starsvaloperpub",
    bech32PrefixConsAddr: "starsvalcons",
    bech32PrefixConsPub: "starsvalconspub"
  },
  currencies: [
    {
      coinDenom: "STARS",
      coinMinimalDenom: "ustars",
      coinDecimals: 6,
      coinImageUrl:
        "https://raw.githubusercontent.com/chainapsis/keplr-chain-registry/main/images/stargaze/ustars.png"
    }
  ],
  feeCurrencies: [
    {
      coinDenom: "STARS",
      coinMinimalDenom: "ustars",
      coinDecimals: 6,
      coinImageUrl:
        "https://raw.githubusercontent.com/chainapsis/keplr-chain-registry/main/images/stargaze/ustars.png",
      gasPriceStep: {
        low: 0.0005,
        average: 0.0025,
        high: 0.025
      }
    }
  ],
  features: []
}
