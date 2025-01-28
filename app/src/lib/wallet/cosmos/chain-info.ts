import type { ChainInfo as KeplrChainInfo } from "@keplr-wallet/types"
import type { ChainInfo as LeapChainInfo } from "@leapwallet/types"

//This exist according to docs
interface LeapExtendedInfo extends LeapChainInfo {
  theme: {
    primaryColor: string
    gradient: string
  }
  image: string
}

//todo handle this for main-net
export const unionKeplrChainInfo: KeplrChainInfo = {
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
  }
}

//todo handle this for main-net
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
      coinGeckoId: "union"
    }
  ],
  gasPriceStep: {
    low: 0.0025,
    average: 0.025,
    high: 0.04
  },
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

export const strideKeplrChainInfo: KeplrChainInfo = {
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

export const elgafarKeplrChainInfo: KeplrChainInfo = {
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
        "https://raw.githubusercontent.com/chainapsis/keplr-chain-registry/main/images/stargaze/ustars.png"
    }
  ],
  features: []
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
      coinGeckoId: "stargaze"
    }
  ],
  gasPriceStep: {
    low: 0.03,
    average: 0.04,
    high: 0.05
  },
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

const keplrChainInfoMap: Record<string, KeplrChainInfo> = {
  "union-testnet-9": unionKeplrChainInfo,
  "stride-internal-1": strideKeplrChainInfo,
  "elgafar-1": elgafarKeplrChainInfo
}

const leapChainInfoMap: Record<string, LeapChainInfo> = {
  "union-testnet-9": unionLeapChainInfo,
  "stride-internal-1": strideLeapChainInfo,
  "elgafar-1": elgafarLeapChainInfo
  // TODO: add stargaze leap definition
}

export function getCosmosChainInfo(
  chainId: string,
  connectedWallet: string | undefined
): LeapChainInfo | KeplrChainInfo | null {
  const chainInfoMap = connectedWallet === "leap" ? leapChainInfoMap : keplrChainInfoMap
  return chainInfoMap[chainId] || null
}
