import type { ChainInfo as LeapChainInfo } from "@leapwallet/types"

interface LeapExtendedInfo extends LeapChainInfo {
  theme: {
    primaryColor: string
    gradient: string
  }
  image: string
}

export const unionLeapChainInfo: LeapExtendedInfo = {
  chainId: "union-testnet-9",
  chainName: "uniontestnet",
  rest: "https://rest.testnet-9.union.build",
  rpc: "https://rpc.testnet-9.union.build",
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
  },
  theme: {
    primaryColor: "#fff",
    gradient: "linear-gradient(180deg, rgba(255, 255, 255, 0.32) 0%, rgba(255, 255, 255, 0) 100%)"
  },
  image:
    "https://raw.githubusercontent.com/cosmos/chain-registry/master/testnets/uniontestnet/images/union.png"
}

export const elgafarLeapChainInfo: LeapExtendedInfo = {
  chainId: "elgafar-1",
  chainName: "Stargaze Testnet",
  rpc: "https://rpc.elgafar-1.stargaze-apis.com",
  rest: "https://rest.elgafar-1.stargaze-apis.com",
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
      coinGeckoId: "stargaze"
    }
  ],
  feeCurrencies: [
    {
      coinDenom: "STARS",
      coinMinimalDenom: "ustars",
      coinDecimals: 6,
      coinGeckoId: "stargaze",
      gasPriceStep: {
        low: 0.03,
        average: 0.04,
        high: 0.05
      }
    }
  ],
  stakeCurrency: {
    coinDenom: "STARS",
    coinMinimalDenom: "ustars",
    coinDecimals: 6,
    coinGeckoId: "stargaze"
  },
  theme: {
    primaryColor: "#E2447B",
    gradient: "linear-gradient(180deg, rgba(226, 68, 123, 0.32) 0%, rgba(226, 68, 123, 0) 100%)"
  },
  image:
    "https://raw.githubusercontent.com/cosmostation/chainlist/main/chain/stargaze/asset/stargaze.png"
}

export const strideLeapChainInfo: LeapExtendedInfo = {
  chainId: "stride-internal-1",
  chainName: "Stride Testnet",
  rpc: "https://stride.testnet-1.stridenet.co",
  rest: "https://stride.testnet-1.stridenet.co/api",
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
      coinGeckoId: "stride"
    }
  ],
  feeCurrencies: [
    {
      coinDenom: "STRD",
      coinMinimalDenom: "ustrd",
      coinDecimals: 6,
      coinGeckoId: "stride"
    },
    {
      coinDenom: "TIA",
      coinMinimalDenom: "ibc/1A7653323C1A9E267FF7BEBF40B3EEA8065E8F069F47F2493ABC3E0B621BF793",
      coinDecimals: 6,
      coinGeckoId: "celestia"
    }
  ],
  gasPriceStep: {
    low: 0.0005,
    average: 0.005,
    high: 0.05
  },
  stakeCurrency: {
    coinDenom: "STRD",
    coinMinimalDenom: "ustrd",
    coinDecimals: 6,
    coinGeckoId: "stride"
  },
  theme: {
    primaryColor: "#E91179",
    gradient: "linear-gradient(180deg, rgba(233, 17, 121, 0.32) 0%, rgba(233, 17, 121, 0) 100%)"
  },
  image:
    "https://raw.githubusercontent.com/chainapsis/keplr-chain-registry/main/images/stride-internal/chain.png"
}

export const babylonLeapChaininfo: LeapExtendedInfo = {
  chainId: "bbn-test-5",
  chainName: "Babylon Phase-2 Testnet",
  rest: "https://babylon-testnet-api.nodes.guru",
  rpc: "https://babylon-testnet-rpc.nodes.guru",
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
      coinGeckoId: ""
    }
  ],
  feeCurrencies: [
    {
      coinDenom: "BABY",
      coinMinimalDenom: "ubbn",
      coinDecimals: 6,
      coinGeckoId: "",
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
    coinGeckoId: ""
  },
  image:
    "https://raw.githubusercontent.com/chainapsis/keplr-chain-registry/main/images/bbn-test/chain.png",
  theme: {
    primaryColor: "#fff",
    gradient: "linear-gradient(180deg, rgba(255, 255, 255, 0.32) 0%, rgba(255, 255, 255, 0) 100%)"
  }
}
