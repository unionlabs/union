import { UniversalChainId } from "../schema/index.js"
import { TokenRawDenom } from "../schema/index.js"

interface GasDenomMetadata {
  address: TokenRawDenom
  name: string
  symbol: string
  decimals: number
}

const ETH_METADATA = {
  address: TokenRawDenom.make("0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"),
  name: "Ether",
  symbol: "ETH",
  decimals: 18,
} as const

const SEI_METADATA = {
  address: TokenRawDenom.make("0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"),
  name: "Sei",
  symbol: "SEI",
  decimals: 18,
} as const

const CORN_METADATA = {
  address: TokenRawDenom.make("0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"),
  name: "Corn",
  symbol: "CORN",
  decimals: 18,
} as const

export const GAS_DENOMS: Record<UniversalChainId, GasDenomMetadata> = {
  // Ethereum chains
  [UniversalChainId.make("ethereum.11155111")]: ETH_METADATA,
  [UniversalChainId.make("ethereum.1")]: ETH_METADATA,
  [UniversalChainId.make("ethereum.17000")]: ETH_METADATA,

  // Bob
  [UniversalChainId.make("bob.60808")]: ETH_METADATA,
  [UniversalChainId.make("bob.808813")]: ETH_METADATA,

  // Sei chains
  [UniversalChainId.make("sei.pacific-1")]: SEI_METADATA,
  [UniversalChainId.make("sei.atlantic-2")]: SEI_METADATA,
  [UniversalChainId.make("sei.1328")]: SEI_METADATA,
  [UniversalChainId.make("sei.1329")]: SEI_METADATA,

  // Corn
  [UniversalChainId.make("corn.21000000")]: CORN_METADATA,
  [UniversalChainId.make("corn.21000001")]: CORN_METADATA,
}
