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
  chainId: "union-testnet-8",
  chainName: "uniontestnet",
  rest: "https://rest.testnet-8.union.build",
  rpc: "https://rpc.testnet-8.union.build",
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
  chainId: "union-testnet-8",
  chainName: "uniontestnet",
  rest: "https://rest.testnet-8.union.build",
  rpc: "https://rpc.testnet-8.union.build",
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
