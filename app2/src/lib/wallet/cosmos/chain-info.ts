// @ts-nocheck
//Leaps types doesnt match in their docs in regards to gasPriceStep
import type { ChainInfo as KeplrChainInfo } from "@keplr-wallet/types"

import type { ChainInfo as LeapChainInfo } from "@leapwallet/types"
import { cosmosStore } from "$lib/wallet/cosmos/config.svelte.ts"

//This exists according to docs
interface LeapExtendedInfo extends LeapChainInfo {
  theme: {
    primaryColor: string
    gradient: string
  }
  image: string
}

//Keplr

export const babylonMainnetKeplrChaininfo : KeplrChainInfo = {
    "chainId": "bbn-1",
    "chainName": "Babylon Genesis",
    "chainSymbolImageUrl": "https://raw.githubusercontent.com/chainapsis/keplr-chain-registry/main/images/bbn/chain.png",
    "rpc": "https://rpc-babylon.keplr.app",
    "rest": "https://lcd-babylon.keplr.app",
    "walletUrlForStaking": "https://wallet.keplr.app/chains/babylon-genesis",
    "bip44": {
    "coinType": 118
  },
    "bech32Config": {
    "bech32PrefixAccAddr": "bbn",
      "bech32PrefixAccPub": "bbnpub",
      "bech32PrefixValAddr": "bbnvaloper",
      "bech32PrefixValPub": "bbnvaloperpub",
      "bech32PrefixConsAddr": "bbnvalcons",
      "bech32PrefixConsPub": "bbnvalconspub"
  },
    "currencies": [
    {
      "coinDenom": "BABY",
      "coinMinimalDenom": "ubbn",
      "coinDecimals": 6,
      "coinGeckoId": "babylon",
      "coinImageUrl": "https://raw.githubusercontent.com/chainapsis/keplr-chain-registry/main/images/bbn/chain.png"
    },
    {
      "coinDenom": "LBTC",
      "coinMinimalDenom": "ibc/89EE10FCF78800B572BAAC7080AEFA301B5F3BBC51C5371E907EB129C5B900E7",
      "coinDecimals": 8,
      "coinImageUrl": "https://raw.githubusercontent.com/chainapsis/keplr-chain-registry/main/images/bbn/LBTC.png"
    }
  ],
    "feeCurrencies": [
    {
      "coinDenom": "BABY",
      "coinMinimalDenom": "ubbn",
      "coinDecimals": 6,
      "coinGeckoId": "babylon",
      "coinImageUrl": "https://raw.githubusercontent.com/chainapsis/keplr-chain-registry/main/images/bbn/chain.png",
      "gasPriceStep": {
        "low": 0.007,
        "average": 0.007,
        "high": 0.01
      }
    }
  ],
    "stakeCurrency": {
    "coinDenom": "BABY",
      "coinMinimalDenom": "ubbn",
      "coinDecimals": 6,
      "coinGeckoId": "babylon",
      "coinImageUrl": "https://raw.githubusercontent.com/chainapsis/keplr-chain-registry/main/images/bbn/chain.png"
  },
    "features": [
    "cosmwasm"
  ]
}

export const unionKeplrChainInfo: KeplrChainInfo = {
  chainId: "union-testnet-10",
  chainName: "uniontestnet",
  rest: "https://rest.union-testnet-10.union.chain.cooking",
  rpc: "https://rpc.union-testnet-10.union.chain.cooking",
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
export const babylonTestnetKeplrChaininfo: KeplrChainInfo = {
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

//Leap
export const unionLeapChainInfo: LeapExtendedInfo = {
  chainId: "union-testnet-10",
  chainName: "uniontestnet",
  rest: "https://rpc.union-testnet-10.union.chain.cooking",
  rpc: "https://rpc.union-testnet-10.union.chain.cooking",
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
      coinGeckoId: "stride",
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
      coinGeckoId: "celestia"
    }
  ],
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

export const babylonMainnetLeapChainInfo: LeapExtendedInfo = {
  chainId: "bbn-1",
  chainName: "Babylon Genesis",
  rest: "https://lcd-babylon.keplr.app",
  rpc: "https://rpc-babylon.keplr.app",
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
      coinGeckoId: "babylon"
    },
    {
      coinDenom: "LBTC",
      coinMinimalDenom: "ibc/89EE10FCF78800B572BAAC7080AEFA301B5F3BBC51C5371E907EB129C5B900E7",
      coinDecimals: 8
    }
  ],
  feeCurrencies: [
    {
      coinDenom: "BABY",
      coinMinimalDenom: "ubbn",
      coinDecimals: 6,
      coinGeckoId: "babylon",
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
    coinGeckoId: "babylon"
  },
  image: "https://raw.githubusercontent.com/chainapsis/keplr-chain-registry/main/images/bbn/chain.png",
  theme: {
    primaryColor: "#fff",
    gradient: "linear-gradient(180deg, rgba(255, 255, 255, 0.32) 0%, rgba(255, 255, 255, 0) 100%)"
  },
  features: ["cosmwasm"]
}

export const babylonTestnetLeapChaininfo: LeapExtendedInfo = {
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

//Maps
const keplrChainInfoMap: Record<string, KeplrChainInfo> = {
  "union-testnet-10": unionKeplrChainInfo,
  "stride-internal-1": strideKeplrChainInfo,
  "elgafar-1": elgafarKeplrChainInfo,
  "bbn-test-5": babylonTestnetKeplrChaininfo,
  "bbn-1": babylonMainnetKeplrChaininfo
}

const leapChainInfoMap: Record<string, LeapChainInfo> = {
  "union-testnet-10": unionLeapChainInfo,
  "stride-internal-1": strideLeapChainInfo,
  "elgafar-1": elgafarLeapChainInfo,
  "bbn-test-5": babylonTestnetLeapChaininfo,
  "bbn-1": babylonMainnetLeapChainInfo
}

//Helper functions
export function getCosmosChainInfo(chainId: string): LeapChainInfo | KeplrChainInfo | null {
  const chainInfoMap = cosmosStore.connectedWallet === "leap" ? leapChainInfoMap : keplrChainInfoMap
  return chainInfoMap[chainId] || null
}

export function getHighGasPriceStep(
  chainInfo: KeplrChainInfo
): { amount: string; denom: string } | null {
  if (!chainInfo.currencies || chainInfo.currencies.length === 0) {
    return null
  }

  const firstCurrency = chainInfo.currencies[0]
  const matchedFeeCurrency = chainInfo.feeCurrencies.find(
    feeCurrency => feeCurrency.coinMinimalDenom === firstCurrency.coinMinimalDenom
  )

  return matchedFeeCurrency?.gasPriceStep
    ? {
        amount: matchedFeeCurrency.gasPriceStep.high.toString(),
        denom: firstCurrency.coinMinimalDenom
      }
    : null
}
